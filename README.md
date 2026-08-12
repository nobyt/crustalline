# crustalline

A desktop application for viewing and editing molecular structures, built on
[Tauri](https://tauri.app/) + [3Dmol.js](https://3dmol.csb.pitt.edu/), backed
by [molrs](https://github.com/nobyt/molrs) (SMILES parsing, 3D conformer
generation, 2D depiction, canonicalization) for all molecular data handling.
Supports a headless mode for rendering molecules to PNG with no visible
window.

See [`../.claude/plans/`](/home/tanaka/.claude/plans) session history for the
original architecture plan, and [`docs/`](docs/) for living reference docs:

- [`docs/molrs-api-contract.md`](docs/molrs-api-contract.md) — the mutable
  graph-editing API design (now implemented as `molrs::edit`, `editing`
  feature, and wired into `crates/core`; see the doc for known deviations
  from the original spec).
- [`docs/headless-rendering.md`](docs/headless-rendering.md) — WebKitGTK /
  WebGL / Xvfb setup notes for Linux.

## Development

```
cd crates/app
npx --prefix ../../frontend tauri dev
```

**Linux**: requires GTK/WebKitGTK dev packages (`libwebkit2gtk-4.1-dev`,
`libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`,
`libsoup-3.0-dev`, `libjavascriptcoregtk-4.1-dev`) and, on hosts without a
real display (CI, SSH-forwarded X11), **Xvfb** — see
[`docs/headless-rendering.md`](docs/headless-rendering.md).

## Workspace layout

```
crates/
  app/         Tauri binary — GUI entry + CLI subcommands (crustalline render ...)
  core/        Shared molecule state + edit orchestration (only crate touching molrs directly)
  ipc-types/   Serde DTOs shared between Rust and the frontend
frontend/      Vanilla TS + Vite webview assets (3Dmol.js viewer, vendored under public/vendor/3dmol/)
```

molrs is consumed as a relative path dependency from the sibling
`~/ghq/github.com/nobyt/molrs` repo (not yet published).
