<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import Console from '$lib/components/console/Console.svelte';
  import { getRuleset, resetRuleset } from '$lib/tauri/commands';
  import type { ExecuteResult, RulesetView } from '$lib/tauri/types';

  const n = $derived($page.params.n);

  let ruleset = $state<RulesetView | null>(null);
  let consoleRef: ReturnType<typeof Console> | null = $state(null);

  onMount(async () => {
    ruleset = await getRuleset();
  });

  function handleResult(result: ExecuteResult) {
    ruleset = result.ruleset;
  }

  async function handleReset() {
    ruleset = await resetRuleset();
    consoleRef?.clear();
  }

  function chainRows(chain: import('$lib/tauri/types').ChainView) {
    return chain.rules;
  }
</script>

<svelte:head>
  <title>Nivel {n} — Castillo de Silvia</title>
</svelte:head>

<div class="layout">
  <!-- Left panel: story + rules -->
  <aside class="panel left-panel">
    <nav class="breadcrumb">
      <button class="btn-ghost back-btn" onclick={() => goto('/')}>← Menú</button>
      <span class="level-badge">Nivel {n}</span>
    </nav>

    <section class="story">
      <h2>📜 Misión</h2>
      <p class="story-text">
        Los niveles llegarán pronto. Por ahora, practica con cualquier comando iptables.
      </p>
    </section>

    <section class="rules-section">
      <div class="rules-header">
        <h2>📋 Reglas activas</h2>
        <button class="btn-ghost reset-btn" onclick={handleReset} title="Resetear reglas">
          🔄
        </button>
      </div>

      {#if ruleset}
        {#each [
          { label: 'filter/INPUT',       chain: ruleset.filter_input },
          { label: 'filter/OUTPUT',      chain: ruleset.filter_output },
          { label: 'filter/FORWARD',     chain: ruleset.filter_forward },
          { label: 'nat/PREROUTING',     chain: ruleset.nat_prerouting },
          { label: 'nat/POSTROUTING',    chain: ruleset.nat_postrouting },
        ] as entry}
          <div class="chain-block">
            <div class="chain-header">
              <span class="chain-name">{entry.label}</span>
              {#if entry.chain.policy}
                <span class="tag tag-{entry.chain.policy.toLowerCase()}">{entry.chain.policy}</span>
              {/if}
            </div>
            {#if entry.chain.rules.length === 0}
              <p class="empty-chain">— vacía —</p>
            {:else}
              <ol class="rule-list">
                {#each entry.chain.rules as rule}
                  <li class="rule-item">
                    <span class="rule-num">{rule.index}</span>
                    <code class="rule-text">{rule.iptables}</code>
                  </li>
                {/each}
              </ol>
            {/if}
          </div>
        {/each}
      {:else}
        <p class="loading">Cargando...</p>
      {/if}
    </section>
  </aside>

  <!-- Right panel: console -->
  <main class="panel right-panel">
    <div class="console-header">
      <span class="console-title">🖥️ Terminal</span>
      <span class="console-hint">Escribe comandos iptables</span>
    </div>
    <div class="console-container">
      <Console onResult={handleResult} bind:this={consoleRef} />
    </div>
  </main>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: 340px 1fr;
    grid-template-rows: 100vh;
    gap: 12px;
    padding: 12px;
    height: 100vh;
    background: var(--bg);
  }

  .left-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    overflow-y: auto;
    padding: 1rem;
  }

  .right-panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0;
  }

  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .back-btn {
    font-size: 0.875rem;
    padding: 6px 12px;
  }

  .level-badge {
    font-weight: 700;
    color: var(--gold);
    font-size: 1rem;
  }

  .story h2,
  .rules-section h2 {
    font-size: 0.875rem;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-muted);
    margin-bottom: 0.5rem;
  }

  .story-text {
    color: var(--text);
    line-height: 1.6;
    font-size: 0.9rem;
  }

  .rules-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }

  .reset-btn {
    padding: 4px 8px;
    font-size: 1rem;
  }

  .chain-block {
    margin-bottom: 0.75rem;
    background: var(--bg-surface);
    border-radius: var(--radius);
    overflow: hidden;
  }

  .chain-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    background: rgba(255,255,255,0.05);
  }

  .chain-name {
    font-family: var(--font-mono);
    font-size: 0.8rem;
    color: var(--gold);
  }

  .empty-chain {
    color: var(--text-muted);
    font-size: 0.8rem;
    padding: 6px 10px;
    font-style: italic;
  }

  .rule-list {
    list-style: none;
    padding: 4px 0;
  }

  .rule-item {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    padding: 4px 10px;
    border-top: 1px solid rgba(255,255,255,0.03);
  }

  .rule-num {
    color: var(--text-muted);
    font-size: 0.75rem;
    font-family: var(--font-mono);
    min-width: 16px;
    margin-top: 2px;
  }

  .rule-text {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    color: var(--text);
    word-break: break-all;
  }

  .loading {
    color: var(--text-muted);
    font-size: 0.875rem;
  }

  .console-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    background: rgba(255,255,255,0.03);
    border-bottom: 1px solid rgba(255,255,255,0.05);
  }

  .console-title {
    font-weight: 600;
    color: var(--text);
  }

  .console-hint {
    font-size: 0.8rem;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .console-container {
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }
</style>
