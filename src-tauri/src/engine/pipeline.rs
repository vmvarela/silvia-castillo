use crate::engine::ast::{RuleCommand, Target};
use crate::engine::matchers::matches_packet;
use crate::engine::packet::Packet;
use crate::engine::ruleset::Ruleset;
use crate::engine::topology::Topology;
use crate::engine::trace::Trace;

const MAX_DEPTH: u32 = 20;

/// Veredicto interno durante la evaluación de cadenas.
#[derive(Debug, Clone, PartialEq)]
enum InternalVerdict {
    Accept,
    Drop,
    Reject,
    Return,
    Jump(String),
    /// LOG u otros targets no terminales: la evaluación continúa.
    Continue,
}

impl InternalVerdict {
    fn as_str(&self) -> &str {
        match self {
            InternalVerdict::Accept => "ACCEPT",
            InternalVerdict::Drop => "DROP",
            InternalVerdict::Reject => "REJECT",
            InternalVerdict::Return => "RETURN",
            InternalVerdict::Jump(_) => "JUMP",
            InternalVerdict::Continue => "CONTINUE",
        }
    }
}

/// Veredicto final de un paquete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalVerdict {
    Accept,
    Drop,
    Reject,
}

impl FinalVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            FinalVerdict::Accept => "ACCEPT",
            FinalVerdict::Drop => "DROP",
            FinalVerdict::Reject => "REJECT",
        }
    }
}

/// Evalúa un paquete contra el ruleset siguiendo el pipeline de Netfilter.
///
/// Pipeline con topología:
///   - SrcIP local → filter/OUTPUT → nat/POSTROUTING
///   - SrcIP no local → nat/PREROUTING → routing → filter/INPUT o FORWARD → nat/POSTROUTING (FORWARD)
///
/// Pipeline sin topología (modo simple):
///   - DstIP local → filter/INPUT
///   - SrcIP local → filter/OUTPUT
///   - else → filter/FORWARD
pub fn evaluate(
    ruleset: &Ruleset,
    topo: Option<&Topology>,
    mut pkt: Packet,
) -> (FinalVerdict, Trace) {
    let mut trace = Trace::default();

    // Asignar interfaces desde la topología si no están ya establecidas
    if let Some(t) = topo {
        if pkt.in_iface.is_empty() {
            pkt.in_iface = t.in_iface_for(&pkt.src_ip);
        }
        if pkt.out_iface.is_empty() {
            pkt.out_iface = t.out_iface_for(&pkt.dst_ip);
        }
    }

    if let Some(t) = topo {
        let is_local_src = t.is_local(&pkt.src_ip);
        if is_local_src {
            eval_local_out(ruleset, topo, pkt, &mut trace)
        } else {
            eval_forward(ruleset, topo, pkt, &mut trace)
        }
    } else {
        // Modo simple sin topología: sin información de routing, asumimos INPUT
        // (usado en tests unitarios del pipeline).
        let (v, _) = eval_chain(ruleset, topo, &pkt, "filter", "INPUT", 0, &mut trace);
        (to_final(v), trace)
    }
}

/// Tráfico generado localmente: filter/OUTPUT → nat/POSTROUTING
fn eval_local_out(
    ruleset: &Ruleset,
    topo: Option<&Topology>,
    pkt: Packet,
    trace: &mut Trace,
) -> (FinalVerdict, Trace) {
    let (v, pkt) = eval_chain(ruleset, topo, &pkt, "filter", "OUTPUT", 0, trace);
    if v != InternalVerdict::Accept {
        return (to_final(v), trace.clone());
    }
    let (v2, _) = eval_chain(ruleset, topo, &pkt, "nat", "POSTROUTING", 0, trace);
    (to_final(v2), trace.clone())
}

