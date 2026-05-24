use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Table {
    #[default]
    Filter,
    Nat,
    Mangle,
    Raw,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Chain {
    Input,
    Output,
    Forward,
    Prerouting,
    Postrouting,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Target {
    Accept,
    Drop,
    Reject { with: Option<String> },
    Log { prefix: Option<String> },
    Masquerade,
    Snat { to: String },
    Dnat { to: String },
    Return,
    Jump(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PortSpec {
    pub start: u16,
    pub end: u16, // end == start means single port
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum MatchKind {
    Protocol(String),
    Source(String),
    Destination(String),
    InInterface(String),
    OutInterface(String),
    DPort(PortSpec),
    SPort(PortSpec),
    State(Vec<String>),
    CtState(Vec<String>),
    IcmpType(String),
    Comment(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Match {
    pub negated: bool,
    pub kind: MatchKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum RuleCommand {
    Append {
        chain: Chain,
        matches: Vec<Match>,
        target: Target,
    },
    Insert {
        chain: Chain,
        pos: Option<u32>,
        matches: Vec<Match>,
        target: Target,
    },
    Delete {
        chain: Chain,
        rule_num: Option<u32>,
    },
    Flush {
        chain: Option<Chain>,
    },
    Policy {
        chain: Chain,
        target: Target,
    },
    List {
        chain: Option<Chain>,
    },
    NewChain(String),
    DeleteChain(Option<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ParsedRule {
    pub table: Table,
    pub command: RuleCommand,
    pub raw: String,
}
