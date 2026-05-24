import { invoke, isTauri } from '@tauri-apps/api/core';

function requireTauri<T>(fn: () => Promise<T>): Promise<T> {
  if (!isTauri()) {
    return Promise.reject(
      new Error('Abre el juego con "npx tauri dev", no en el navegador.')
    );
  }
  return fn();
}
import type { 
  ExecuteResult, RulesetView, LevelInfo, LevelView, CheckResult, ProgressView 
} from './types';

export async function executeCommand(input: string): Promise<ExecuteResult> {
  return requireTauri(() => invoke('execute_command', { input }));
}

export async function getRuleset(): Promise<RulesetView> {
  return requireTauri(() => invoke('get_ruleset'));
}

export async function resetRuleset(): Promise<RulesetView> {
  return requireTauri(() => invoke('reset_ruleset'));
}

export async function getLevelList(): Promise<LevelInfo[]> {
  return requireTauri(() => invoke('get_level_list'));
}

export async function loadLevel(index: number): Promise<LevelView> {
  return requireTauri(() => invoke('load_level', { index }));
}

export async function checkTests(): Promise<CheckResult> {
  return requireTauri(() => invoke('check_tests'));
}

export async function markLevelComplete(): Promise<ProgressView> {
  return requireTauri(() => invoke('mark_level_complete'));
}

export async function getProgress(): Promise<ProgressView> {
  return requireTauri(() => invoke('get_progress'));
}
