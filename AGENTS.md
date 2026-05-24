# AGENTS.md — silvia-castillo

App de escritorio Tauri v2 + SvelteKit SPA + Rust. El jugador escribe comandos `iptables` reales en una terminal xterm.js; el motor Netfilter (Rust) los evalúa contra 9 niveles YAML.

## Developer commands

```bash
# Ejecutar en modo desarrollo (OBLIGATORIO — no abrir localhost en navegador)
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk npx tauri dev

# Tests Rust (desde src-tauri/)
cd src-tauri && cargo test

# Typecheck Svelte (0 errores requeridos)
npm run check

# Formateo y lint Rust (CI los exige)
cd src-tauri && cargo fmt --all
cd src-tauri && cargo clippy --all-targets -- -D warnings

# Regenerar bindings TypeScript tras cambiar tipos Rust con #[ts(export)]
npm run sync-types          # ejecuta cargo test, que escribe src-tauri/bindings/

# Build distribución
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk npx tauri build
```

## Arquitectura

```
src-tauri/src/
  lib.rs            entrypoint Tauri + registro de 8 comandos
  commands.rs       comandos Tauri + tipos exportados (#[ts(export)])
  state.rs          AppState: Mutex<Ruleset + Level + Topology + Progress>
  engine/           motor Netfilter: parser (winnow), ruleset, pipeline, matchers,
                    topology, packet, trace, ast
  levels/           Level struct + 9 YAML embebidos con include_str!
    data/*.yaml     01-observar … 09-examen-silvia (orden = índice)
  progress/         FileStore: JSON atómico → ~/.silvia-castillo.json
  solutions_test.rs soluciones canónicas para los 9 niveles (end-to-end)

src/
  routes/
    +page.svelte          mapa de selección de niveles
    level/[n]/+page.svelte  experiencia completa: historia · consola · tests
    +layout.ts            ssr=false, prerender=true
    level/[n]/+page.ts    prerender=false (ruta dinámica)
  lib/tauri/
    commands.ts     wrappers invoke (todos protegidos con requireTauri())
    types.ts        tipos TS (no editar los de bindings/, sí los de aquí)
  lib/components/console/Console.svelte  terminal xterm.js
src-tauri/bindings/   ← GENERADO por cargo test. No editar a mano.
```

**Regla de estado:** todo el estado autoritativo vive en Rust (`AppState`). El frontend solo refleja lo que le devuelve un comando.

## Netfilter pipeline

`engine/pipeline.rs`: `nat/PREROUTING` → routing → `filter/INPUT` o `FORWARD` → `nat/POSTROUTING`. Tráfico con `src_ip` en `local_ips` → `filter/OUTPUT` → `nat/POSTROUTING`.

El routing decide INPUT vs FORWARD comprobando si `dst_ip` está en `topology.local_ips`. Eso incluye la `firewall_ip` **y todas las IPs de interfaz** (`LevelInterface.ip`). Si falta alguna IP de interfaz en `local_ips`, los paquetes a esa IP se enrutan a FORWARD en lugar de INPUT.

## Convenciones

- **Texto en español:** campos YAML, mensajes de error, UI, comentarios de código.
- **Svelte 5 runes:** `$state`, `$derived`, `$props`, `onclick=` — nunca `on:click`.
- **ts-rs:** añadir `#[derive(TS)] #[ts(export)]` a cualquier tipo nuevo en Rust que deba llegar al frontend. Luego `npm run sync-types` y copiar/actualizar en `types.ts`.
- Los YAML de niveles usan campos en español (`titulo`, `cuento`, `mision`, `pruebas`, `reglas_iniciales`, etc.).

## Añadir un nivel

1. Crear `src-tauri/src/levels/data/NN-nombre.yaml` (el orden de nombre de archivo = índice).
2. Añadir `include_str!("data/NN-nombre.yaml")` al array `LEVEL_YAMLS` en `levels/mod.rs`.
3. Añadir la solución canónica en `solutions_test.rs` → función `solution()`.
4. `cargo test` debe pasar.

## Lessons learned

- **`npx tauri dev`, nunca `npm run dev` + navegador.** `window.__TAURI_INTERNALS__` solo existe en el webview nativo. En el navegador, `invoke` falla con `Cannot read properties of undefined`. Todos los `invoke` están envueltos en `requireTauri()` (commands.ts) para dar un error legible.
- **`SDKROOT` obligatorio en macOS** para compilar Rust: `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk`.
- **`LevelInterface.ip` ≠ `firewall_ip`.** Cada interfaz puede tener su propia IP (ej. eth2 = `203.0.113.1`). `topology_from_level` añade todas las IPs de interfaz a `local_ips`; no hacerlo rompe el routing de INPUT en el nivel 9 (REJECT ICMP desde WAN llega a FORWARD en lugar de INPUT).
- **`solutions_test.rs` es la verdad.** Está registrado con `#[cfg(test)] mod solutions_test;` en `lib.rs`. Si cambia la lógica del pipeline o de la topología, actualizar también los tests.
- **Bindings generados.** `src-tauri/bindings/*.ts` se sobreescriben en cada `cargo test`. Los tipos que el frontend usa directamente van en `src/lib/tauri/types.ts` (editable); los bindings son solo la fuente de referencia.
- **`adapter-static` + `ssr=false`.** SvelteKit genera un SPA puro; no hay SSR. Las rutas dinámicas como `level/[n]` requieren `prerender=false` en su `+page.ts`.
