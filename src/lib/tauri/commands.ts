import { invoke } from '@tauri-apps/api/core';
import type { ExecuteResult, RulesetView } from './types';

export async function executeCommand(input: string): Promise<ExecuteResult> {
  return invoke('execute_command', { input });
}

export async function getRuleset(): Promise<RulesetView> {
  return invoke('get_ruleset');
}

export async function resetRuleset(): Promise<RulesetView> {
  return invoke('reset_ruleset');
}
