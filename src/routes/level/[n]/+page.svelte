<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import Console from '$lib/components/console/Console.svelte';
  import { 
    loadLevel, checkTests, markLevelComplete, resetRuleset 
  } from '$lib/tauri/commands';
  import type { 
    ExecuteResult, LevelView, CheckResult, ProgressView 
  } from '$lib/tauri/types';
  import { audio } from '$lib/audio.svelte';

  const n = $derived($page.params.n || '1');
  const idx = $derived(parseInt(n) - 1);

  let level = $state<LevelView | null>(null);
  let checkResult = $state<CheckResult | null>(null);
  let progress = $state<ProgressView | null>(null);
  let consoleRef: any = $state(null);
  let loading = $state(true);
  let errorMsg = $state<string | null>(null);
  let showHints = $state(false);

  // Versión para cancelar peticiones obsoletas si el usuario navega rápido
  let loadVersion = 0;

  // $effect reactivo a idx: se re-ejecuta al navegar entre niveles
  $effect(() => {
    const currentIdx = idx;
    const version = ++loadVersion;

    loading = true;
    errorMsg = null;
    level = null;
    checkResult = null;
    showHints = false;

    loadLevel(currentIdx)
      .then((l) => {
        if (version === loadVersion) {
          level = l;
          loading = false;
        }
      })
      .catch((e) => {
        if (version === loadVersion) {
          errorMsg = String(e);
          loading = false;
        }
      });
  });

  function handleResult(result: ExecuteResult) {
    if (level) {
      level = { ...level, ruleset: result.ruleset };
    }
  }

  async function handleReset() {
    try {
      const newRuleset = await resetRuleset();
      if (level) {
        level = { ...level, ruleset: newRuleset };
      }
      consoleRef?.clear();
      checkResult = null;
    } catch (e) {
      console.error(e);
    }
  }

  async function handleCheck() {
    try {
      checkResult = await checkTests();
      // Sonidos escalonados por test
      checkResult.results.forEach((test, i) => {
        setTimeout(() => {
          if (test.passed) audio.testPass();
          else audio.testFail();
        }, i * 130);
      });
      // Fanfarria si todos pasaron
      if (checkResult.all_passed) {
        setTimeout(() => audio.victory(), checkResult!.results.length * 130 + 100);
      }
    } catch (e) {
      console.error(e);
    }
  }

  async function handleComplete() {
    try {
      progress = await markLevelComplete();
      audio.levelComplete();
    } catch (e) {
      console.error(e);
    }
  }

  function getHostEmoji(zona: string, isFirewall: boolean) {
    if (isFirewall) return '🏰';
    if (zona === 'barrio') return '👸';
    if (zona === 'mundo' || zona === 'exterior') return '🐥';
    return '💻';
  }
</script>

<svelte:head>
  <title>Nivel {n} — Castillo de Silvia</title>
</svelte:head>

