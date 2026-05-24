use std::net::IpAddr;

use ipnet::IpNet;

use crate::engine::ast::{Match, MatchKind};
use crate::engine::packet::{parse_state_bit, Packet, Proto};

/// Devuelve true si el paquete satisface el matcher dado.
/// La negación se aplica al resultado final.
pub fn matches_packet(m: &Match, pkt: &Packet) -> bool {
    let result = match &m.kind {
        MatchKind::Protocol(p) => match_proto(p, pkt.proto),
        MatchKind::Source(s) => match_ip_prefix(s, &pkt.src_ip),
        MatchKind::Destination(d) => match_ip_prefix(d, &pkt.dst_ip),
        MatchKind::DPort(ps) => pkt.dst_port >= ps.start && pkt.dst_port <= ps.end,
        MatchKind::SPort(ps) => pkt.src_port >= ps.start && pkt.src_port <= ps.end,
        MatchKind::InInterface(i) => iface_matches(&pkt.in_iface, i),
        MatchKind::OutInterface(i) => iface_matches(&pkt.out_iface, i),
        MatchKind::State(states) | MatchKind::CtState(states) => {
            let mask: u8 = states
                .iter()
                .map(|s| parse_state_bit(s))
                .fold(0, |a, b| a | b);
            // Si la máscara es 0 (estados desconocidos), no filtramos
            mask == 0 || (pkt.state & mask != 0)
        }
        // ICMP type y Comment nunca bloquean
        MatchKind::IcmpType(_) | MatchKind::Comment(_) => true,
    };
    if m.negated {
        !result
    } else {
        result
    }
}

fn match_proto(proto_str: &str, pkt_proto: Proto) -> bool {
    match proto_str.to_lowercase().as_str() {
        "all" | "0" => true,
        "tcp" => pkt_proto == Proto::Tcp,
        "udp" => pkt_proto == Proto::Udp,
        "icmp" => pkt_proto == Proto::Icmp,
        _ => true, // desconocido → tolerante
    }
}

/// Comprueba si una IP cae dentro de un prefijo CIDR o es exactamente igual.
fn match_ip_prefix(prefix_str: &str, ip_str: &str) -> bool {
    let ip: IpAddr = match ip_str.parse() {
        Ok(ip) => ip,
        Err(_) => return true, // IP inválida en el paquete → tolerante
    };
    // Intentar como CIDR primero, luego como IP exacta
    if let Ok(net) = prefix_str.parse::<IpNet>() {
        net.contains(&ip)
    } else if let Ok(addr) = prefix_str.parse::<IpAddr>() {
        addr == ip
    } else {
        true // prefijo inválido → tolerante
    }
}

/// Soporte de wildcard '*' al final del nombre de interfaz (p.ej. "eth+").
fn iface_matches(iface: &str, pattern: &str) -> bool {
    if pattern.ends_with('*') || pattern.ends_with('+') {
        let prefix = &pattern[..pattern.len() - 1];
        iface.starts_with(prefix)
    } else {
        iface == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ast::{Match, MatchKind, PortSpec};
    use crate::engine::packet::{Packet, Proto, STATE_ESTABLISHED, STATE_NEW};

    fn pkt() -> Packet {
        Packet {
            src_ip: "192.168.1.10".into(),
            dst_ip: "192.168.1.1".into(),
            src_port: 54321,
            dst_port: 80,
            proto: Proto::Tcp,
            in_iface: "eth0".into(),
            out_iface: "eth1".into(),
            state: STATE_NEW,
        }
    }

    #[test]
    fn test_proto_match() {
        let m = Match {
            negated: false,
            kind: MatchKind::Protocol("tcp".into()),
        };
        assert!(matches_packet(&m, &pkt()));
        let m2 = Match {
            negated: false,
            kind: MatchKind::Protocol("udp".into()),
        };
        assert!(!matches_packet(&m2, &pkt()));
    }

    #[test]
    fn test_proto_negated() {
        let m = Match {
            negated: true,
            kind: MatchKind::Protocol("udp".into()),
        };
        assert!(matches_packet(&m, &pkt()));
    }

    #[test]
    fn test_src_cidr() {
        let m = Match {
            negated: false,
            kind: MatchKind::Source("192.168.1.0/24".into()),
        };
        assert!(matches_packet(&m, &pkt()));
        let m2 = Match {
            negated: false,
            kind: MatchKind::Source("10.0.0.0/8".into()),
        };
        assert!(!matches_packet(&m2, &pkt()));
    }

    #[test]
    fn test_dport() {
        let m = Match {
            negated: false,
            kind: MatchKind::DPort(PortSpec { start: 80, end: 80 }),
        };
        assert!(matches_packet(&m, &pkt()));
        let m2 = Match {
            negated: false,
            kind: MatchKind::DPort(PortSpec {
                start: 443,
                end: 443,
            }),
        };
        assert!(!matches_packet(&m2, &pkt()));
    }

    #[test]
    fn test_state_new() {
        let m = Match {
            negated: false,
            kind: MatchKind::State(vec!["NEW".into()]),
        };
        assert!(matches_packet(&m, &pkt()));
        let m2 = Match {
            negated: false,
            kind: MatchKind::State(vec!["ESTABLISHED".into()]),
        };
        assert!(!matches_packet(&m2, &pkt()));
    }

    #[test]
    fn test_state_multi() {
        let m = Match {
            negated: false,
            kind: MatchKind::State(vec!["NEW".into(), "ESTABLISHED".into()]),
        };
        assert!(matches_packet(&m, &pkt()));
        let mut p2 = pkt();
        p2.state = STATE_ESTABLISHED;
        assert!(matches_packet(&m, &p2));
    }

    #[test]
    fn test_iface_wildcard() {
        let m = Match {
            negated: false,
            kind: MatchKind::InInterface("eth+".into()),
        };
        assert!(matches_packet(&m, &pkt()));
        let m2 = Match {
            negated: false,
            kind: MatchKind::InInterface("eth*".into()),
        };
        assert!(matches_packet(&m2, &pkt()));
    }
}
