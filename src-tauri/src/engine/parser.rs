use crate::engine::ast::{
    Chain, Match, MatchKind, ParsedRule, PortSpec, RuleCommand, Table, Target,
};

pub fn parse_line(input: &str) -> Result<ParsedRule, String> {
    let raw = input.to_string();
    let tokens: Vec<&str> = input.trim().split_whitespace().collect();
    if tokens.is_empty() {
        return Err("Comando vacío".to_string());
    }

    let mut iter = tokens.into_iter().peekable();

    // Skip optional iptables / iptables6 prefix
    if let Some(&first) = iter.peek() {
        if first == "iptables" || first == "iptables6" {
            iter.next();
        }
    }

    // Collect remaining tokens into a vec for two-pass table extraction
    let tokens_vec: Vec<&str> = iter.collect();
    let mut table = Table::Filter;
    let mut remaining: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < tokens_vec.len() {
        if tokens_vec[i] == "-t" || tokens_vec[i] == "--table" {
            if i + 1 < tokens_vec.len() {
                table = match tokens_vec[i + 1].to_lowercase().as_str() {
                    "filter" => Table::Filter,
                    "nat" => Table::Nat,
                    "mangle" => Table::Mangle,
                    "raw" => Table::Raw,
                    _ => Table::Filter,
                };
                i += 2;
                continue;
            } else {
                return Err("Falta valor para -t".to_string());
            }
        }
        remaining.push(tokens_vec[i]);
        i += 1;
    }

    let mut state = ParserState {
        command: None,
        matches: Vec::new(),
        target: None,
        negated: false,
    };

    let mut iter = remaining.into_iter().peekable();
    while let Some(token) = iter.next() {
        match token {
            "!" => {
                state.negated = true;
            }
            "-A" | "--append" => {
                let chain_str = iter
                    .next()
                    .ok_or_else(|| "Falta cadena para -A".to_string())?;
                state.command = Some(CommandKind::Append(parse_chain(chain_str)));
            }
            "-I" | "--insert" => {
                let chain_str = iter
                    .next()
                    .ok_or_else(|| "Falta cadena para -I".to_string())?;
                let pos = if let Some(&next) = iter.peek() {
                    if let Ok(n) = next.parse::<u32>() {
                        iter.next();
                        Some(n)
                    } else {
                        None
                    }
                } else {
                    None
                };
                state.command = Some(CommandKind::Insert(parse_chain(chain_str), pos));
            }
            "-D" | "--delete" => {
                let chain_str = iter
                    .next()
                    .ok_or_else(|| "Falta cadena para -D".to_string())?;
                let rule_num = if let Some(&next) = iter.peek() {
                    if let Ok(n) = next.parse::<u32>() {
                        iter.next();
                        Some(n)
                    } else {
                        None
                    }
                } else {
                    None
                };
                state.command = Some(CommandKind::Delete(parse_chain(chain_str), rule_num));
            }
            "-F" | "--flush" => {
                let chain = if let Some(&next) = iter.peek() {
                    if !next.starts_with('-') && next != "!" {
                        iter.next();
                        Some(parse_chain(next))
                    } else {
                        None
                    }
                } else {
                    None
                };
                state.command = Some(CommandKind::Flush(chain));
            }
            "-P" | "--policy" => {
                let chain_str = iter
                    .next()
                    .ok_or_else(|| "Falta cadena para -P".to_string())?;
                let target_str = iter
                    .next()
                    .ok_or_else(|| "Falta target para -P".to_string())?;
                state.command = Some(CommandKind::Policy(parse_chain(chain_str)));
                state.target = Some(parse_target_name(target_str));
            }
            "-L" | "--list" => {
                let chain = if let Some(&next) = iter.peek() {
                    if !next.starts_with('-') && next != "!" {
                        iter.next();
                        Some(parse_chain(next))
                    } else {
                        None
                    }
                } else {
                    None
                };
                state.command = Some(CommandKind::List(chain));
            }
            "-N" | "--new-chain" => {
                let name = iter
                    .next()
                    .ok_or_else(|| "Falta nombre para -N".to_string())?;
                state.command = Some(CommandKind::NewChain(name.to_string()));
            }
            "-X" | "--delete-chain" => {
                let name = if let Some(&next) = iter.peek() {
                    if !next.starts_with('-') && next != "!" {
                        iter.next();
                        Some(next.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };
                state.command = Some(CommandKind::DeleteChain(name));
            }
            "-p" | "--protocol" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Falta protocolo".to_string())?;
                state.matches.push(Match {
                    negated: state.negated,
                    kind: MatchKind::Protocol(val.to_string()),
                });
                state.negated = false;
            }
            "-s" | "--source" | "--src" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Falta origen".to_string())?;
                state.matches.push(Match {
                    negated: state.negated,
                    kind: MatchKind::Source(val.to_string()),
                });
                state.negated = false;
            }
            "-d" | "--destination" | "--dst" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Falta destino".to_string())?;
                state.matches.push(Match {
                    negated: state.negated,
                    kind: MatchKind::Destination(val.to_string()),
                });
                state.negated = false;
            }
            "-i" | "--in-interface" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Falta interfaz de entrada".to_string())?;
                state.matches.push(Match {
                    negated: state.negated,
                    kind: MatchKind::InInterface(val.to_string()),
                });
                state.negated = false;
            }
            "-o" | "--out-interface" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Falta interfaz de salida".to_string())?;
                state.matches.push(Match {
                    negated: state.negated,
                    kind: MatchKind::OutInterface(val.to_string()),
                });
                state.negated = false;
            }
            "--dport" | "--destination-port" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Falta puerto de destino".to_string())?;
                state.matches.push(Match {
                    negated: state.negated,
                    kind: MatchKind::DPort(parse_port_spec(val)?),
                });
                state.negated = false;
            }
            "--sport" | "--source-port" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Falta puerto de origen".to_string())?;
                state.matches.push(Match {
                    negated: state.negated,
                    kind: MatchKind::SPort(parse_port_spec(val)?),
                });
                state.negated = false;
            }
            "-m" | "--match" => {
                // consume module name, do nothing
                let _ = iter.next();
            }
            "--state" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Falta estado".to_string())?;
                let states: Vec<String> = val.split(',').map(|s| s.to_string()).collect();
                state.matches.push(Match {
                    negated: state.negated,
                    kind: MatchKind::State(states),
                });
                state.negated = false;
            }
            "--ctstate" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Falta ctstate".to_string())?;
                let states: Vec<String> = val.split(',').map(|s| s.to_string()).collect();
                state.matches.push(Match {
                    negated: state.negated,
                    kind: MatchKind::CtState(states),
                });
                state.negated = false;
            }
            "--icmp-type" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Falta tipo ICMP".to_string())?;
                state.matches.push(Match {
                    negated: state.negated,
                    kind: MatchKind::IcmpType(val.to_string()),
                });
                state.negated = false;
            }
            "--comment" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Falta comentario".to_string())?;
                state.matches.push(Match {
                    negated: state.negated,
                    kind: MatchKind::Comment(val.to_string()),
                });
                state.negated = false;
            }
            "-j" | "--jump" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Falta target para -j".to_string())?;
                state.target = Some(parse_target_name(val));
            }
            "-g" | "--goto" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Falta target para -g".to_string())?;
                state.target = Some(Target::Jump(val.to_string()));
            }
            "--to-destination" | "--to-dest" => {
                if let Some(Target::Dnat { ref mut to }) = state.target {
                    let val = iter
                        .next()
                        .ok_or_else(|| "Falta valor para --to-destination".to_string())?;
                    *to = val.to_string();
                } else {
                    // ignore if target is not Dnat
                    if let Some(&next) = iter.peek() {
                        if !next.starts_with('-') && next != "!" {
                            iter.next();
                        }
                    }
                }
            }
            "--to-source" => {
                if let Some(Target::Snat { ref mut to }) = state.target {
                    let val = iter
                        .next()
                        .ok_or_else(|| "Falta valor para --to-source".to_string())?;
                    *to = val.to_string();
                } else {
                    if let Some(&next) = iter.peek() {
                        if !next.starts_with('-') && next != "!" {
                            iter.next();
                        }
                    }
                }
            }
            "--reject-with" => {
                if let Some(Target::Reject { ref mut with }) = state.target {
                    let val = iter
                        .next()
                        .ok_or_else(|| "Falta valor para --reject-with".to_string())?;
                    *with = Some(val.to_string());
                } else {
                    if let Some(&next) = iter.peek() {
                        if !next.starts_with('-') && next != "!" {
                            iter.next();
                        }
                    }
                }
            }
            "--log-prefix" => {
                if let Some(Target::Log { ref mut prefix }) = state.target {
                    let val = iter
                        .next()
                        .ok_or_else(|| "Falta valor para --log-prefix".to_string())?;
                    *prefix = Some(val.to_string());
                } else {
                    if let Some(&next) = iter.peek() {
                        if !next.starts_with('-') && next != "!" {
                            iter.next();
                        }
                    }
                }
            }
            _ if token.starts_with('-') => {
                // Unknown flag: silently skip + skip next token if it doesn't start with '-'
                if let Some(&next) = iter.peek() {
                    if !next.starts_with('-') && next != "!" {
                        iter.next();
                    }
                }
            }
            _ => {
                // Unknown token, ignore
            }
        }
    }

    let command = build_command(state)?;
    Ok(ParsedRule { table, command, raw })
}

