# 🏰 Castillo de Silvia

Aprende iptables protegiendo el castillo de la princesa Silvia de mensajeros peligrosos.

## Qué es esto

Un juego de escritorio (Mac / Linux / Windows) donde escribes comandos `iptables` reales en una consola para superar 8 misiones. Cada nivel tiene una historia, unos patitos que necesitan protección, y unos guardias (cadenas INPUT/OUTPUT/FORWARD) que obedecen tus reglas.

## Descargar

[Releases en GitHub →](https://github.com/tu-usuario/silvia-castillo/releases)

| Sistema | Archivo |
|---------|---------|
| macOS | `.dmg` |
| Linux | `.AppImage` |
| Windows | `.msi` |

> Primera vez en macOS: botón derecho → Abrir (app sin firma).  
> Primera vez en Windows: "Más información" → "Ejecutar de todas formas".

## Desarrollo

### Requisitos

- [Rust](https://rustup.rs/) 1.77+
- Node 20+
- En macOS: Xcode Command Line Tools (`xcode-select --install`)
- En Linux: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
- En Windows: [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### Arrancar en modo dev

```bash
npm install
npm run tauri dev
```

### Compilar para distribución

```bash
npm run tauri build
```

### Tests

```bash
# Tests Rust (parser + engine)
cd src-tauri && cargo test

# Comprobación de tipos Svelte
npm run check

# Regenerar tipos TypeScript desde Rust
npm run sync-types
```

## Arquitectura

```
silvia-castillo/
├── src-tauri/          Backend Rust (Tauri)
│   └── src/
│       ├── engine/     Parser iptables + ruleset + pipeline
│       ├── levels/     Niveles YAML embebidos
│       ├── translate/  Humanizador en español
│       ├── progress/   Persistencia JSON local
│       ├── commands.rs Comandos Tauri (#[tauri::command])
│       └── state.rs    AppState (Mutex<Ruleset>)
└── src/                Frontend SvelteKit (SPA)
    ├── lib/
    │   ├── tauri/      Wrappers invoke() tipados
    │   ├── stores/     Estado reactivo Svelte 5
    │   └── components/ Castle, NetworkDiagram, Console, ...
    └── routes/
        ├── +page.svelte        Menú principal
        └── level/[n]/          Pantalla de nivel
```

El frontend nunca toca el ruleset directamente: todo cambio va a través de `invoke('execute_command')` y el backend Rust devuelve el estado nuevo. Cero drift entre motor y UI.

## Niveles

Los niveles son ficheros YAML en `src-tauri/src/levels/data/`. Para añadir uno, crea `NN-nombre.yaml` — sin codegen ni registro manual.

## Licencia

MIT
