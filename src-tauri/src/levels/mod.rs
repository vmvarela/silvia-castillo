use std::collections::HashMap;

use serde::Deserialize;

/// Interfaz de red de un nivel (campos en español, como el YAML).
#[derive(Debug, Clone, Deserialize)]
pub struct LevelInterface {
    pub nombre: String,
    pub zona: String,
    pub cidr: String,
    /// IP propia del firewall en esta interfaz (puede diferir del firewall_ip principal).
    #[serde(default)]
    pub ip: Option<String>,
}

/// Host virtual en la topología de un nivel.
#[derive(Debug, Clone, Deserialize)]
pub struct LevelHost {
    pub nombre: String,
    pub ip: String,
    pub zona: String,
    pub iface: String,
}

/// Bloque `red:` de un nivel.
#[derive(Debug, Clone, Deserialize)]
pub struct LevelNetwork {
    pub firewall_ip: String,
    #[serde(default)]
    pub interfaces: Vec<LevelInterface>,
    #[serde(default)]
    pub hosts: Vec<LevelHost>,
}

/// Una prueba de evaluación del nivel.
#[derive(Debug, Clone, Deserialize)]
pub struct LevelTest {
    #[serde(default)]
    pub descripcion: String,
    pub src_ip: String,
    pub dst_ip: String,
    pub dst_port: u16,
    pub proto: String,
    pub estado: String,
    /// "ACCEPT" | "DROP"
    pub esperado: String,
}

/// Un nivel completo (mapeado desde YAML).
#[derive(Debug, Clone, Deserialize)]
pub struct Level {
    pub id: String,
    pub titulo: String,
    pub cuento: String,
    pub mision: String,
    pub red: LevelNetwork,
    /// Políticas por defecto: {"INPUT": "ACCEPT", "OUTPUT": "ACCEPT", "FORWARD": "ACCEPT"}
    #[serde(default)]
    pub politicas: HashMap<String, String>,
    /// Comandos iptables iniciales (se aplican al cargar el nivel).
    #[serde(default)]
    pub reglas_iniciales: Vec<String>,
    #[serde(default)]
    pub pistas: Vec<String>,
    #[serde(default)]
    pub pruebas: Vec<LevelTest>,
    #[serde(default)]
    pub recompensa: String,
}

// Los 9 YAML incrustados en el binario
const LEVEL_YAMLS: &[&str] = &[
    include_str!("data/01-observar.yaml"),
    include_str!("data/02-primer-guardia.yaml"),
    include_str!("data/03-bloquear-tipo.yaml"),
    include_str!("data/04-politica-drop.yaml"),
    include_str!("data/05-tres-portones.yaml"),
    include_str!("data/06-barbacana.yaml"),
    include_str!("data/07-estandarte.yaml"),
    include_str!("data/08-puerta-secreta.yaml"),
    include_str!("data/09-examen-silvia.yaml"),
];

/// Carga todos los niveles disponibles. Falla silenciosamente (omite YAMLs inválidos).
pub fn load_all() -> Vec<Level> {
    LEVEL_YAMLS
        .iter()
        .filter_map(|yaml| serde_yml::from_str(yaml).ok())
        .collect()
}

/// Carga el nivel en la posición `index` (0-based).
/// Devuelve None si el índice está fuera de rango o el YAML es inválido.
pub fn load_level(index: usize) -> Option<Level> {
    LEVEL_YAMLS
        .get(index)
        .and_then(|yaml| serde_yml::from_str(yaml).ok())
}

/// Número total de niveles.
pub fn level_count() -> usize {
    LEVEL_YAMLS.len()
}
