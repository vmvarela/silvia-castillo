use serde::{Deserialize, Serialize};
use ts_rs::TS;
use crate::engine::parser::parse_line;
use crate::engine::ast::ParsedRule;
use crate::engine::ruleset::RulesetView;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExecuteResult {
    pub ok: bool,
    pub ast: Option<ParsedRule>,
    pub error: Option<String>,
    pub humanize: Option<String>,
    pub ruleset: RulesetView,
}

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

#[tauri::command]
pub fn get_ruleset(state: tauri::State<'_, AppState>) -> Result<RulesetView, String> {
    let ruleset = state.ruleset.lock().map_err(|e| e.to_string())?;
    Ok(RulesetView::from(&*ruleset))
}

#[tauri::command]
pub fn reset_ruleset(state: tauri::State<'_, AppState>) -> Result<RulesetView, String> {
    let mut ruleset = state.ruleset.lock().map_err(|e| e.to_string())?;
    *ruleset = crate::engine::ruleset::Ruleset::new();
    Ok(RulesetView::from(&*ruleset))
}

/// Stub humanizer: genera una descripción en español del comando
fn humanize_rule(rule: &ParsedRule) -> String {
    use crate::engine::ast::RuleCommand;
    match &rule.command {
        RuleCommand::Append { chain, matches, target } => {
            let chain_name = format!("{:?}", chain).to_uppercase();
            if matches.is_empty() {
                format!("Añadir a {}: {} todos los paquetes", chain_name, target_str(target))
            } else {
                format!(
                    "Añadir a {}: {} paquetes que cumplan {} condición/es",
                    chain_name,
                    target_str(target),
                    matches.len()
                )
            }
        }
        RuleCommand::Flush { chain: None } => "Vaciar todas las cadenas".to_string(),
        RuleCommand::Flush { chain: Some(c) } => {
            format!("Vaciar cadena {:?}", c).to_uppercase()
        }
        RuleCommand::Policy { chain, target } => {
            format!(
                "Política por defecto de {:?}: {}",
                chain,
                target_str(target)
            )
            .to_uppercase()
        }
        RuleCommand::Delete { chain, rule_num } => {
            if let Some(n) = rule_num {
                format!("Eliminar regla {} de {:?}", n, chain)
            } else {
                format!("Eliminar regla de {:?}", chain)
            }
        }
        _ => format!("Comando ejecutado: {:?}", rule.command)
            .chars()
            .take(80)
            .collect(),
    }
}

fn target_str(target: &crate::engine::ast::Target) -> &str {
    use crate::engine::ast::Target;
    match target {
        Target::Accept => "ACEPTAR",
        Target::Drop => "DESCARTAR",
        Target::Reject { .. } => "RECHAZAR",
        Target::Log { .. } => "REGISTRAR",
        Target::Masquerade => "ENMASCARAR",
        Target::Snat { .. } => "SNAT",
        Target::Dnat { .. } => "DNAT",
        Target::Return => "RETORNAR",
        Target::Jump(s) => s.as_str(),
    }
}
