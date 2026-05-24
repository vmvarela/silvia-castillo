<script lang="ts">
  import { goto } from '$app/navigation';
</script>

<svelte:head>
  <title>Castillo de Silvia</title>
</svelte:head>

<main>
  <div class="hero">
    <div class="castle-art">🏰</div>
    <h1>Castillo de Silvia</h1>
    <p class="subtitle">Aprende a proteger el castillo con reglas de firewall</p>

    <div class="tags">
      <span class="tag">8 niveles</span>
      <span class="tag">👸 princesas</span>
      <span class="tag">🐥 patitos</span>
    </div>

    <button class="btn-primary play-btn" onclick={() => goto('/level/1')}>
      ⚔️ Jugar
    </button>

    <p class="hint">
      Aprenderás iptables — el guardián real de las redes Linux
    </p>
  </div>
</main>

<style>
  main {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: radial-gradient(ellipse at center, #1a1a3e 0%, var(--bg) 70%);
  }

  .hero {
    text-align: center;
    max-width: 480px;
    padding: 2rem;
  }

  .castle-art {
    font-size: 6rem;
    line-height: 1;
    margin-bottom: 1rem;
    filter: drop-shadow(0 0 20px rgba(255, 215, 0, 0.4));
    animation: float 4s ease-in-out infinite;
  }

  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-10px); }
  }

  h1 {
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--gold);
    text-shadow: 0 0 20px rgba(255, 215, 0, 0.3);
    margin-bottom: 0.5rem;
  }

  .subtitle {
    color: var(--text-muted);
    font-size: 1.1rem;
    margin-bottom: 1.5rem;
  }

  .tags {
    display: flex;
    gap: 0.5rem;
    justify-content: center;
    margin-bottom: 2rem;
    flex-wrap: wrap;
  }

  .tag {
    background: rgba(255, 215, 0, 0.1);
    color: var(--gold);
    border: 1px solid rgba(255, 215, 0, 0.2);
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 0.875rem;
  }

  .play-btn {
    font-size: 1.25rem;
    padding: 14px 40px;
    border-radius: 50px;
    background: linear-gradient(135deg, var(--accent), #c0392b);
    color: white;
    box-shadow: 0 4px 20px rgba(233, 69, 96, 0.4);
    margin-bottom: 1.5rem;
    transition: box-shadow 200ms, transform 200ms;
  }

  .play-btn:hover {
    box-shadow: 0 6px 30px rgba(233, 69, 96, 0.6);
    transform: translateY(-2px);
  }

  .hint {
    color: var(--text-muted);
    font-size: 0.85rem;
    font-family: var(--font-mono);
  }
</style>
