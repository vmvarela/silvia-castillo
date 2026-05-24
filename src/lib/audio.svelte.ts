/**
 * Servicio de audio — efectos de sonido y música medieval procedural.
 * Usa Web Audio API sin archivos externos ni dependencias.
 *
 * Lookahead scheduling: las notas se programan 350 ms por delante usando
 * AudioContext.currentTime (reloj preciso del motor de audio), no setTimeout.
 * Así el tempo es estable aunque la ventana pierda el foco y el navegador
 * ralentice los timers de JS a ~1 s.
 */

type ToneOpts = {
  type: OscillatorType;
  freq: number;
  freqEnd?: number;
  duration: number;
  gain?: number;
  delay?: number;
};

// [frecuencia Hz, duración en tiempos] — freq = 0 → silencio
type MelodyNote = [number, number];

class AudioService {
  // Estado reactivo — visible en la UI
  muted = $state(false);
  musicOn = $state(false);

  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;

  private musicRunning = false;
  private schedulerTimer: ReturnType<typeof setInterval> | null = null;
  private nextMelodyTime = 0;
  private nextBassTime = 0;
  private melodyStep = 0;
  private bassStep = 0;

  private readonly BEAT = 0.52;      // s por tiempo (~115 BPM)
  private readonly LOOKAHEAD = 0.35; // s que pre-programamos por delante
  private readonly TICK_MS = 50;     // ms entre comprobaciones del scheduler

  // ── Melodía: motivo inspirado en Game of Thrones (G menor Dorian) ─────────
  // G3–C4–Eb4–F4–G4–C4–Eb4–Bb3 / G3–C4–Eb4–F4–Eb4–D4–C4 / silencio
  private readonly MELODY: MelodyNote[] = [
    // Frase 1
    [196.00, 1], // G3
    [261.63, 1], // C4
    [311.13, 2], // Eb4  (largo)
    [349.23, 1], // F4
    [392.00, 1], // G4
    [261.63, 1], // C4
    [311.13, 1], // Eb4
    [233.08, 2], // Bb3  (largo)
    // Frase 2
    [196.00, 1], // G3
    [261.63, 1], // C4
    [311.13, 2], // Eb4
    [349.23, 1], // F4
    [311.13, 1], // Eb4
    [293.66, 1], // D4
    [261.63, 2], // C4
    // Silencio — respiración entre frases
    [0.00,   4],
  ];
  // 23 tiempos por ciclo

  // ── Bajo: dron grave en G menor ───────────────────────────────────────────
  private readonly BASS: MelodyNote[] = [
    [98.00,  8], // G2
    [130.81, 8], // C3
    [116.54, 4], // Bb2
    [98.00,  4], // G2
  ];
  // 24 tiempos por ciclo — primo con melodía (23) → variedad natural sin bucle exacto

  // ── Contexto lazy ─────────────────────────────────────────────────────────

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

  // ── Tone helper para efectos de sonido ────────────────────────────────────

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

  // ── Instrumentos medievales ───────────────────────────────────────────────

  /** Viella / laúd: sawtooth + filtro + envelope de cuerda pulsada */
  private scheduleString(freq: number, t: number, dur: number) {
    if (!this.ctx || !this.master || freq <= 0) return;
    const ctx = this.ctx;
    const master = this.master;

    // Oscilador principal (cuerpo)
    const osc = ctx.createOscillator();
    osc.type = 'sawtooth';
    osc.frequency.setValueAtTime(freq, t);

    // Armónico de octava (brillo inicial)
    const osc2 = ctx.createOscillator();
    osc2.type = 'triangle';
    osc2.frequency.setValueAtTime(freq * 2, t);

    // Filtro paso-bajo → efecto caja de resonancia
    const filter = ctx.createBiquadFilter();
    filter.type = 'lowpass';
    filter.frequency.setValueAtTime(freq * 4.5, t);
    filter.Q.value = 1.2;

    // Envelope: ataque breve, caída suave
    const env = ctx.createGain();
    env.gain.setValueAtTime(0.001, t);
    env.gain.linearRampToValueAtTime(0.2, t + 0.04);
    env.gain.exponentialRampToValueAtTime(0.09, t + dur * 0.45);
    env.gain.exponentialRampToValueAtTime(0.001, t + dur);

    // Armónico con decaimiento rápido
    const env2 = ctx.createGain();
    env2.gain.setValueAtTime(0.07, t);
    env2.gain.exponentialRampToValueAtTime(0.001, t + dur * 0.25);

    osc.connect(filter);
    filter.connect(env);
    env.connect(master);
    osc2.connect(env2);
    env2.connect(master);

    osc.start(t);
    osc.stop(t + dur + 0.05);
    osc2.start(t);
    osc2.stop(t + dur * 0.28);
  }