/// Tráfico entrante: nat/PREROUTING → routing → filter/INPUT o FORWARD → nat/POSTROUTING
fn eval_forward(
    ruleset: &Ruleset,
    topo: Option<&Topology>,
    pkt: Packet,
    trace: &mut Trace,
) -> (FinalVerdict, Trace) {
    // 1. nat/PREROUTING (puede DNAT)
    let (v, mut pkt) = eval_chain(ruleset, topo, &pkt, "nat", "PREROUTING", 0, trace);
    if v == InternalVerdict::Drop || v == InternalVerdict::Reject {
        return (to_final(v), trace.clone());
    }

    // 2. Decisión de routing tras posible DNAT
    let filter_chain = if let Some(t) = topo {
        // Actualizar out_iface por si DNAT cambió el DstIP
        pkt.out_iface = t.out_iface_for(&pkt.dst_ip);
        if t.is_local(&pkt.dst_ip) {
            "INPUT"
        } else {
            "FORWARD"
        }
    } else {
        "FORWARD"
    };

    // 3. filter/INPUT o filter/FORWARD
    let (v, pkt) = eval_chain(ruleset, topo, &pkt, "filter", filter_chain, 0, trace);
    if v != InternalVerdict::Accept {
        return (to_final(v), trace.clone());
    }

    // 4. nat/POSTROUTING solo para FORWARD
    if filter_chain != "FORWARD" {
        return (FinalVerdict::Accept, trace.clone());
    }
    let (v2, _) = eval_chain(ruleset, topo, &pkt, "nat", "POSTROUTING", 0, trace);
    (to_final(v2), trace.clone())
}

/// Evalúa el paquete contra una cadena del ruleset.
/// Devuelve (veredicto_interno, paquete_posiblemente_modificado).
fn eval_chain(
    ruleset: &Ruleset,
    topo: Option<&Topology>,
    pkt: &Packet,
    table: &str,
    chain: &str,
    depth: u32,
    trace: &mut Trace,
) -> (InternalVerdict, Packet) {
    if depth > MAX_DEPTH {
        return (InternalVerdict::Drop, pkt.clone());
    }

    let (policy, rules) = get_chain_data(ruleset, table, chain);
    let mut current_pkt = pkt.clone();

    for (i, rule) in rules.iter().enumerate() {
        // Extraer matches y target del comando (solo Append/Insert contienen reglas activas)
        let (matches, target) = match &rule.command {
            RuleCommand::Append {
                matches, target, ..
            }
            | RuleCommand::Insert {
                matches, target, ..
            } => (matches, target),
            _ => continue,
        };

        // Comprobar todos los matchers
        if !matches.iter().all(|m| matches_packet(m, &current_pkt)) {
            continue;
        }

        // Aplicar el target
        let (verdict, new_pkt) = apply_target(target, current_pkt.clone(), topo);
        trace.add(table, chain, i as i32, verdict.as_str(), new_pkt.clone());

        match verdict {
            InternalVerdict::Accept => return (InternalVerdict::Accept, new_pkt),
            InternalVerdict::Drop => return (InternalVerdict::Drop, new_pkt),
            InternalVerdict::Reject => return (InternalVerdict::Reject, new_pkt),
            InternalVerdict::Return => return (InternalVerdict::Return, new_pkt),
            InternalVerdict::Jump(ref chain_name) => {
                let target_chain = chain_name.clone();
                let (sub_v, sub_pkt) = eval_chain(
                    ruleset,
                    topo,
                    &new_pkt,
                    table,
                    &target_chain,
                    depth + 1,
                    trace,
                );
                match sub_v {
                    // RETURN desde cadena de usuario → continuar en la cadena padre
                    InternalVerdict::Return | InternalVerdict::Continue => {
                        current_pkt = sub_pkt;
                        continue;
                    }
                    v => return (v, sub_pkt),
                }
            }
            // LOG y similares: no terminan, continuar con pkt posiblemente modificado
            InternalVerdict::Continue => {
                current_pkt = new_pkt;
            }
        }
    }

    // Ninguna regla coincidió → política por defecto
    if let Some(pol) = policy {
        let (v, new_pkt) = apply_target(pol, current_pkt.clone(), topo);
        trace.add(table, chain, -1, v.as_str(), new_pkt.clone());
        return (v, new_pkt);
    }

    // Cadena de usuario sin match → RETURN implícito
    (InternalVerdict::Return, current_pkt)
}

/// Extrae la política y las reglas activas de una cadena del ruleset.
fn get_chain_data<'a>(
    ruleset: &'a Ruleset,
    table: &str,
    chain: &str,
) -> (Option<&'a Target>, &'a [crate::engine::ast::ParsedRule]) {
    let cs = match (table, chain) {
        ("filter", "INPUT") => Some(&ruleset.filter_input),
        ("filter", "OUTPUT") => Some(&ruleset.filter_output),
        ("filter", "FORWARD") => Some(&ruleset.filter_forward),
        ("nat", "PREROUTING") => Some(&ruleset.nat_prerouting),
        ("nat", "POSTROUTING") => Some(&ruleset.nat_postrouting),
        _ => None,
    };
    match cs {
        Some(c) => (c.policy.as_ref(), &c.rules),
        None => (None, &[]),
    }
}

