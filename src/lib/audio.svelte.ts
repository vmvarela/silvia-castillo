/**
 * Servicio de audio — efectos de sonido y música de fondo procedural.
 * Usa Web Audio API sin archivos externos ni dependencias.
 * Extensión .svelte.ts para poder usar $state en la clase.
 */

type ToneOpts = {
  type: OscillatorType;
  freq: number;
  freqEnd?: number;
  duration: number;
  gain?: number;
  delay?: number;
};

class AudioService {
  // Estado reactivo — visible en la UI
  muted = $state(false);
  musicOn = $state(false);

  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;

  // Música de fondo
  private musicRunning = false;
  private musicTimer: ReturnType<typeof setTimeout> | null = null;
  private musicStep = 0;

  // Escala pentatónica menor de La — sonido medieval/ambiente
  private readonly SCALE = [220, 261.63, 293.66, 329.63, 392, 329.63, 293.66, 261.63];

  // ── Contexto lazy ──────────────────────────────────────────────────────────

  private ctx_(): AudioContext {
    if (!this.ctx) {
      this.ctx = new AudioContext();
      this.master = this.ctx.createGain();
      this.master.gain.value = 0.7;
      this.master.connect(this.ctx.destination);
    }
    if (this.ctx.state === 'suspended') this.ctx.resume();
    return this.ctx;
  }

  // ── Generador de tonos ────────────────────────────────────────────────────

  private tone(o: ToneOpts) {
    if (this.muted) return;
    const ctx = this.ctx_();
    const osc = ctx.createOscillator();
    const env = ctx.createGain();
    const t = ctx.currentTime + (o.delay ?? 0);

    osc.type = o.type;
    osc.frequency.setValueAtTime(o.freq, t);
    if (o.freqEnd !== undefined) {
      osc.frequency.linearRampToValueAtTime(o.freqEnd, t + o.duration);
    }

    const g = o.gain ?? 0.25;
    env.gain.setValueAtTime(g, t);
    env.gain.exponentialRampToValueAtTime(0.0001, t + o.duration);

    osc.connect(env);
    env.connect(this.master!);
    osc.start(t);
    osc.stop(t + o.duration + 0.02);
  }

  // ── Efectos de sonido ─────────────────────────────────────────────────────

  /** Tick sutil al pulsar una tecla en el terminal */
  typing() {
    this.tone({ type: 'square', freq: 880, duration: 0.022, gain: 0.032 });
  }

  /** Regla añadida correctamente */
  commandOk() {
    this.tone({ type: 'sine', freq: 440, freqEnd: 660, duration: 0.18, gain: 0.28 });
  }

  /** Error al ejecutar un comando */
  commandError() {
    this.tone({ type: 'sawtooth', freq: 240, freqEnd: 140, duration: 0.22, gain: 0.2 });
  }

  /** Un test individual ha pasado */
  testPass() {
    this.tone({ type: 'sine', freq: 660, freqEnd: 880, duration: 0.13, gain: 0.2 });
  }

  /** Un test individual ha fallado */
  testFail() {
    this.tone({ type: 'triangle', freq: 200, freqEnd: 120, duration: 0.2, gain: 0.18 });
  }

  /** ¡Todos los tests superados! Acorde ascendente de victoria */
  victory() {
    const notas = [261.63, 329.63, 392, 523.25]; // Do mayor
    notas.forEach((freq, i) =>
      this.tone({ type: 'sine', freq, duration: 0.25, gain: 0.32, delay: i * 0.11 })
    );
    // Flourish final
    this.tone({ type: 'sine', freq: 523.25, freqEnd: 1046.5, duration: 0.3, gain: 0.28, delay: 0.52 });
  }

  /** Nivel completado y guardado en el progreso */
  levelComplete() {
    const melodia = [261.63, 329.63, 392, 523.25, 659.25, 783.99, 1046.5];
    melodia.forEach((freq, i) =>
      this.tone({ type: 'sine', freq, duration: 0.22, gain: 0.35, delay: i * 0.13 })
    );
  }

  // ── Música de fondo ───────────────────────────────────────────────────────

  toggleMusic() {
    if (this.musicOn) {
      this.musicOn = false;
      this.musicRunning = false;
      if (this.musicTimer !== null) {
        clearTimeout(this.musicTimer);
        this.musicTimer = null;
      }
    } else {
      this.musicOn = true;
      this.musicRunning = true;
      this.tick();
    }
  }

  private tick() {
    if (!this.musicRunning) return;
    if (!this.muted) {
      const freq = this.SCALE[this.musicStep % this.SCALE.length];
      this.musicStep++;
      this.tone({ type: 'sine', freq, duration: 0.38, gain: 0.08 });
    }
    this.musicTimer = setTimeout(() => this.tick(), 420);
  }

  // ── Control global ────────────────────────────────────────────────────────

  toggleMute() {
    this.muted = !this.muted;
    if (this.master) {
      this.master.gain.value = this.muted ? 0 : 0.7;
    }
  }
}

// Singleton — una sola instancia para toda la app
export const audio = new AudioService();