fn parse_chain(s: &str) -> Chain {
    match s.to_uppercase().as_str() {
        "INPUT" => Chain::Input,
        "OUTPUT" => Chain::Output,
        "FORWARD" => Chain::Forward,
        "PREROUTING" => Chain::Prerouting,
        "POSTROUTING" => Chain::Postrouting,
        _ => Chain::Custom(s.to_string()),
    }
}

fn parse_port_spec(s: &str) -> Result<PortSpec, String> {
    if let Some((start, end)) = s.split_once(':') {
        let start: u16 = start
            .parse()
            .map_err(|_| format!("Puerto inválido: {}", start))?;
        let end: u16 = end
            .parse()
            .map_err(|_| format!("Puerto inválido: {}", end))?;
        Ok(PortSpec { start, end })
    } else {
        let port: u16 = s
            .parse()
            .map_err(|_| format!("Puerto inválido: {}", s))?;
        Ok(PortSpec {
            start: port,
            end: port,
        })
    }
}

fn parse_target_name(s: &str) -> Target {
    match s.to_uppercase().as_str() {
        "ACCEPT" => Target::Accept,
        "DROP" => Target::Drop,
        "REJECT" => Target::Reject { with: None },
        "LOG" => Target::Log { prefix: None },
        "MASQUERADE" => Target::Masquerade,
        "SNAT" => Target::Snat { to: String::new() },
        "DNAT" => Target::Dnat { to: String::new() },
        "RETURN" => Target::Return,
        _ => Target::Jump(s.to_string()),
    }
}

