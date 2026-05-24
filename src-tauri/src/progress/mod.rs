use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Progreso del jugador.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Progress {
    /// Índice 0-based del último nivel desbloqueado.
    /// 0 = solo el primer nivel disponible.
    pub unlocked_until: usize,
    /// IDs de los niveles completados.
    pub completed: Vec<String>,
}

impl Progress {
    /// Devuelve true si el nivel en `index` está desbloqueado.
    pub fn is_unlocked(&self, index: usize) -> bool {
        index <= self.unlocked_until
    }

    /// Marca el nivel como completado y desbloquea el siguiente.
    pub fn complete_level(&mut self, level_id: &str, level_index: usize, total: usize) {
        if !self.completed.contains(&level_id.to_string()) {
            self.completed.push(level_id.to_string());
        }
        if level_index >= self.unlocked_until && level_index + 1 < total {
            self.unlocked_until = level_index + 1;
        }
    }
}

/// Almacén de progreso en disco (JSON atómico).
pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    /// Crea un FileStore en el directorio home del usuario.
    /// Ruta: `~/.silvia-castillo.json`
    pub fn new() -> Result<Self, String> {
        let path = dirs::home_dir()
            .ok_or_else(|| "No se pudo determinar el directorio home".to_string())?
            .join(".silvia-castillo.json");
        Ok(FileStore { path })
    }

    /// Crea un FileStore en una ruta personalizada (útil para tests).
    pub fn at(path: PathBuf) -> Self {
        FileStore { path }
    }

    /// Carga el progreso desde disco. Devuelve un progreso vacío si el fichero
    /// no existe o no se puede parsear.
    pub fn load(&self) -> Progress {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Guarda el progreso en disco de forma atómica (tmp + rename).
    pub fn save(&self, p: &Progress) -> Result<(), String> {
        let data =
            serde_json::to_string_pretty(p).map_err(|e| format!("Error serializando progreso: {e}"))?;
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, data)
            .map_err(|e| format!("Error escribiendo fichero temporal: {e}"))?;
        std::fs::rename(&tmp, &self.path)
            .map_err(|e| format!("Error renombrando fichero de progreso: {e}"))?;
        Ok(())
    }
}
