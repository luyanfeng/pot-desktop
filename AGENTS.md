# AGENTS.md

Pot: cross-platform selection-translation / OCR desktop app. Tauri 2 (migrated from 1.8) + React 18 + Vite 5, frontend in JSX (`.jsx`), backend in Rust. Package manager is **pnpm** (never npm/yarn).

## Commands

- `pnpm tauri dev` — run app in dev mode. Vite dev server uses `http://localhost:1420` with `strictPort`; port conflicts hard-fail.
- `pnpm tauri build` — full bundle build (slow; Rust release compile).
- `pnpm dev` / `pnpm build` — frontend only (Vite).
- `pnpm prettier --write .` — the only formatter. There are **no test, lint, or typecheck scripts** — verify frontend changes by building (`pnpm build`) and Rust changes with `cargo check`/`cargo build` in `src-tauri/`.
- Linux dev prerequisites (apt): `libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf libxdo-dev libxcb1 libxrandr2 libdbus-1-3 libssl-dev` (see README).
- Platform-confined Rust deps are behind `#[cfg(target_os = ...)]` in `src-tauri/Cargo.toml`; `src-tauri/src/system_ocr.rs` and `src-tauri/src/screenshot.rs` compile different code per OS.

## Versioning quirk (do not hand-edit casually)

- `package.json` and `src-tauri/tauri.conf.json` say `3.0.7`; `src-tauri/Cargo.toml` says `0.0.0`. This is intentional: `.github/workflows/package.yml` rewrites all three from `git describe --tags` on release. Do not "fix" the mismatch.
- `tauri.conf.json` `plugins.updater.pubkey` must stay in sync with the signing key used in CI (`TAURI_PRIVATE_KEY` secrets).
- Rust tauri stack: `tauri 2.11.5` + `tauri-runtime-wry 2.11.4` + `wry 0.55`(依赖经 `cargo update` 全量解析,与 npm 的 `@tauri-apps/api@2.11.x` 对齐)。**注意 Tauri 要求 Rust crate 与 npm 包的 minor 版本一致**,否则报 "Found version mismatched Tauri packages" 且 IPC 异常(窗口/托盘失灵)。不要单独 `cargo update -p <crate> --precise` 偏离这套矩阵。
- `zbus` must stay on 5.15+ (5.12.0 has a broken `#[interface]` macro that fails with `DispatchResult2` not found); currently 5.19.
- Linux 编译还需 `libssl-dev`(openssl-sys 依赖)。

## Architecture

- **Single webview, many windows**: `src/App.jsx` renders `windowMap[appWindow.label]` — window type is decided by the Rust side via `appWindow.label` (`translate`, `screenshot`, `recognize`, `config`, `updater`). The hidden `daemon` window (`daemon.html`) has no React app. Frontend uses `getCurrentWebviewWindow()` from `@tauri-apps/api/webviewWindow`.
- **Main process**: `src-tauri/src/main.rs` wires everything — global shortcuts, tray, HTTP server, clipboard monitor, first-run config window, update check. On macOS it needs Accessibility permission for selection translation.
- **Config**: JSON file at `<config_dir>/com.pot-app.desktop/config.json`. Two independent Stores read/write it: Rust (`src-tauri/src/config.rs` `StoreWrapper(Mutex<Store<Wry>>)`, sync, `get`/`set` helpers) and frontend (`src/utils/store.js` `Store` + fs-watch, async). When adding a config key, wire both sides; `main.rs` setup and `reload_store` invoke both consult it.
- **HTTP API**: `src-tauri/src/server.rs` runs `tiny_http` on port 60828 (config `server_port`). Docs in README's 外部调用 section. Response is always `ok`; errors are not returned to callers.
- **Plugin system**: `.potext` plugins are unzipped into `<config_dir>/com.pot-app.desktop/plugins/<type>/<name>` and loaded via `eval()` in `src/utils/invoke_plugin.js`. Service registry: `src/services/translate/index.jsx`, `src/services/recognize/index.jsx`, `src/services/tts/index.jsx`, `src/services/collection/index.jsx`, each re-exporting one dir per provider. `src-tauri/src/config.rs::check_service_available` keeps hardcoded lists of builtin provider names in sync with those dirs — adding a builtin provider means editing both places. Service keys are `name@randomId` (`src/utils/service_instance.ts`).
- **i18n**: i18next with locale JSON bundled in `src/i18n/locales/` (managed via Weblate). New UI strings: add `t('key')` calls; locale files get filled by translators, don't hand-edit all of them.

## Tauri 2 migration specifics (verified working)