struct ParserState {
    command: Option<CommandKind>,
    matches: Vec<Match>,
    target: Option<Target>,
    negated: bool,
}

enum CommandKind {
    Append(Chain),
    Insert(Chain, Option<u32>),
    Delete(Chain, Option<u32>),
    Flush(Option<Chain>),
    Policy(Chain),
    List(Option<Chain>),
    NewChain(String),
    DeleteChain(Option<String>),
}

fn build_command(state: ParserState) -> Result<RuleCommand, String> {
    let cmd = state
        .command
        .ok_or_else(|| "Comando no reconocido".to_string())?;
    match cmd {
        CommandKind::Append(chain) => Ok(RuleCommand::Append {
            chain,
            matches: state.matches,
            target: state.target.unwrap_or(Target::Accept),
        }),
        CommandKind::Insert(chain, pos) => Ok(RuleCommand::Insert {
            chain,
            pos,
            matches: state.matches,
            target: state.target.unwrap_or(Target::Accept),
        }),
        CommandKind::Delete(chain, rule_num) => Ok(RuleCommand::Delete { chain, rule_num }),
        CommandKind::Flush(chain) => Ok(RuleCommand::Flush { chain }),
        CommandKind::Policy(chain) => Ok(RuleCommand::Policy {
            chain,
            target: state
                .target
                .ok_or_else(|| "Falta target para política".to_string())?,
        }),
        CommandKind::List(chain) => Ok(RuleCommand::List { chain }),
        CommandKind::NewChain(name) => Ok(RuleCommand::NewChain(name)),
        CommandKind::DeleteChain(name) => Ok(RuleCommand::DeleteChain(name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_accept() {
        let r = parse_line("iptables -A INPUT -j ACCEPT").unwrap();
        assert!(matches!(r.table, Table::Filter));
        match r.command {
            RuleCommand::Append { chain, matches, target } => {
                assert_eq!(chain, Chain::Input);
                assert!(matches.is_empty());
                assert!(matches!(target, Target::Accept));
            }
            _ => panic!("Expected Append"),
        }
    }

    #[test]
    fn test_tcp_dport() {
        let r = parse_line("iptables -A INPUT -p tcp --dport 22 -j ACCEPT").unwrap();
        match r.command {
            RuleCommand::Append { chain, matches, target } => {
                assert_eq!(chain, Chain::Input);
                assert!(matches!(target, Target::Accept));
                assert_eq!(matches.len(), 2);
                assert!(matches!(matches[0].kind, MatchKind::Protocol(ref s) if s == "tcp"));
                assert!(matches!(matches[1].kind, MatchKind::DPort(PortSpec { start: 22, end: 22 })));
            }
            _ => panic!("Expected Append"),
        }
    }

    #[test]
    fn test_source_drop() {
        let r = parse_line("iptables -A FORWARD -s 192.168.1.0/24 -j DROP").unwrap();
        match r.command {
            RuleCommand::Append { chain, matches, target } => {
                assert_eq!(chain, Chain::Forward);
                assert!(matches!(target, Target::Drop));
                assert_eq!(matches.len(), 1);
                assert!(matches!(matches[0].kind, MatchKind::Source(ref s) if s == "192.168.1.0/24"));
            }
            _ => panic!("Expected Append"),
        }
    }

    #[test]
    fn test_no_prefix() {
        let r = parse_line("-A INPUT -p tcp --dport 80:443 -j ACCEPT").unwrap();
        match r.command {
            RuleCommand::Append { chain, matches, target } => {
                assert_eq!(chain, Chain::Input);
                assert!(matches!(target, Target::Accept));
                assert_eq!(matches.len(), 2);
                assert!(matches!(matches[1].kind, MatchKind::DPort(PortSpec { start: 80, end: 443 })));
            }
            _ => panic!("Expected Append"),
        }
    }

    #[test]
    fn test_flush_all() {
        let r = parse_line("iptables -F").unwrap();
        match r.command {
            RuleCommand::Flush { chain: None } => {}
            _ => panic!("Expected Flush all"),
        }
    }

    #[test]
    fn test_flush_chain() {
        let r = parse_line("iptables -F INPUT").unwrap();
        match r.command {
            RuleCommand::Flush { chain: Some(Chain::Input) } => {}
            _ => panic!("Expected Flush INPUT"),
        }
    }

    #[test]
    fn test_policy() {
        let r = parse_line("iptables -P INPUT DROP").unwrap();
        match r.command {
            RuleCommand::Policy { chain, target } => {
                assert_eq!(chain, Chain::Input);
                assert!(matches!(target, Target::Drop));
            }
            _ => panic!("Expected Policy"),
        }
    }

    #[test]
    fn test_nat_dnat() {
        let r = parse_line(
            "iptables -t nat -A PREROUTING -p tcp --dport 80 -j DNAT --to-destination 192.168.1.10:8080",
        )
        .unwrap();
        assert!(matches!(r.table, Table::Nat));
        match r.command {
            RuleCommand::Append { chain, target, .. } => {
                assert_eq!(chain, Chain::Prerouting);
                assert!(matches!(target, Target::Dnat { ref to } if to == "192.168.1.10:8080"));
            }
            _ => panic!("Expected Append"),
        }
    }

    #[test]
    fn test_negation() {
        let r = parse_line("iptables -A INPUT ! -s 192.168.1.0/24 -j DROP").unwrap();
        match r.command {
            RuleCommand::Append { matches, target, .. } => {
                assert!(matches!(target, Target::Drop));
                assert_eq!(matches.len(), 1);
                assert!(matches[0].negated);
                assert!(matches!(matches[0].kind, MatchKind::Source(ref s) if s == "192.168.1.0/24"));
            }
            _ => panic!("Expected Append"),
        }
    }

    #[test]
    fn test_state_match() {
        let r = parse_line("iptables -A INPUT -m state --state NEW,ESTABLISHED -j ACCEPT").unwrap();
        match r.command {
            RuleCommand::Append { matches, target, .. } => {
                assert!(matches!(target, Target::Accept));
                assert_eq!(matches.len(), 1);
                assert!(matches!(matches[0].kind, MatchKind::State(ref v) if v == &["NEW", "ESTABLISHED"]));
            }
            _ => panic!("Expected Append"),
        }
    }

    #[test]
    fn test_unknown_flags_ignored() {
        let r = parse_line("iptables -A INPUT -p tcp --dport 22 --unknown-flag value -j DROP").unwrap();
        match r.command {
            RuleCommand::Append { matches, target, .. } => {
                assert!(matches!(target, Target::Drop));
                assert_eq!(matches.len(), 2);
                assert!(matches!(matches[0].kind, MatchKind::Protocol(ref s) if s == "tcp"));
                assert!(matches!(matches[1].kind, MatchKind::DPort(PortSpec { start: 22, end: 22 })));
            }
            _ => panic!("Expected Append"),
        }
    }
}
