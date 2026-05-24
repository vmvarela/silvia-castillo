use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Protocolo de red de un paquete.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Proto {
    #[default]
    All,
    Tcp,
    Udp,
    Icmp,
}

impl Proto {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "tcp" => Proto::Tcp,
            "udp" => Proto::Udp,
            "icmp" => Proto::Icmp,
            _ => Proto::All,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Proto::Tcp => "tcp",
            Proto::Udp => "udp",
            Proto::Icmp => "icmp",
            Proto::All => "all",
        }
    }
}

// ConnState bitmask constants (espejo del Go engine)
pub const STATE_NEW: u8 = 1 << 0;
pub const STATE_ESTABLISHED: u8 = 1 << 1;
pub const STATE_RELATED: u8 = 1 << 2;
pub const STATE_INVALID: u8 = 1 << 3;
pub const STATE_UNTRACKED: u8 = 1 << 4;

/// Convierte un string de estado conntrack (NEW, ESTABLISHED, …) a su bit.
pub fn parse_state_bit(s: &str) -> u8 {
    match s.trim().to_uppercase().as_str() {
        "NEW" => STATE_NEW,
        "ESTABLISHED" => STATE_ESTABLISHED,
        "RELATED" => STATE_RELATED,
        "INVALID" => STATE_INVALID,
        "UNTRACKED" => STATE_UNTRACKED,
        _ => 0,
    }
}

/// Paquete de red que atraviesa el firewall simulado.
/// Las IPs se almacenan como strings para facilitar la serialización.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Packet {
    pub src_ip: String,
    pub dst_ip: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub proto: Proto,
    pub in_iface: String,
    pub out_iface: String,
    /// Máscara de bits ConnState (STATE_NEW, STATE_ESTABLISHED, …)
    pub state: u8,
}

impl Packet {
    pub fn with_src_ip(mut self, ip: impl Into<String>) -> Self {
        self.src_ip = ip.into();
        self
    }
    pub fn with_dst_ip(mut self, ip: impl Into<String>) -> Self {
        self.dst_ip = ip.into();
        self
    }
    pub fn with_dst_port(mut self, port: u16) -> Self {
        self.dst_port = port;
        self
    }
    pub fn with_in_iface(mut self, iface: impl Into<String>) -> Self {
        self.in_iface = iface.into();
        self
    }
    pub fn with_out_iface(mut self, iface: impl Into<String>) -> Self {
        self.out_iface = iface.into();
        self
    }
}
