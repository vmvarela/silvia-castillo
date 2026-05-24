<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { executeCommand } from '$lib/tauri/commands';
  import type { ExecuteResult } from '$lib/tauri/types';
  import { audio } from '$lib/audio.svelte';

  // Props
  let { onResult }: { onResult?: (r: ExecuteResult) => void } = $props();

  let container: HTMLDivElement;
  let term: import('@xterm/xterm').Terminal | null = null;
  let fitAddon: import('@xterm/addon-fit').FitAddon | null = null;
  let ro: ResizeObserver | null = null;

  // Input state
  let lineBuffer = '';
  let history: string[] = [];
  let historyIdx = -1;

  const PROMPT = '\r\n\x1b[33m$ \x1b[0m';
  const MAX_HISTORY = 50;

  export function print(text: string) {
    term?.write('\r\n' + text);
  }

  export function clear() {
    term?.clear();
    term?.write(PROMPT);
  }

  async function handleEnter() {
    const input = lineBuffer.trim();
    term?.write('\r\n');

    if (!input) {
      term?.write(PROMPT);
      return;
    }

    // Add to history
    if (history[history.length - 1] !== input) {
      history = [...history.slice(-MAX_HISTORY + 1), input];
    }
    historyIdx = -1;
    lineBuffer = '';

    try {
      const result = await executeCommand(input);
      if (result.ok) {
        if (result.humanize) {
          term?.write('\x1b[32m→ ' + result.humanize + '\x1b[0m');
        }
        audio.commandOk();
      } else {
        const msg = result.error ?? 'Error desconocido';
        term?.write('\x1b[31m✗ ' + msg + '\x1b[0m');
        audio.commandError();
      }
      onResult?.(result);
    } catch (e) {
      term?.write('\x1b[31m✗ Error de conexión con el backend\x1b[0m');
    }

    term?.write(PROMPT);
  }

  function handleKeyDown(data: string, ev: KeyboardEvent) {
    // Up arrow — history back
    if (ev.key === 'ArrowUp') {
      if (history.length === 0) return;
      if (historyIdx === -1) historyIdx = history.length - 1;
      else if (historyIdx > 0) historyIdx--;
      replaceBuffer(history[historyIdx]);
      return;
    }
    // Down arrow — history forward
    if (ev.key === 'ArrowDown') {
      if (historyIdx === -1) return;
      if (historyIdx < history.length - 1) {
        historyIdx++;
        replaceBuffer(history[historyIdx]);
      } else {
        historyIdx = -1;
        replaceBuffer('');
      }
      return;
    }
    // Ctrl+C — cancel line
    if (ev.ctrlKey && ev.key === 'c') {
      term?.write('^C' + PROMPT);
      lineBuffer = '';
      historyIdx = -1;
      return;
    }
    // Ctrl+L — clear
    if (ev.ctrlKey && ev.key === 'l') {
      clear();
      return;
    }
  }

  function replaceBuffer(newVal: string) {
    if (!term) return;
    // Move to start of current input and clear it
    term.write('\x1b[2K\r\x1b[33m$ \x1b[0m');
    term.write(newVal);
    lineBuffer = newVal;
  }

  onMount(async () => {
    const { Terminal } = await import('@xterm/xterm');
    const { FitAddon } = await import('@xterm/addon-fit');
    const { WebLinksAddon } = await import('@xterm/addon-web-links');

    term = new Terminal({
      theme: {
        background: '#0d1117',
        foreground: '#e6c87a',
        cursor: '#ffd700',
        cursorAccent: '#0d1117',
        selectionBackground: 'rgba(255, 215, 0, 0.3)',
        black: '#0d1117',
        brightBlack: '#444',
        red: '#f44336',
        brightRed: '#ef9a9a',
        green: '#4caf50',
        brightGreen: '#a5d6a7',
        yellow: '#ffd700',
        brightYellow: '#fff59d',
        blue: '#42a5f5',
        brightBlue: '#90caf9',
        magenta: '#ce93d8',
        brightMagenta: '#e1bee7',
        cyan: '#26c6da',
        brightCyan: '#80deea',
        white: '#e0e0e0',
        brightWhite: '#ffffff',
      },
      fontFamily: '"Cascadia Code", "Fira Code", Consolas, monospace',
      fontSize: 14,
      lineHeight: 1.5,
      cursorBlink: true,
      cursorStyle: 'block',
      scrollback: 1000,
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.loadAddon(new WebLinksAddon());

    // Esperar a que las fuentes estén cargadas ANTES de que xterm abra el canvas
    // y mida el ancho de glifo. Si se hace después, charWidth queda mal y la
    // línea 1 se corrompe (aparece como '$$$' en WKWebView macOS).
    await document.fonts.ready;
    term.open(container);
    fitAddon.fit();

    // Diferir el banner un tick para que xterm complete la inicialización del canvas.
    // Welcome message — WKWebView macOS no renderiza box-drawing horizontales
    // (U+2500-U+257F): aparecen como '$'. Solución: ASCII puro, sin emoji.
    setTimeout(() => {
      if (!term) return;
      // reset() borra los artefactos de medición que xterm escribe en línea 0 durante open()
      term.reset();
      term.writeln('\x1b[33m  +--------------------------------------+\x1b[0m');
      term.writeln('\x1b[1;33m  |       Castillo de Silvia            |\x1b[0m');
      term.writeln('\x1b[33m  +--------------------------------------+\x1b[0m');
      term.writeln('\x1b[90mEscribe comandos iptables y pulsa Enter.\x1b[0m');
      term.writeln('\x1b[90mCtrl+C cancelar  Ctrl+L limpiar  flechas historial\x1b[0m');
      term.write(PROMPT);
    }, 0);

    // Key handler
    term.onKey(({ key, domEvent }) => {
      handleKeyDown(key, domEvent);

      if (domEvent.key === 'Enter') {
        handleEnter();
        return;
      }
      if (domEvent.key === 'Backspace') {
        if (lineBuffer.length > 0) {
          lineBuffer = lineBuffer.slice(0, -1);
          term?.write('\b \b');
        }
        return;
      }
      // Ignore control keys
      if (domEvent.ctrlKey || domEvent.altKey || domEvent.metaKey) return;
      if (domEvent.key.length > 1) return; // Arrow keys etc already handled

      lineBuffer += key;
      term?.write(key);
      audio.typing();
    });

    // Resize observer
    ro = new ResizeObserver(() => fitAddon?.fit());
    ro.observe(container);

    term.focus();
  });

  onDestroy(() => {
    ro?.disconnect();
    term?.dispose();
  });
</script>

<div class="console-wrapper" bind:this={container}></div>

<style>
  .console-wrapper {
    width: 100%;
    height: 100%;
    background: #0d1117;
    border-radius: 8px;
    overflow: hidden;
  }

  /* xterm.js needs these to fill the container */
  :global(.xterm) {
    height: 100%;
    padding: 8px;
  }

  :global(.xterm-viewport) {
    border-radius: 8px;
  }
</style>
