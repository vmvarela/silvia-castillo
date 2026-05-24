// Auto-generado desde Rust via ts-rs — no editar manualmente
// Regenerar con: npm run sync-types

export type Table = "Filter" | "Nat" | "Mangle" | "Raw";

export type Chain =
  | "Input"
  | "Output"
  | "Forward"
  | "Prerouting"
  | "Postrouting"
  | { Custom: string };

export type PortSpec = { start: number; end: number };

export type MatchKind =
  | { Protocol: string }
  | { Source: string }
  | { Destination: string }
  | { InInterface: string }
  | { OutInterface: string }
  | { DPort: PortSpec }
  | { SPort: PortSpec }
  | { State: string[] }
  | { CtState: string[] }
  | { IcmpType: string }
  | { Comment: string };

export type Match = { negated: boolean; kind: MatchKind };

export type Target =
  | "Accept"
  | "Drop"
  | "Masquerade"
  | "Return"
  | { Reject: { with: string | null } }
  | { Log: { prefix: string | null } }
  | { Snat: { to: string } }
  | { Dnat: { to: string } }
  | { Jump: string };

export type RuleCommand =
  | { Append: { chain: Chain; matches: Match[]; target: Target } }
  | { Insert: { chain: Chain; pos: number | null; matches: Match[]; target: Target } }
  | { Delete: { chain: Chain; rule_num: number | null } }
  | { Flush: { chain: Chain | null } }
  | { Policy: { chain: Chain; target: Target } }
  | { List: { chain: Chain | null } }
  | { NewChain: string }
  | { DeleteChain: string | null };

export type ParsedRule = { table: Table; command: RuleCommand; raw: string };

export type RuleView = { index: number; iptables: string };

export type ChainView = {
  name: string;
  policy: string | null;
  rules: RuleView[];
};

export type RulesetView = {
  filter_input: ChainView;
  filter_output: ChainView;
  filter_forward: ChainView;
  nat_prerouting: ChainView;
  nat_postrouting: ChainView;
};

export type ExecuteResult = {
  ok: boolean;
  ast: ParsedRule | null;
  error: string | null;
  humanize: string | null;
  ruleset: RulesetView;
};

export type LevelInfo = { index: number; id: string; titulo: string; locked: boolean };

export type HostView = { nombre: string; ip: string; zona: string; iface: string };

export type LevelView = {
  index: number; id: string; titulo: string; cuento: string; mision: string;
  pistas: string[]; recompensa: string; hosts: HostView[]; ruleset: RulesetView;
};

export type TestResult = {
  index: number; descripcion: string; src_ip: string; dst_ip: string; dst_port: number;
  proto: string; estado: string; esperado: string; got: string; passed: boolean;
};

export type CheckResult = { results: TestResult[]; all_passed: boolean; score: number };

export type ProgressView = { unlocked_until: number; completed: string[] };