- **Tray**: built in Rust via `TrayIconBuilder` + `tauri::menu::{Menu, MenuItem, CheckMenuItem}` in `tray.rs`. The `TrayIcon` and `CheckMenuItem` handles are held in `OnceCell` statics (dropping them makes the tray disappear). There is no `get_item(id)` — keep item references to call `set_checked`.
- **Global shortcuts**: use `tauri-plugin-global-shortcut` (core API was removed in v2). Registration is by string via `GlobalShortcutExt::global_shortcut().register("ctrl+shift+y")`.
- **Updater**: `tauri-plugin-updater`. Rust side uses `app.updater()?.check().await`; frontend uses `@tauri-apps/plugin-updater` `check()` → `update.downloadAndInstall(onEvent)` then `relaunch()` from `@tauri-apps/plugin-process`. The old `installUpdate()` / `tauri://update-download-progress` event no longer exist — progress comes from the `downloadAndInstall` callback.
- **Notifications**: `tauri-plugin-notification`, `app.notification().builder().title().body().show()`.
- **HTTP from frontend**: `@tauri-apps/plugin-http` uses native `fetch` — there is **no `Body` class** anymore. Use `JSON.stringify(x)` + `Content-Type: application/json`, `new URLSearchParams(x).toString()` for forms, and read responses with `await res.json()` / `await res.text()` (no `res.data`). `fetch(url, { query })` is gone — append `?` + `URLSearchParams` to the URL.
- **fs**: `@tauri-apps/plugin-fs`. `readBinaryFile` → `readFile` (returns `Uint8Array`), `removeDir` → `remove` with `baseDir`/`recursive` options.
- **Capabilities**: permissions live in `src-tauri/capabilities/*.json` (v1 `allowlist` is gone). The migrated file lists all six window labels (`daemon`/`translate`/`screenshot`/`recognize`/`config`/`updater`) with `core:default` + plugin `:default` sets. If you add a plugin or window, update this file or frontend IPC will be denied.
- **Platform conf files** (`tauri.linux.conf.json` etc): use `app.trayIcon` (not `tauri.systemTray`).
- **Store** (`tauri-plugin-store`): use `Store.load(path)` (async static), **not `new Store()`**; the instance reload method is `reload()` (there is no `load()`). `store.get(key)` returns `undefined` for missing keys (not `null`) — guard with `== null`.
- **Global shortcut matching** (`hotkey.rs`): the event's `Shortcut` `Display` format (`control+Digit6`) differs from the stored config string (`Ctrl+6`). Parse the config string to `Shortcut` and compare by value, don't string-compare.
- **Tray icon**: `TrayIconBuilder` needs an explicit `.icon(app.default_window_icon().unwrap().clone())` — without it the tray shows a placeholder.
- **fs option name**: pass `{ baseDir: BaseDirectory.AppConfig }`, **not** `{ dir: ... }` (v1 name) — a wrong name silently skips path resolution and fails scope checks with "forbidden path".

## Capabilities permission gotchas (v2 runtime denials are silent-ish, check logs)

- `fs:default` does **not** include `watch` — add `fs:allow-watch` + enable the `watch` cargo feature on `tauri-plugin-fs`.
- `http:default` enables fetch ops but **allows no URLs** — add an object `{ "identifier": "http:default", "allow": [{ "url": "*://*:*" }] }` (URL Pattern syntax; `https://*` misses non-default ports like Ollama's 11434).
- `core:window:default` has no `show`/`hide`/`close`/drag — add `core:window:allow-show`, `allow-start-dragging`, etc. as used.
- `global-shortcut:default` lacks `register`/`unregister`/`is-registered` — add the `allow-*` variants.
- `sql:default` lacks `execute`/`select`/`load`/`close` — add them if the frontend uses `@tauri-apps/plugin-sql`.
- Capabilities are compiled into the app at build time — after editing `capabilities/*.json` you must fully restart `pnpm tauri dev` (a lingering old process serves stale capabilities).

## Conventions

- Prettier config (`.prettierrc.json`): 4-space indent, single quotes, `jsxSingleQuote`, printWidth 120, semicolons. **This repo does not use 2-space JS style** — match the existing 4-space formatting or `pnpm prettier --write .` at the end.
- Files are `.jsx`/`.js` (JavaScript, not TypeScript) except a few `.ts` helpers (`service_instance.ts`, `language.ts`, provider `info.ts`). New provider metadata goes in `info.ts` following existing ones.
- UI uses NextUI + Tailwind with custom theme in `tailwind.config.cjs` (dark mode via `class`).
- The tray keeps the app alive: `main.rs` prevents exit on window close (`RunEvent::ExitRequested` + `api.prevent_exit()`); closing windows hides the app, tray menu quits it. App must be single-instance (plugin enforces this).
- CSP is loose (`script-src * 'unsafe-eval'`) — plugins require `eval`; don't tighten it without breaking the plugin system.

## Gotchas

- Frontend-only build (`pnpm build`) produces `dist/` but cannot be tested in a browser — all Tauri APIs (`@tauri-apps/api`, plugin APIs) need the desktop runtime. Use `pnpm tauri dev` for real verification.
- `src-tauri/src/window.rs` manages which windows exist; windows are created by label from Rust (`WebviewWindowBuilder`). The frontend's `windowMap` must contain every label the Rust side can open.
- Keep `Cargo.lock` updated after Cargo.toml changes (`cargo check` in `src-tauri/`), but only with `--precise` pins — see the versioning quirk above.
- The release build runs `pnpm prettier --write .` before packaging; unformatted files get auto-fixed and could produce dirty diffs.
- `patches/hyprland.patch` is an optional Wayland workaround, not part of normal builds.
- `zbus` must stay on 5.15+ (5.12.0 has a broken `#[interface]` macro that fails with `DispatchResult2` not found).