/// Aplica un target AST a un paquete y devuelve (veredicto_interno, paquete_nuevo).
fn apply_target(
    target: &Target,
    mut pkt: Packet,
    topo: Option<&Topology>,
) -> (InternalVerdict, Packet) {
    match target {
        Target::Accept => (InternalVerdict::Accept, pkt),
        Target::Drop => (InternalVerdict::Drop, pkt),
        Target::Reject { .. } => (InternalVerdict::Reject, pkt),
        Target::Return => (InternalVerdict::Return, pkt),
        Target::Jump(chain) => (InternalVerdict::Jump(chain.clone()), pkt),
        Target::Log { .. } => (InternalVerdict::Continue, pkt),
        Target::Masquerade => {
            // Reescribir SrcIP con la IP del firewall en la interfaz de salida
            if let Some(t) = topo {
                if let Some(ip) = t.interface_ip(&pkt.out_iface) {
                    pkt.src_ip = ip.to_string();
                }
            }
            (InternalVerdict::Accept, pkt)
        }
        Target::Snat { to } => {
            // Formato: "ip" o "ip:port"
            let parts: Vec<&str> = to.splitn(2, ':').collect();
            pkt.src_ip = parts[0].to_string();
            (InternalVerdict::Accept, pkt)
        }
        Target::Dnat { to } => {
            // Formato: "ip:port" o solo "ip"
            let parts: Vec<&str> = to.splitn(2, ':').collect();
            pkt.dst_ip = parts[0].to_string();
            if parts.len() > 1 {
                if let Ok(port) = parts[1].parse::<u16>() {
                    pkt.dst_port = port;
                }
            }
            (InternalVerdict::Accept, pkt)
        }
    }
}

/// Convierte un veredicto interno a veredicto final (DROP como default seguro).
fn to_final(v: InternalVerdict) -> FinalVerdict {
    match v {
        InternalVerdict::Accept | InternalVerdict::Return | InternalVerdict::Continue => {
            FinalVerdict::Accept
        }
        InternalVerdict::Reject => FinalVerdict::Reject,
        _ => FinalVerdict::Drop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::packet::{Proto, STATE_NEW};
    use crate::engine::ruleset::Ruleset;

    fn packet_tcp_80() -> Packet {
        Packet {
            src_ip: "192.168.1.10".into(),
            dst_ip: "192.168.1.1".into(),
            src_port: 54321,
            dst_port: 80,
            proto: Proto::Tcp,
            in_iface: "eth0".into(),
            out_iface: String::new(),
            state: STATE_NEW,
        }
    }

    #[test]
    fn test_accept_policy() {
        let rs = Ruleset::new(); // políticas ACCEPT por defecto
        let (v, _) = evaluate(&rs, None, packet_tcp_80());
        assert_eq!(v, FinalVerdict::Accept);
    }

    #[test]
    fn test_drop_policy() {
        use crate::engine::parser::parse_line;
        let mut rs = Ruleset::new();
        let cmd = parse_line("-P INPUT DROP").unwrap();
        rs.apply(&cmd).unwrap();
        let (v, _trace) = evaluate(&rs, None, packet_tcp_80());
        assert_eq!(v, FinalVerdict::Drop);
    }

    #[test]
    fn test_append_accept_then_drop_policy() {
        use crate::engine::parser::parse_line;
        let mut rs = Ruleset::new();
        rs.apply(&parse_line("-P INPUT DROP").unwrap()).unwrap();
        rs.apply(&parse_line("-A INPUT -p tcp --dport 80 -j ACCEPT").unwrap())
            .unwrap();
        let (v, _) = evaluate(&rs, None, packet_tcp_80());
        assert_eq!(v, FinalVerdict::Accept);
        // Puerto 443 debería caer
        let mut pkt443 = packet_tcp_80();
        pkt443.dst_port = 443;
        let (v2, _) = evaluate(&rs, None, pkt443);
        assert_eq!(v2, FinalVerdict::Drop);
    }
}
