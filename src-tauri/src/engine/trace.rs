use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::engine::packet::Packet;

/// Un paso de la evaluación de un paquete a través del pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TraceStep {
    pub table: String,
    pub chain: String,
    /// Índice de la regla (0-based); -1 = política por defecto.
    pub rule_idx: i32,
    /// Veredicto aplicado: "ACCEPT", "DROP", "REJECT", "RETURN", "JUMP", "CONTINUE"
    pub verdict: String,
    /// Estado del paquete en este paso (puede variar tras DNAT/SNAT).
    pub pkt: Packet,
}

/// Registro completo de la evaluación de un paquete a través del pipeline.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Trace {
    pub steps: Vec<TraceStep>,
}

impl Trace {
    pub fn add(&mut self, table: &str, chain: &str, rule_idx: i32, verdict: &str, pkt: Packet) {
        self.steps.push(TraceStep {
            table: table.to_string(),
            chain: chain.to_string(),
            rule_idx,
            verdict: verdict.to_string(),
            pkt,
        });
    }

    /// Devuelve el estado del paquete en el último paso registrado.
    #[allow(dead_code)]
    pub fn last_pkt(&self) -> Option<&Packet> {
        self.steps.last().map(|s| &s.pkt)
    }
}