  /** Bajo de órgano: sine + sub-octava + decaimiento lento */
  private scheduleBass(freq: number, t: number, dur: number) {
    if (!this.ctx || !this.master || freq <= 0) return;
    const ctx = this.ctx;
    const master = this.master;

    const osc = ctx.createOscillator();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(freq, t);

    const sub = ctx.createOscillator();
    sub.type = 'triangle';
    sub.frequency.setValueAtTime(freq * 0.5, t); // sub-octava

    const env = ctx.createGain();
    env.gain.setValueAtTime(0.001, t);
    env.gain.linearRampToValueAtTime(0.14, t + 0.12);
    env.gain.exponentialRampToValueAtTime(0.07, t + dur * 0.6);
    env.gain.exponentialRampToValueAtTime(0.001, t + dur);

    osc.connect(env);
    sub.connect(env);
    env.connect(master);

    osc.start(t);
    osc.stop(t + dur + 0.05);
    sub.start(t);
    sub.stop(t + dur + 0.05);
  }

  // ── Lookahead scheduler ───────────────────────────────────────────────────

  private scheduleAhead() {
    if (!this.ctx) return;
    if (this.ctx.state === 'suspended') this.ctx.resume();
    const horizon = this.ctx.currentTime + this.LOOKAHEAD;

    while (this.nextMelodyTime < horizon) {
      const [freq, beats] = this.MELODY[this.melodyStep % this.MELODY.length];
      this.scheduleString(freq, this.nextMelodyTime, beats * this.BEAT * 0.82);
      this.melodyStep++;
      this.nextMelodyTime += beats * this.BEAT;
    }

    while (this.nextBassTime < horizon) {
      const [freq, beats] = this.BASS[this.bassStep % this.BASS.length];
      this.scheduleBass(freq, this.nextBassTime, beats * this.BEAT * 0.9);
      this.bassStep++;
      this.nextBassTime += beats * this.BEAT;
    }
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
    this.tone({ type: 'sine', freq: 523.25, freqEnd: 1046.5, duration: 0.3, gain: 0.28, delay: 0.52 });
  }

  /** Nivel completado y guardado en el progreso */
  levelComplete() {
    const melodia = [261.63, 329.63, 392, 523.25, 659.25, 783.99, 1046.5];
    melodia.forEach((freq, i) =>
      this.tone({ type: 'sine', freq, duration: 0.22, gain: 0.35, delay: i * 0.13 })
    );
  }

  // ── Controles globales ────────────────────────────────────────────────────

  toggleMusic() {
    if (this.musicOn) {
      this.musicOn = false;
      this.musicRunning = false;
      if (this.schedulerTimer !== null) {
        clearInterval(this.schedulerTimer);
        this.schedulerTimer = null;
      }
    } else {
      this.musicOn = true;
      this.musicRunning = true;
      const ctx = this.ctx_();
      // Inicializar tiempos desde ahora + 100 ms
      this.nextMelodyTime = ctx.currentTime + 0.1;
      this.nextBassTime = ctx.currentTime + 0.1;
      this.melodyStep = 0;
      this.bassStep = 0;
      // Pre-programar primer bloque y arrancar el scheduler
      this.scheduleAhead();
      this.schedulerTimer = setInterval(() => {
        if (!this.musicRunning) return;
        this.scheduleAhead();
      }, this.TICK_MS);
    }
  }

  toggleMute() {
    this.muted = !this.muted;
    if (this.master) {
      this.master.gain.value = this.muted ? 0 : 0.7;
    }
  }
}

// Singleton — una sola instancia para toda la app
export const audio = new AudioService();
