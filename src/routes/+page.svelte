<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { getLevelList, getProgress } from '$lib/tauri/commands';
  import type { LevelInfo, ProgressView } from '$lib/tauri/types';

  let levels = $state<LevelInfo[]>([]);
  let progress = $state<ProgressView | null>(null);
  let loading = $state(true);
  let errorMsg = $state<string | null>(null);

  onMount(async () => {
    try {
      progress = await getProgress();
      levels = await getLevelList();
    } catch (e) {
      errorMsg = String(e);
    } finally {
      loading = false;
    }
  });

  function isCompleted(id: string) {
    return progress?.completed.includes(id) ?? false;
  }
</script>

<svelte:head>
  <title>Castillo de Silvia - Niveles</title>
</svelte:head>

<main class="page-container">
  <header>
    <div class="header-titles">
      <div class="castle-art">🏰</div>
      <div>
        <h1>Castillo de Silvia</h1>
        <p class="subtitle">Aprende a proteger el castillo con reglas de firewall</p>
      </div>
    </div>
    {#if progress}
      <div class="progress-badge">
        Progreso: {progress.completed.length} / 9 niveles
      </div>
    {/if}
  </header>

  {#if loading}
    <div class="loading-state">
      <p>Cargando niveles...</p>
    </div>
  {:else if errorMsg}
    <div class="error-state">
      <p>Error: {errorMsg}</p>
    </div>
  {:else}
    <div class="levels-grid">
      {#each levels as level}
        {@const completed = isCompleted(level.id)}
        {@const cardClass = level.locked ? 'card locked' : completed ? 'card completed' : 'card unlocked'}
        
        <button 
          class={cardClass}
          onclick={() => { if (!level.locked) goto('/level/' + (level.index + 1)); }}
          disabled={level.locked}
        >
          <div class="card-header">
            <span class="level-num">Nivel {level.index + 1}</span>
            <span class="level-emoji">
              {#if level.locked}
                🔒
              {:else if completed}
                ✅
              {:else}
                🏰
              {/if}
            </span>
          </div>
          <h2 class="level-title">{level.titulo}</h2>
          {#if completed}
            <span class="tag tag-accept completed-badge">✅ Completado</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</main>

<style>
  .page-container {
    height: 100vh;
    display: flex;
    flex-direction: column;
    padding: 2rem;
    background: radial-gradient(ellipse at center top, var(--bg-panel) 0%, var(--bg) 100%);
    overflow-y: auto;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 3rem;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .header-titles {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .castle-art {
    font-size: 3rem;
    line-height: 1;
    filter: drop-shadow(0 0 10px rgba(255, 215, 0, 0.4));
  }

  h1 {
    font-size: 2rem;
    color: var(--gold);
    margin: 0 0 0.25rem 0;
  }

  .subtitle {
    color: var(--text-muted);
    font-size: 1rem;
    margin: 0;
  }

  .progress-badge {
    background: rgba(255, 215, 0, 0.1);
    border: 1px solid rgba(255, 215, 0, 0.3);
    color: var(--gold);
    padding: 8px 16px;
    border-radius: 20px;
    font-family: var(--font-mono);
    font-size: 0.9rem;
  }

  .loading-state, .error-state {
    text-align: center;
    margin-top: 4rem;
    font-size: 1.2rem;
  }

  .error-state {
    color: var(--error);
  }

  .levels-grid {
    display: grid;
    grid-template-columns: repeat(1, 1fr);
    gap: 1.5rem;
    max-width: 1200px;
    margin: 0 auto;
    width: 100%;
  }

  @media (min-width: 768px) {
    .levels-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (min-width: 1024px) {
    .levels-grid {
      grid-template-columns: repeat(3, 1fr);
    }
  }

  .card {
    background: var(--bg-surface);
    border: 2px solid rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-lg);
    padding: 1.5rem;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    box-shadow: var(--shadow);
    transition: all var(--transition);
    position: relative;
    overflow: hidden;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .level-num {
    font-family: var(--font-mono);
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .level-emoji {
    font-size: 1.5rem;
  }

  .level-title {
    font-size: 1.25rem;
    color: var(--text);
    margin: 0;
    line-height: 1.4;
  }

  .completed-badge {
    align-self: flex-start;
    margin-top: auto;
  }

  .unlocked {
    cursor: pointer;
  }

  .unlocked:hover {
    border-color: var(--gold);
    transform: translateY(-4px);
    box-shadow: 0 8px 30px rgba(255, 215, 0, 0.15);
  }

  .completed {
    border-color: rgba(76, 175, 80, 0.5);
    background: linear-gradient(145deg, var(--bg-surface) 0%, rgba(76, 175, 80, 0.05) 100%);
    cursor: pointer;
  }

  .completed:hover {
    border-color: var(--success);
    transform: translateY(-4px);
  }

  .locked {
    opacity: 0.5;
    cursor: not-allowed;
    background: rgba(15, 52, 96, 0.3);
  }
</style>
