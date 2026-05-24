use std::net::IpAddr;

use ipnet::IpNet;

/// Una interfaz de red del firewall simulado.
pub struct Interface {
    pub name: String,
    pub zone: String,
    /// CIDR asignado a esta interfaz (puede ser 0.0.0.0/0 para "mundo").
    pub cidr: Option<IpNet>,
    /// IP del propio firewall en esta interfaz (usada en MASQUERADE).
    pub ip: Option<IpAddr>,
}

/// Un host en la topología simulada.
pub struct Host {
    pub name: String,
    pub ip: IpAddr,
    pub zone: String,
    /// Nombre de la interfaz del firewall por la que llega/sale el tráfico.
    pub iface: String,
}

/// Topología de red del firewall simulado.
pub struct Topology {
    /// IPs que pertenecen al propio firewall (el castillo).
    pub local_ips: Vec<IpAddr>,
    pub interfaces: Vec<Interface>,
    pub hosts: Vec<Host>,
}

impl Topology {
    /// Devuelve true si la IP dada pertenece al propio firewall.
    pub fn is_local(&self, ip_str: &str) -> bool {
        if let Ok(ip) = ip_str.parse::<IpAddr>() {
            return self.local_ips.contains(&ip);
        }
        false
    }

    /// Interfaz de entrada para un paquete cuyo origen es `src_ip`.
    pub fn in_iface_for(&self, src_ip: &str) -> String {
        if self.is_local(src_ip) {
            return "lo".to_string();
        }
        self.iface_for_ip(src_ip).unwrap_or_default()
    }

    /// Interfaz de salida para un paquete cuyo destino es `dst_ip`.
    pub fn out_iface_for(&self, dst_ip: &str) -> String {
        if self.is_local(dst_ip) {
            return "lo".to_string();
        }
        self.iface_for_ip(dst_ip).unwrap_or_default()
    }

    /// Encuentra la interfaz cuyo CIDR contiene la IP.
    /// Las subredes concretas tienen prioridad sobre 0.0.0.0/0.
    fn iface_for_ip(&self, ip_str: &str) -> Option<String> {
        let ip: IpAddr = ip_str.parse().ok()?;
        let mut default_iface: Option<String> = None;
        for iface in &self.interfaces {
            if let Some(cidr) = &iface.cidr {
                if cidr.prefix_len() == 0 {
                    default_iface = Some(iface.name.clone());
                } else if cidr.contains(&ip) {
                    return Some(iface.name.clone());
                }
            }
        }
        default_iface
    }

    /// IP del firewall en la interfaz con el nombre dado.
    /// Devuelve la primera local_ip como fallback.
    pub fn interface_ip(&self, name: &str) -> Option<IpAddr> {
        for iface in &self.interfaces {
            if iface.name == name {
                if let Some(ip) = iface.ip {
                    return Some(ip);
                }
            }
        }
        self.local_ips.first().copied()
    }
}

/// Construye una Topology mínima desde la descripción YAML de un nivel.
/// Los campos YAML usan nombres en español (nombre, cidr, zona, iface).
pub fn topology_from_level(
    firewall_ip: &str,
    interfaces: &[crate::levels::LevelInterface],
    hosts: &[crate::levels::LevelHost],
) -> Topology {
    let fw_ip: IpAddr = firewall_ip
        .parse()
        .unwrap_or(IpAddr::from([192, 168, 1, 1]));

    let mut local_ips: Vec<IpAddr> = vec![fw_ip];

    let ifaces: Vec<Interface> = interfaces
        .iter()
        .map(|i| {
            // Usar la IP propia de la interfaz si se especifica en el YAML;
            // si no, usar la IP principal del firewall.
            let iface_ip: Option<IpAddr> = i
                .ip
                .as_deref()
                .and_then(|s| s.parse().ok())
                .or(Some(fw_ip));

            // Registrar la IP como local (para que `is_local` enrute al INPUT
            // los paquetes dirigidos a cualquier interfaz del firewall).
            if let Some(ip) = iface_ip {
                if !local_ips.contains(&ip) {
                    local_ips.push(ip);
                }
            }

            Interface {
                name: i.nombre.clone(),
                zone: i.zona.clone(),
                cidr: i.cidr.parse().ok(),
                ip: iface_ip,
            }
        })
        .collect();

    let hs: Vec<Host> = hosts
        .iter()
        .filter_map(|h| {
            let ip = h.ip.parse().ok()?;
            Some(Host {
                name: h.nombre.clone(),
                ip,
                zone: h.zona.clone(),
                iface: h.iface.clone(),
            })
        })
        .collect();

    Topology {
        local_ips,
        interfaces: ifaces,
        hosts: hs,
    }
}