{#if loading}
  <div class="full-center">
    <p>Cargando nivel {n}...</p>
  </div>
{:else if errorMsg}
  <div class="full-center">
    <p class="error-text">Error: {errorMsg}</p>
    <button class="btn-ghost" onclick={() => goto('/')}>← Volver al menú</button>
  </div>
{:else if level}
  <div class="layout">
    <!-- Left panel: story + hosts -->
    <aside class="panel left-panel">
      <nav class="breadcrumb">
        <button class="btn-ghost back-btn" onclick={() => goto('/')}>← Volver</button>
        <span class="level-badge">Nivel {n}</span>
      </nav>

      <div class="scroll-content">
        <section class="story">
          <h1 class="level-title">{level.titulo}</h1>
          <div class="story-text">
            {level.cuento}
          </div>
        </section>

        <section class="mission-box">
          <h2>📜 Misión</h2>
          <p>{level.mision}</p>
        </section>

        {#if level.pistas && level.pistas.length > 0}
          <section class="hints-section">
            <button 
              class="hints-toggle" 
              onclick={() => showHints = !showHints}
            >
              💡 Pistas {showHints ? '▲' : '▼'}
            </button>
            {#if showHints}
              <ul class="hints-list">
                {#each level.pistas as pista}
                  <li>{pista}</li>
                {/each}
              </ul>
            {/if}
          </section>
        {/if}

        <section class="hosts-section">
          <h2>🗺️ Red del Castillo</h2>
          <div class="table-container">
            <table class="hosts-table">
              <thead>
                <tr>
                  <th></th>
                  <th>Nombre</th>
                  <th>IP</th>
                  <th>Zona</th>
                  <th>Iface</th>
                </tr>
              </thead>
              <tbody>
                {#each level.hosts as host}
                  {@const isFirewall = host.nombre.toLowerCase().includes('firewall') || host.nombre.toLowerCase() === 'iptables'}
                  <tr>
                    <td>{getHostEmoji(host.zona, isFirewall)}</td>
                    <td>{host.nombre}</td>
                    <td class="font-mono text-small">{host.ip}</td>
                    <td><span class="tag">{host.zona}</span></td>
                    <td class="font-mono text-small">{host.iface}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </section>
      </div>
    </aside>

    <!-- Center panel: console -->
    <main class="panel center-panel">
      <div class="console-header">
        <div>
          <span class="console-title">🖥️ Terminal</span>
          <span class="console-hint">Escribe comandos iptables</span>
        </div>
        <div class="console-actions">
          <button
            class="btn-ghost audio-btn"
            onclick={() => audio.toggleMute()}
            title={audio.muted ? 'Activar sonido' : 'Silenciar'}
          >{audio.muted ? '🔇' : '🔊'}</button>
          <button
            class="btn-ghost audio-btn"
            onclick={() => audio.toggleMusic()}
            title={audio.musicOn ? 'Parar música' : 'Música de fondo'}
          >{audio.musicOn ? '🎵' : '🔕'}</button>
          <button class="btn-ghost reset-btn" onclick={handleReset} title="Resetear reglas">
            🔄 Reset
          </button>
          <button class="btn-primary check-btn" onclick={handleCheck}>
            🧪 Comprobar
          </button>
        </div>
      </div>
      
      <div class="console-container">
        <Console onResult={handleResult} bind:this={consoleRef} />
      </div>

      <div class="rules-section">
        <div class="rules-header">
          <h2>📋 Reglas activas</h2>
        </div>
        <div class="chains-container">
          {#each [
            { label: 'filter/INPUT',       chain: level.ruleset.filter_input },
            { label: 'filter/OUTPUT',      chain: level.ruleset.filter_output },
            { label: 'filter/FORWARD',     chain: level.ruleset.filter_forward },
            { label: 'nat/PREROUTING',     chain: level.ruleset.nat_prerouting },
            { label: 'nat/POSTROUTING',    chain: level.ruleset.nat_postrouting },
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
        </div>
      </div>
    </main>

    <!-- Right panel: tests -->
    <aside class="panel right-panel">
      <div class="tests-header">
        <h2>🧪 Resultados</h2>
      </div>

      <div class="tests-content">
        {#if !checkResult}
          <div class="tests-empty">
            <p>Escribe tus reglas y pulsa <strong>Comprobar</strong> cuando creas que has cumplido la misión.</p>
          </div>
        {:else}
          <div class="tests-list">
            {#each checkResult.results as test}
              <div class="test-item {test.passed ? 'passed' : 'failed'}">
                <div class="test-title">
                  <span class="test-icon">{test.passed ? '✅' : '❌'}</span>
                  <span>{test.descripcion}</span>
                </div>
                <div class="test-details">
                  <div class="test-route">{test.src_ip} → {test.dst_ip}:{test.dst_port}</div>
                  <div class="test-compare">
                    Esperado: <span class="expected">{test.esperado}</span><br>
                    Obtenido: <span class="got">{test.got}</span>
                  </div>
                </div>
              </div>
            {/each}
          </div>

          <div class="score-section">
            <div class="score-text">
              {Math.round(checkResult.score * checkResult.results.length)} / {checkResult.results.length} tests pasados
            </div>
            <div class="progress-bar-bg">
              <div 
                class="progress-bar-fill {checkResult.all_passed ? 'perfect' : ''}" 
                style="width: {checkResult.score * 100}%"
              ></div>
            </div>
          </div>

          {#if checkResult.all_passed}
            <div class="success-banner">
              <h3>🎉 ¡Todos los tests superados!</h3>
              
              {#if progress}
                <div class="reward-box">
                  <p><strong>Recompensa:</strong> {level.recompensa}</p>
                  <button class="btn-primary next-btn" onclick={() => goto('/level/' + (level!.index + 2))}>
                    → Siguiente nivel
                  </button>
                </div>
              {:else}
                <button class="btn-primary complete-btn" onclick={handleComplete}>
                  ✅ Marcar nivel completo
                </button>
              {/if}
            </div>
          {/if}
        {/if}
      </div>
    </aside>
  </div>
{/if}

<style>
  .full-center {
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    background: var(--bg);
  }

  .error-text {
    color: var(--error);
    font-size: 1.2rem;
  }

  .layout {
    display: grid;
    grid-template-columns: 320px 1fr 300px;
    grid-template-rows: 100vh;
    gap: 16px;
    padding: 16px;
    height: 100vh;
    background: var(--bg);
  }

  @media (max-width: 1200px) {
    .layout {
      grid-template-columns: 300px 1fr 280px;
    }
  }

  /* Left Panel */
  .left-panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 16px;
    border-bottom: 1px solid rgba(255,255,255,0.05);
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

  .scroll-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .level-title {
    font-size: 1.5rem;
    color: var(--gold);
    margin-bottom: 0.75rem;
    line-height: 1.2;
  }

  .story-text {
    color: var(--text);
    line-height: 1.6;
    font-size: 0.95rem;
    white-space: pre-wrap;
  }

  .mission-box {
    background: rgba(255, 215, 0, 0.05);
    border: 1px solid rgba(255, 215, 0, 0.3);
    border-radius: var(--radius);
    padding: 12px;
  }

  .mission-box h2, .hosts-section h2 {
    font-size: 0.875rem;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--gold);
    margin-bottom: 0.5rem;
  }

  .mission-box p {
    font-size: 0.95rem;
    line-height: 1.5;
  }

  .hints-toggle {
    width: 100%;
    text-align: left;
    background: rgba(255,255,255,0.05);
    color: var(--text);
    padding: 10px 12px;
    border-radius: var(--radius);
    font-weight: 600;
    display: flex;
    justify-content: space-between;
  }

  .hints-toggle:hover {
    background: rgba(255,255,255,0.1);
  }

  .hints-list {
    margin: 8px 0 0 0;
    padding-left: 24px;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .hints-list li {
    margin-bottom: 6px;
  }

  .table-container {
    overflow-x: auto;
  }

  .hosts-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
  }

  .hosts-table th, .hosts-table td {
    padding: 8px 6px;
    text-align: left;
    border-bottom: 1px solid rgba(255,255,255,0.05);
  }

  .hosts-table th {
    color: var(--text-muted);
    font-weight: normal;
  }

  .font-mono {
    font-family: var(--font-mono);
  }

  .text-small {
    font-size: 0.8rem;
  }

  /* Center Panel */
  .center-panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0;
  }

  .console-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: rgba(0,0,0,0.2);
    border-bottom: 1px solid rgba(255,255,255,0.05);
  }

  .console-title {
    font-weight: 600;
    color: var(--text);
    margin-right: 8px;
  }

  .console-hint {
    font-size: 0.8rem;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .console-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .audio-btn {
    padding: 6px 8px;
    font-size: 1rem;
    min-width: 36px;
    opacity: 0.75;
    transition: opacity 0.15s;
  }

  .audio-btn:hover {
    opacity: 1;
  }

  .reset-btn {
    padding: 6px 12px;
    font-size: 0.875rem;
  }

  .check-btn {
    padding: 6px 16px;
    font-size: 0.9rem;
  }

  .console-container {
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .rules-section {
    height: 35%;
    min-height: 200px;
    border-top: 1px solid rgba(255,255,255,0.1);
    background: rgba(0,0,0,0.1);
    display: flex;
    flex-direction: column;
  }

  .rules-header {
    padding: 8px 16px;
    background: rgba(255,255,255,0.02);
    border-bottom: 1px solid rgba(255,255,255,0.05);
  }

  .rules-header h2 {
    font-size: 0.875rem;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-muted);
  }

  .chains-container {
    flex: 1;
    overflow-y: auto;
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .chain-block {
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

  /* Right Panel */
  .right-panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .tests-header {
    padding: 16px;
    border-bottom: 1px solid rgba(255,255,255,0.05);
  }

  .tests-header h2 {
    font-size: 1.1rem;
    color: var(--text);
  }

  .tests-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .tests-empty {
    color: var(--text-muted);
    text-align: center;
    margin-top: 2rem;
    line-height: 1.5;
  }

  .tests-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .test-item {
    padding: 12px;
    border-radius: var(--radius);
    border: 1px solid transparent;
  }

  .test-item.passed {
    background: rgba(76, 175, 80, 0.1);
    border-color: rgba(76, 175, 80, 0.2);
  }

  .test-item.failed {
    background: rgba(244, 67, 54, 0.1);
    border-color: rgba(244, 67, 54, 0.2);
  }

  .test-title {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    font-weight: 600;
    margin-bottom: 8px;
    font-size: 0.95rem;
  }

  .test-details {
    padding-left: 28px;
    font-size: 0.85rem;
  }

  .test-route {
    font-family: var(--font-mono);
    color: var(--text-muted);
    margin-bottom: 4px;
  }

  .test-compare {
    background: rgba(0,0,0,0.2);
    padding: 6px 8px;
    border-radius: 4px;
    font-family: var(--font-mono);
  }

  .expected { color: var(--text); }
  .got { color: var(--text-muted); }

  .score-section {
    margin-top: 8px;
  }

  .score-text {
    font-weight: 600;
    margin-bottom: 8px;
    text-align: right;
  }

  .progress-bar-bg {
    height: 8px;
    background: rgba(255,255,255,0.1);
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.5s ease;
  }

  .progress-bar-fill.perfect {
    background: var(--success);
  }

  .success-banner {
    margin-top: 1rem;
    padding: 1.5rem;
    background: rgba(76, 175, 80, 0.15);
    border: 2px solid var(--success);
    border-radius: var(--radius-lg);
    text-align: center;
    animation: popIn 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  }

  @keyframes popIn {
    0% { transform: scale(0.9); opacity: 0; }
    100% { transform: scale(1); opacity: 1; }
  }

  .success-banner h3 {
    color: var(--success);
    margin-bottom: 1rem;
    font-size: 1.2rem;
  }

  .complete-btn, .next-btn {
    width: 100%;
    font-size: 1rem;
    padding: 12px;
  }

  .complete-btn {
    background: var(--success);
  }

  .complete-btn:hover {
    background: #388e3c;
  }

  .reward-box {
    text-align: left;
  }

  .reward-box p {
    margin-bottom: 1rem;
    font-size: 0.95rem;
    line-height: 1.4;
  }
</style>
