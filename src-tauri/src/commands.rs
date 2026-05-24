use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::engine::ast::ParsedRule;
use crate::engine::packet::{parse_state_bit, Packet, Proto};
use crate::engine::parser::parse_line;
use crate::engine::pipeline::evaluate;
use crate::engine::ruleset::{Ruleset, RulesetView};
use crate::engine::topology::topology_from_level;
use crate::levels;
use crate::progress::Progress;
use crate::state::AppState;

// ─── tipos exportados al frontend ────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExecuteResult {
    pub ok: bool,
    pub ast: Option<ParsedRule>,
    pub error: Option<String>,
    pub humanize: Option<String>,
    pub ruleset: RulesetView,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LevelInfo {
    pub index: usize,
    pub id: String,
    pub titulo: String,
    pub locked: bool,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HostView {
    pub nombre: String,
    pub ip: String,
    pub zona: String,
    pub iface: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LevelView {
    pub index: usize,
    pub id: String,
    pub titulo: String,
    pub cuento: String,
    pub mision: String,
    pub pistas: Vec<String>,
    pub recompensa: String,
    pub hosts: Vec<HostView>,
    pub ruleset: RulesetView,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TestResult {
    pub index: usize,
    pub descripcion: String,
    pub src_ip: String,
    pub dst_ip: String,
    pub dst_port: u16,
    pub proto: String,
    pub estado: String,
    pub esperado: String,
    pub got: String,
    pub passed: bool,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CheckResult {
    pub results: Vec<TestResult>,
    pub all_passed: bool,
    /// Porcentaje de tests pasados (0.0 – 1.0).
    pub score: f32,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProgressView {
    pub unlocked_until: usize,
    pub completed: Vec<String>,
}

impl From<&Progress> for ProgressView {
    fn from(p: &Progress) -> Self {
        ProgressView {
            unlocked_until: p.unlocked_until,
            completed: p.completed.clone(),
        }
    }
}

// ─── comandos Tauri ───────────────────────────────────────────────────────────

/// Ejecuta una línea de comando iptables en el ruleset actual.
#[tauri::command]
pub fn execute_command(
    input: String,
    state: tauri::State<'_, AppState>,
) -> Result<ExecuteResult, String> {
    let mut ruleset = state.ruleset.lock().map_err(|e| e.to_string())?;

    match parse_line(&input) {
        Ok(rule) => {
            let humanize = humanize_rule(&rule);
            match ruleset.apply(&rule) {
                Ok(()) => {
                    let view = RulesetView::from(&*ruleset);
                    Ok(ExecuteResult {
                        ok: true,
                        ast: Some(rule),
                        error: None,
                        humanize: Some(humanize),
                        ruleset: view,
                    })
                }
                Err(e) => {
                    let view = RulesetView::from(&*ruleset);
                    Ok(ExecuteResult {
                        ok: false,
                        ast: None,
                        error: Some(e),
                        humanize: None,
                        ruleset: view,
                    })
                }
            }
        }
        Err(e) => {
            let view = RulesetView::from(&*ruleset);
            Ok(ExecuteResult {
                ok: false,
                ast: None,
                error: Some(e),
                humanize: None,
                ruleset: view,
            })
        }
    }
}

/// Devuelve la vista actual del ruleset sin modificarlo.
#[tauri::command]
pub fn get_ruleset(state: tauri::State<'_, AppState>) -> Result<RulesetView, String> {
    let ruleset = state.ruleset.lock().map_err(|e| e.to_string())?;
    Ok(RulesetView::from(&*ruleset))
}

/// Reinicia el ruleset al estado inicial (políticas ACCEPT, sin reglas).
#[tauri::command]
pub fn reset_ruleset(state: tauri::State<'_, AppState>) -> Result<RulesetView, String> {
    let mut ruleset = state.ruleset.lock().map_err(|e| e.to_string())?;
    *ruleset = Ruleset::new();
    Ok(RulesetView::from(&*ruleset))
}

/// Devuelve la lista de niveles con su estado de bloqueo.
#[tauri::command]
pub fn get_level_list(state: tauri::State<'_, AppState>) -> Result<Vec<LevelInfo>, String> {
    let progress = state.progress.lock().map_err(|e| e.to_string())?;
    let all = levels::load_all();
    Ok(all
        .into_iter()
        .enumerate()
        .map(|(i, l)| LevelInfo {
            index: i,
            id: l.id,
            titulo: l.titulo,
            locked: !progress.is_unlocked(i),
        })
        .collect())
}

/// Carga un nivel por índice 0-based. Aplica reglas iniciales y configura topología.
#[tauri::command]
pub fn load_level(index: usize, state: tauri::State<'_, AppState>) -> Result<LevelView, String> {
    let level = levels::load_level(index).ok_or_else(|| format!("Nivel {index} no encontrado"))?;

    // Verificar que el nivel está desbloqueado
    {
        let progress = state.progress.lock().map_err(|e| e.to_string())?;
        if !progress.is_unlocked(index) {
            return Err(format!("El nivel {index} está bloqueado"));
        }
    }

    // Construir topología
    let topo = topology_from_level(
        &level.red.firewall_ip,
        &level.red.interfaces,
        &level.red.hosts,
    );

    // Reiniciar ruleset y aplicar políticas iniciales
    let mut ruleset = state.ruleset.lock().map_err(|e| e.to_string())?;
    *ruleset = Ruleset::new();

    // Aplicar políticas del nivel
    for (chain, policy) in &level.politicas {
        let cmd_str = format!("-P {chain} {policy}");
        if let Ok(cmd) = parse_line(&cmd_str) {
            let _ = ruleset.apply(&cmd);
        }
    }

    // Aplicar reglas iniciales
    for rule_str in &level.reglas_iniciales {
        if let Ok(cmd) = parse_line(rule_str) {
            let _ = ruleset.apply(&cmd);
        }
    }

    let ruleset_view = RulesetView::from(&*ruleset);

    // Guardar nivel y topología en el estado
    *state.current_level.lock().map_err(|e| e.to_string())? = Some(level.clone());
    *state.current_level_idx.lock().map_err(|e| e.to_string())? = Some(index);
    *state.topology.lock().map_err(|e| e.to_string())? = Some(topo);

    let hosts = level
        .red
        .hosts
        .iter()
        .map(|h| HostView {
            nombre: h.nombre.clone(),
            ip: h.ip.clone(),
            zona: h.zona.clone(),
            iface: h.iface.clone(),
        })
        .collect();

    Ok(LevelView {
        index,
        id: level.id,
        titulo: level.titulo,
        cuento: level.cuento,
        mision: level.mision,
        pistas: level.pistas,
        recompensa: level.recompensa,
        hosts,
        ruleset: ruleset_view,
    })
}

/// Ejecuta todas las pruebas del nivel actual contra el ruleset actual.
#[tauri::command]
pub fn check_tests(state: tauri::State<'_, AppState>) -> Result<CheckResult, String> {
    let level_guard = state.current_level.lock().map_err(|e| e.to_string())?;
    let level = level_guard.as_ref().ok_or("No hay ningún nivel cargado")?;
    let topo_guard = state.topology.lock().map_err(|e| e.to_string())?;
    let topo = topo_guard.as_ref();
    let ruleset = state.ruleset.lock().map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    let mut passed_count = 0usize;

    for (i, prueba) in level.pruebas.iter().enumerate() {
        let pkt = Packet {
            src_ip: prueba.src_ip.clone(),
            dst_ip: prueba.dst_ip.clone(),
            src_port: 0,
            dst_port: prueba.dst_port,
            proto: Proto::from_str(&prueba.proto),
            in_iface: String::new(),
            out_iface: String::new(),
            state: parse_state_bit(&prueba.estado),
        };

        let (verdict, _trace) = evaluate(&ruleset, topo, pkt);
        let got = verdict.as_str().to_string();
        let passed = got == prueba.esperado.to_uppercase();
        if passed {
            passed_count += 1;
        }

        results.push(TestResult {
            index: i,
            descripcion: prueba.descripcion.clone(),
            src_ip: prueba.src_ip.clone(),
            dst_ip: prueba.dst_ip.clone(),
            dst_port: prueba.dst_port,
            proto: prueba.proto.clone(),
            estado: prueba.estado.clone(),
            esperado: prueba.esperado.clone(),
            got,
            passed,
        });
    }

    let total = results.len();
    let all_passed = total > 0 && passed_count == total;
    let score = if total > 0 {
        passed_count as f32 / total as f32
    } else {
        0.0
    };

    Ok(CheckResult {
        results,
        all_passed,
        score,
    })
}

/// Marca el nivel actual como completado y desbloquea el siguiente.
#[tauri::command]
pub fn mark_level_complete(state: tauri::State<'_, AppState>) -> Result<ProgressView, String> {
    let level_guard = state.current_level.lock().map_err(|e| e.to_string())?;
    let level = level_guard.as_ref().ok_or("No hay ningún nivel cargado")?;
    let level_id = level.id.clone();
    drop(level_guard);

    let idx = state
        .current_level_idx
        .lock()
        .map_err(|e| e.to_string())?
        .ok_or("No hay índice de nivel")?;

    let total = levels::level_count();
    let mut progress = state.progress.lock().map_err(|e| e.to_string())?;
    progress.complete_level(&level_id, idx, total);
    state.store.save(&progress).map_err(|e| e.to_string())?;

    Ok(ProgressView::from(&*progress))
}

/// Devuelve el progreso actual del jugador.
#[tauri::command]
pub fn get_progress(state: tauri::State<'_, AppState>) -> Result<ProgressView, String> {
    let progress = state.progress.lock().map_err(|e| e.to_string())?;
    Ok(ProgressView::from(&*progress))
}

// ─── humanizer ───────────────────────────────────────────────────────────────

fn humanize_rule(rule: &ParsedRule) -> String {
    use crate::engine::ast::RuleCommand;
    match &rule.command {
        RuleCommand::Append {
            chain,
            matches,
            target,
        }
        | RuleCommand::Insert {
            chain,
            matches,
            target,
            ..
        } => {
            let chain_name = format!("{chain:?}").to_uppercase();
            let target_desc = humanize_target(target);
            if matches.is_empty() {
                format!("En {chain_name}: {target_desc} todos los paquetes.")
            } else {
                let conditions = matches
                    .iter()
                    .filter_map(humanize_match)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("En {chain_name}: si llega {conditions}, {target_desc}.")
            }
        }
        RuleCommand::Flush { chain: None } => "Vaciar todas las cadenas.".into(),
        RuleCommand::Flush { chain: Some(c) } => format!("Vaciar cadena {:?}.", c).to_uppercase(),
        RuleCommand::Policy { chain, target } => {
            format!(
                "Política por defecto de {:?}: {}.",
                chain,
                humanize_target(target)
            )
        }
        RuleCommand::Delete { chain, rule_num } => {
            if let Some(n) = rule_num {
                format!("Eliminar regla {n} de {chain:?}.")
            } else {
                format!("Eliminar regla de {chain:?}.")
            }
        }
        _ => format!("{:?}", rule.command).chars().take(80).collect(),
    }
}

fn humanize_target(t: &crate::engine::ast::Target) -> &str {
    use crate::engine::ast::Target;
    match t {
        Target::Accept => "¡déjalo pasar!",
        Target::Drop => "¡detenlo en silencio!",
        Target::Reject { .. } => "¡recházalo con aviso!",
        Target::Log { .. } => "anotarlo en el libro",
        Target::Masquerade => "enmascarar origen",
        Target::Snat { .. } => "cambiar IP origen",
        Target::Dnat { .. } => "redirigir destino",
        Target::Return => "devolver al guardia anterior",
        Target::Jump(c) => c.as_str(),
    }
}

fn humanize_match(m: &crate::engine::ast::Match) -> Option<String> {
    use crate::engine::ast::MatchKind;
    let neg = if m.negated { "NO " } else { "" };
    let desc = match &m.kind {
        MatchKind::Protocol(p) => format!("protocolo {p}"),
        MatchKind::Source(s) => format!("desde {s}"),
        MatchKind::Destination(d) => format!("hacia {d}"),
        MatchKind::DPort(ps) => {
            if ps.start == ps.end {
                format!("puerto destino {}", ps.start)
            } else {
                format!("puerto destino {}-{}", ps.start, ps.end)
            }
        }
        MatchKind::SPort(ps) => {
            if ps.start == ps.end {
                format!("puerto origen {}", ps.start)
            } else {
                format!("puerto origen {}-{}", ps.start, ps.end)
            }
        }
        MatchKind::InInterface(i) => format!("entrando por {i}"),
        MatchKind::OutInterface(i) => format!("saliendo por {i}"),
        MatchKind::State(s) | MatchKind::CtState(s) => {
            format!("estado {}", s.join(","))
        }
        MatchKind::IcmpType(t) => format!("tipo ICMP {t}"),
        MatchKind::Comment(_) => return None,
    };
    Some(format!("{neg}{desc}"))
}
