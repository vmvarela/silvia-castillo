use crate::engine::ast::{Chain, ParsedRule, RuleCommand, Table, Target};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainState {
    pub policy: Option<Target>,
    pub rules: Vec<ParsedRule>,
}

impl ChainState {
    pub fn new_accept() -> Self {
        ChainState {
            policy: Some(Target::Accept),
            rules: vec![],
        }
    }
    pub fn new_no_policy() -> Self {
        ChainState {
            policy: None,
            rules: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ruleset {
    // filter table
    pub filter_input: ChainState,
    pub filter_output: ChainState,
    pub filter_forward: ChainState,
    // nat table
    pub nat_prerouting: ChainState,
    pub nat_postrouting: ChainState,
}

impl Ruleset {
    pub fn new() -> Self {
        Ruleset {
            filter_input: ChainState::new_accept(),
            filter_output: ChainState::new_accept(),
            filter_forward: ChainState::new_accept(),
            nat_prerouting: ChainState::new_no_policy(),
            nat_postrouting: ChainState::new_no_policy(),
        }
    }

    /// Applies a parsed rule to the ruleset.
    /// Returns an error string if the command is invalid.
    pub fn apply(&mut self, rule: &ParsedRule) -> Result<(), String> {
        match &rule.command {
            RuleCommand::Append { chain, .. } => {
                let chain_state = self.get_chain_mut(&rule.table, chain)?;
                chain_state.rules.push(rule.clone());
                Ok(())
            }
            RuleCommand::Insert { chain, pos, .. } => {
                let chain_state = self.get_chain_mut(&rule.table, chain)?;
                let idx = pos.map(|p| (p.saturating_sub(1)) as usize).unwrap_or(0);
                let idx = idx.min(chain_state.rules.len());
                chain_state.rules.insert(idx, rule.clone());
                Ok(())
            }
            RuleCommand::Delete { chain, rule_num } => {
                let chain_state = self.get_chain_mut(&rule.table, chain)?;
                if let Some(n) = rule_num {
                    let idx = (*n as usize).saturating_sub(1);
                    if idx < chain_state.rules.len() {
                        chain_state.rules.remove(idx);
                        Ok(())
                    } else {
                        Err(format!("No existe la regla número {}", n))
                    }
                } else {
                    Ok(()) // delete by spec: not implemented in Fase 1
                }
            }
            RuleCommand::Flush { chain } => {
                if let Some(c) = chain {
                    let chain_state = self.get_chain_mut(&rule.table, c)?;
                    chain_state.rules.clear();
                } else {
                    // flush all chains of the table
                    match &rule.table {
                        Table::Filter => {
                            self.filter_input.rules.clear();
                            self.filter_output.rules.clear();
                            self.filter_forward.rules.clear();
                        }
                        Table::Nat => {
                            self.nat_prerouting.rules.clear();
                            self.nat_postrouting.rules.clear();
                        }
                        _ => {}
                    }
                }
                Ok(())
            }
            RuleCommand::Policy { chain, target } => {
                let chain_state = self.get_chain_mut(&rule.table, chain)?;
                chain_state.policy = Some(target.clone());
                Ok(())
            }
            RuleCommand::List { .. } | RuleCommand::NewChain(_) | RuleCommand::DeleteChain(_) => {
                Ok(())
            } // no-op for Fase 1
        }
    }

    fn get_chain_mut(&mut self, table: &Table, chain: &Chain) -> Result<&mut ChainState, String> {
        match (table, chain) {
            (Table::Filter, Chain::Input) => Ok(&mut self.filter_input),
            (Table::Filter, Chain::Output) => Ok(&mut self.filter_output),
            (Table::Filter, Chain::Forward) => Ok(&mut self.filter_forward),
            (Table::Nat, Chain::Prerouting) => Ok(&mut self.nat_prerouting),
            (Table::Nat, Chain::Postrouting) => Ok(&mut self.nat_postrouting),
            _ => Err(format!("Cadena {:?} no válida en tabla {:?}", chain, table)),
        }
    }
}

/// Vista serializable del ruleset para el frontend
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChainView {
    pub name: String,
    pub policy: Option<String>,
    pub rules: Vec<RuleView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RuleView {
    pub index: usize,
    pub iptables: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RulesetView {
    pub filter_input: ChainView,
    pub filter_output: ChainView,
    pub filter_forward: ChainView,
    pub nat_prerouting: ChainView,
    pub nat_postrouting: ChainView,
}

impl From<&Ruleset> for RulesetView {
    fn from(r: &Ruleset) -> Self {
        RulesetView {
            filter_input: chain_to_view("INPUT", &r.filter_input),
            filter_output: chain_to_view("OUTPUT", &r.filter_output),
            filter_forward: chain_to_view("FORWARD", &r.filter_forward),
            nat_prerouting: chain_to_view("PREROUTING", &r.nat_prerouting),
            nat_postrouting: chain_to_view("POSTROUTING", &r.nat_postrouting),
        }
    }
}

fn chain_to_view(name: &str, chain: &ChainState) -> ChainView {
    ChainView {
        name: name.to_string(),
        policy: chain
            .policy
            .as_ref()
            .map(|t| format!("{:?}", t).to_uppercase()),
        rules: chain
            .rules
            .iter()
            .enumerate()
            .map(|(i, r)| RuleView {
                index: i + 1,
                iptables: r.raw.clone(),
            })
            .collect(),
    }
}
