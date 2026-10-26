# Helio (`helio`): Terminal Minecraft Launcher, Agent Prompt in 4 Phases

## How to use this file

1. Paste **STANDING ORDERS** into the agent once, at the very start.
2. Paste **Phase 1**. When the agent's report shows every Done item passing with evidence, paste **Phase 2**, then 3, then 4. Do not let it skip ahead.
3. If the agent loses context, paste the **RESUME prompt** at the bottom, then the current phase.

| Phase | Name | Gate |
|---|---|---|
| 1 | Foundation, CLI skeleton, accounts (login) | `helio login` works for Ely.by and offline; keychain; JSON output |
| 2 | Engine: download, Java, instances, launch, process control | `helio launch` runs vanilla 1.20.1 and 1.8.9; `helio ps`, `helio kill`, `helio logs` work |
| 3 | Loaders, Modrinth, content management | Fabric/Quilt/Forge/NeoForge launch; mods and modpacks install from the CLI |
| 4 | The TUI (`helio` with no args), mouse, polish, release | Everything above doable from the mouse-driven TUI; tests, CI, docs |

---

# STANDING ORDERS (paste once)

You are a senior Rust engineer who knows Minecraft launcher internals, terminal UIs and CLI design. You are building **Helio**: a Minecraft launcher where **everything** is done from the terminal through one command, `helio`.

- `helio` with no arguments opens a full-screen **TUI** with **mouse support** (click, scroll, hover, right-click menus, drag) where you can manage accounts, instances, mods, modpacks, resource packs, shaders, worlds, logs, and launch the game.
- Every action is also a **scriptable subcommand**: `helio launch my-pack`, `helio mod add sodium`, `helio login --offline Steve`, etc.
- Both use the **same engine**. The TUI never contains logic that the CLI cannot do, and vice versa.

## Architecture (fixed)

Cargo workspace:
```
helio/
  Cargo.toml                 # [workspace]
  crates/
    helio-core/                 # library: ALL logic, no printing, no terminal code
    helio/                      # binary named "helio": clap commands + TUI
  tests/ fixtures/ docs/
  .github/workflows/
```
`helio-core` exposes an async API plus a progress/event channel (`tokio::sync::broadcast` or `mpsc`) so any front end (CLI progress bars, TUI, a future GUI) can render progress. `helio-core` must not call `println!`; it returns data and emits events.

## Branding: the project is called **Helio**

This product is **Helio** (command: `helio`). It is a clean rebrand: no trace of any previous name may exist anywhere (code, strings, folders, env vars, keychain entries, log names, CI artifacts, installers, docs, comments, commit messages).

- Centralize identity in one module `helio-core::brand`: `NAME = "Helio"`, `BIN = "helio"`, `DATA_DIR_NAME = "Helio"`, `ENV_PREFIX = "HELIO_"`, `KEYCHAIN_SERVICE = "Helio"`, `USER_AGENT = "Helio/<version> (+<repo url>)"`, `REPO_URL` (read from `git remote get-url origin` in `build.rs`, else from `Cargo.toml` `repository`; **never invent a URL**), `TAGLINE`. Every user-visible string, folder, env var and log name derives from these constants.
- **Visual identity (sun theme; "helio" means "of the sun"):** primary accent warm amber, secondary orange, on a deep charcoal-navy background; the light theme sits on warm cream. Define semantic tokens in one `theme.rs` (`accent`, `accent_hover`, `accent_dim`, `bg`, `bg_elev`, `border`, `text`, `text_dim`, `success`, `warn`, `error`, `info`, `grid`, `grid_hot`) and ship three named palettes: **Sun** (default), **Midnight**, **Daylight**, switchable with `helio config set theme <name>`. Every palette needs truecolor, 256-color and 16-color variants chosen automatically from the terminal.
- **Banner and splash:** a small sun glyph plus a `HELIO` wordmark built from box-drawing or half-block characters (with an ASCII version for `--ascii`). Shown by `helio version --banner`, by the TUI splash for about 400 ms on a cold start, and at the top of the login screen. Set the terminal title to `Helio` (OSC 0/2) and restore it on exit.
- **Name check (Phase 1):** use web search to verify whether `helio` is available on crates.io, GitHub and winget, and whether any common tool named `helio` could shadow the binary. Report conflicts. If the crates.io name is taken, name the packages `helio-launcher` and `helio-launcher-core` while the **binary stays `helio`**; record the decision in `PROGRESS.md`.
- The Ely.by OAuth application is registered under the name **Helio**.
- There is no compatibility requirement with any earlier launcher or data folder. Start clean.

## Data layout

Single data root `Helio/` (default: platform data dir; override with `--data-dir`, `HELIO_DATA_DIR` or config). Never use `.minecraft`.
```
Helio/
  config.json  accounts.json  logs/helio.log
  cache/  runtime/(java/, authlib-injector.jar, run/, secrets.bin, .key)
  libraries/  assets/(indexes, objects, virtual)  versions/
  instances/<slug>/ .helio.json mods/ resourcepacks/ shaderpacks/ saves/ logs/ screenshots/ config/ backups/ icon.png
```

## Icon system (Helio is icon-heavy by design)

Icons are part of Helio's identity in **both** the plain CLI output and the TUI. Rule: every list row, table header, tab, sidebar entry, menu item, dialog button, toast, status line, badge, empty state, prompt and `--help` section heading carries an icon. The only exceptions are `--json` output and plain output when stdout is not a TTY.

- **One registry, `icons.rs`:** a semantic `Icon` enum with **at least 100** distinct icons, for example: `Home, Browse, Add, Avatar, Resources, Settings, Account, Elyby, OfflineAccount, Instance, Play, Stop, Kill, Logs, Folder, Mods, ResourcePacks, Shaders, Worlds, Screenshots, DataPacks, Java, Memory, Cpu, Disk, Download, Upload, Update, Search, Filter, Sort, Delete, Duplicate, Export, Import, Edit, Rename, Copy, Check, Warn, Error, Info, Network, NoNetwork, Lock, Key, Vanilla, Fabric, Quilt, Forge, NeoForge, Modpack, Version, Snapshot, Release, Clock, Playtime, Calendar, Star, Dependency, Cape, Skin, Sun, Help, Palette, Terminal, Command, Link, ExternalLink, Pin, Running, Stopped, Crashed, Pause, Refresh, Cancel, Close, ChevronRight, ChevronDown, Collapse, Expand, Toggle, Tag, Author, License, Gallery, Jar, Zip, Config, Theme, Keyboard, Mouse, Palette`. Each maps to three renderings:
  1. **Nerd Font glyph** (choose codepoints from the Nerd Fonts cheat sheet and verify each one renders).
  2. **Unicode fallback:** single-width characters present in common monospace fonts (geometric shapes, arrows, check marks, box drawing).
  3. **ASCII fallback** (`[+]`, `>`, `*`, `x`, ...).
- **Modes:** `nerd`, `unicode`, `ascii`. Selected by `--icons <mode>`, `--ascii`, `HELIO_ICONS`, config `icons`, or the first-run check. **No emoji code points in any mode** (they render with inconsistent width and color). Add a test that scans the registry for emoji ranges and fails if any are found.
- **Width correctness:** Nerd Font glyphs live in the private use area and have ambiguous width. Always render `glyph + one space` (configurable `icon_pad`), measure with `unicode-width` plus an override table, and test alignment of icons inside table cells, truncated names, right-aligned columns and bordered panels. An icon must never break a border or shift a column.
- **Color:** each icon has a semantic theme color (success, warn, error, info, accent for brand and primary actions, dim for secondary). Output must stay understandable without color (the glyph shape or fallback text carries the meaning).
- **Loaders and account types** have no official Nerd Font logos: assign each a distinctive glyph and a fixed color (Vanilla, Fabric, Quilt, Forge, NeoForge; Ely.by, Offline) and use them identically everywhere.
- **First-run icon check:** on first run (and via `helio doctor --icons`), print a row of sample Nerd Font glyphs and ask "Do you see icons, not boxes or question marks? [Y/n]". Store the result (`icons = nerd` or `unicode`). If the answer is no, print how to install a Nerd Font and select it in the terminal (Windows: verify the current winget package id for a Nerd Font such as JetBrainsMono, then Windows Terminal Settings, Profile, Appearance, Font face; Linux and macOS: package manager or the Nerd Fonts download page), and continue in `unicode` mode in the meantime. Never download or install fonts silently.
- **`helio icons`:** prints the whole registry as a grid (name, nerd, unicode, ascii, color); `--mode` previews one mode; `--json` for tooling. This doubles as a visual test for the user.
- **Coverage is enforced by the type system:** every `Action`, `Tab`, `Screen`, `MenuItem`, `Status`, `Loader` and `AccountKind` enum variant must map to an icon through an exhaustive `match`, plus a unit test. The Phase 4 report lists every screen with every element and its icon.

## Feature parity: Helio does everything a full launcher GUI does

| Capability | CLI | TUI |
|---|---|---|
| Ely.by OAuth login, offline login, multiple accounts, switching | `helio login`, `helio account ...` | login screen, account picker at the bottom of the sidebar |
| Skin and cape viewing (3D-style turntable) | `helio skin` | Avatar screen |
| Instances: create, rename, duplicate, delete, export, import, icons | `helio instance ...` | Home, New-instance wizard, instance menu |
| Versions and loaders (Vanilla, Fabric, Quilt, Forge, NeoForge) | `helio versions`, `helio loader versions`, `helio instance create --loader` | wizard, loader cards |
| Launch with progress, Kill, running state, logs, crash detection | `helio launch`, `kill`, `ps`, `logs` | Launch/Kill button, Logs tab, running dots |
| Per-instance and global settings (Java, RAM, JVM args, resolution) | `helio instance set`, `helio config` | instance Settings tab, Settings screen |
| Java detection, install, selection | `helio java ...` | Settings screen |
| Modrinth search and project details | `helio search`, `helio info` | Browse screen |
| Mods, resource packs, shaders, data packs: add, remove, enable, update | `helio mod`, `pack`, `shader` | content tabs with clickable switches |
| Modpacks (`.mrpack`) install and import | `helio modpack install` | Browse and wizard import |
| Worlds and screenshots | `helio world ...`, `helio screenshot ...` | Worlds and Screenshots tabs |
| Resources: CPU, RAM, disk, per-game memory | `helio stats [--watch]`, `helio disk` | Resources screen |
| Drag-and-drop install | `helio mod add --file` | pasted file paths on the instance screen |
| Collapsible sidebar, themes, grid background with hover highlight | n/a | sidebar, palettes, background |
| Command palette, help overlay | n/a | `Ctrl+P` / `:`, `?` |
| Diagnostics | `helio doctor` | Settings, About |

## Rules that never change
1. **Diagnose first, never stack code on broken code.** Everything must really work. No stubs, no mock data, no `TODO`, no `unimplemented!()`, no fake progress. If something cannot be finished, say so plainly.
2. **No emojis anywhere** (UI, output, logs, docs, commits). Icons are mandatory and are implemented through the **Icon system** section above: Nerd Font glyphs with Unicode and ASCII fallbacks, on every element of the CLI output and the TUI. Never a raw emoji.
3. **No silent failures.** Typed errors (`thiserror`) in `helio-core`; human-friendly messages in the CLI with an actionable hint ("Minecraft 1.20.5 needs Java 21; run: helio java install 21"). `-v/-vv` raise log verbosity. Logs go to `Helio/logs/helio.log` (rotating). Never log tokens or passwords (redact).
4. **Network goes through one HTTP client** in `helio-core` (`reqwest`, rustls, project User-Agent `Helio/<version>`), with a **host allowlist**: `account.ely.by`, `authserver.ely.by`, `skinsystem.ely.by`, `api.modrinth.com`, `cdn.modrinth.com`, `launchermeta.mojang.com`, `piston-meta.mojang.com`, `piston-data.mojang.com`, `resources.download.minecraft.net`, `libraries.minecraft.net`, `meta.fabricmc.net`, `maven.fabricmc.net`, `meta.quiltmc.org`, `maven.quiltmc.org`, `files.minecraftforge.net`, `maven.minecraftforge.net`, `maven.neoforged.net`, `api.adoptium.net`, `github.com`, `objects.githubusercontent.com`, `raw.githubusercontent.com` (add the authlib-injector host after verifying). Non-allowlisted hosts (including redirects) are rejected.
5. **Verify external APIs against current docs** (web fetch/search): Ely.by OAuth and Yggdrasil, authlib-injector, Modrinth v2, Mojang launchermeta, Fabric/Quilt meta, Forge/NeoForge maven, Adoptium. Never trust memory for endpoints, scopes, parameters. Note source URLs in code comments and `PROGRESS.md`.
6. **Security:** every path goes through `safe_join()`; names through `sanitize_name()`; zip extraction is zip-slip safe; JVM args are separate argv entries (never a shell); JSON writes are atomic (temp then rename); never panic on corrupt user files (back up as `.bak`, use defaults); secrets only in the OS keychain (or an encrypted file fallback with a visible warning), never plaintext JSON.
7. **Scriptability is a feature:** global flags `--json` (machine-readable output on stdout, human text/progress on stderr), `-q/--quiet`, `-v`, `--no-color` (also respect `NO_COLOR`), `--ascii`, `--data-dir`, `-y/--yes` (skip confirmations), `--no-input` (never prompt; fail with an error instead). Stable exit codes: `0` ok, `1` generic error, `2` usage error, `3` not found, `4` auth error, `5` network error, `6` Java problem, `7` launch/game error, `130` interrupted. Progress bars (`indicatif`) only when stderr is a TTY; plain line output otherwise.
8. **Run what you build.** After each step: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`, then actually run the binary and exercise the feature. Commit at the end of each phase; the repo always builds.
9. **Be honest.** Reports list each Done item as PASS or FAIL with evidence (command output). Never claim something works unless you ran it. If blocked externally, stop and explain with evidence.
10. **Stay in scope.** Only do the current phase. Note out-of-scope problems under "Known issues" in `PROGRESS.md`.
11. **Brand discipline.** The project is Helio. Use the `brand` module constants; never hard-code names. A repository-wide search for the legacy name must always come back empty.

## PROGRESS.md
Create it now and update it at the end of each phase: Current phase, Completed phases (commit hash), Decisions (with reasons), Known issues, Test evidence, External facts verified (endpoint, source URL, date).

## Report format (end of every phase)
1. What was built (files). 2. Done checklist PASS/FAIL with evidence. 3. Decisions and deviations. 4. Known issues. 5. Commit hash. 6. What you need from me before the next phase.

Reply only: "Standing orders acknowledged. Waiting for Phase 1."

---

# PHASE 1: Foundation, CLI skeleton, accounts (login)

**Goal:** A working `helio` binary with the full command-line skeleton, safe core plumbing, and a complete account system (Ely.by OAuth and offline). By the end, `helio login` works.

## 1.1 Environment check
Record: `rustc -V`, `cargo -V`, `rustup show` (MSVC toolchain on Windows), `where link.exe` (or a trivial build links), `java -version`, `git --version`, terminal info (`$env:WT_SESSION` for Windows Terminal). If something is missing, stop and tell me exactly what to install (Visual Studio Build Tools with "Desktop development with C++", rustup stable-msvc). Do not work around it.

## 1.2 Workspace and dependencies
Create the workspace. `helio-core` deps: `tokio` (full), `reqwest` (rustls-tls, json, stream, gzip; no openssl), `serde`, `serde_json`, `thiserror`, `tracing`, `tracing-subscriber`, `tracing-appender`, `dirs`, `uuid`, `md-5` (or `md5`), `sha1`, `sha2`, `hex`, `base64`, `rand`, `url`, `keyring`, `chrono`, `zip`, `walkdir`, `tokio-util`, `futures`, `sysinfo`, `aes-gcm` (for the encrypted-file fallback). `helio` deps: `clap` (derive, env, wrap_help), `clap_complete`, `indicatif`, `console` (or `owo-colors`), `dialoguer` (or `inquire`) for prompts, `rpassword`, `comfy-table` for tables, `anyhow` (binary only), `ratatui` and `crossterm` (added in Phase 4; do not add earlier). Release profile: `lto = "thin"`, `codegen-units = 1`, `strip = true`, `panic = "abort"` only if no panics need unwinding for terminal restore (decide and document). Binary name must be `helio` (`[[bin]] name = "helio"`).

## 1.3 Core plumbing (`helio-core`)
- `error.rs`: `OmError` enum (`Io, Json, Http, Auth, NotFound, InvalidInput, Path, Java, Launch, Download, Hash, Cancelled, Config, Other`) with an `exit_code()` and a `hint()` method; `OmResult<T>`.
- `paths.rs`: `resolve_data_dir()` (order: `--data-dir`, `HELIO_DATA_DIR`, config, default platform data dir `.../Helio`), `ensure_tree()`, `safe_join`, `sanitize_name`, `unique_slug`, `write_json_atomic`, `read_json_or_default` (with `.bak` recovery). **Unit tests:** traversal (`../x`, `..\x`, `a/../../b`), absolute and drive paths, reserved Windows names (`CON`, `NUL`, `COM1`...), unicode, collisions, long names.
- `config.rs`: `Config` (`schema`, `data_dir`, `java_path`, `min_ram_mb`, `max_ram_mb`, `jvm_args`, `resolution`, `concurrent_downloads` (default 8, range 1-16), `close_on_launch`, `theme`, `ascii_icons`, `active_account_id`, `elyby_client_id`, `check_updates`). `#[serde(default)]`, migration hook, validation (min <= max, max <= system RAM). Commands `helio config list|get|set|reset|path` must validate keys and values with clear errors.
- `http.rs`: shared client, allowlist enforcement (test: non-allowlisted URL and a redirect to one are rejected without a request), `get_json`, `get_bytes`, and `download_to` with hash verify (full downloader comes in Phase 2).
- `events.rs`: `Event` enum (`Progress { task, stage, current, total, bytes_done, bytes_total, message }`, `Log`, `AuthStage`, `Info`, `Warn`) and a throttled emitter (max ~10/s per task, always emit the last one).
- `tracing` setup with a redaction helper; a panic hook that logs location.

## 1.3b Brand and icon foundations (build these before any command output)
- `brand.rs` with the constants from the Branding section, and run the **name check** (web search), recording results in `PROGRESS.md`.
- `theme.rs` with the semantic tokens and the three palettes (Sun, Midnight, Daylight), each with truecolor, 256 and 16-color variants, and automatic selection from `COLORTERM`/`TERM`.
- `icons.rs`: the full registry (at least 100 icons, three renderings each), the modes, the `glyph + pad` renderer, the width table, and a `Styled<Icon>` helper used by **every** printing path in the CLI. Tests: no emoji code points; every enum variant has all three renderings; alignment of icons inside `comfy-table` cells and bordered boxes in all three modes.
- First-run icon check and `helio icons` and `helio doctor --icons` as specified in the Icon system section.
- A shared output layer (`output.rs`) used by all commands: `success`, `info`, `warn`, `error`, `hint`, `section`, `kv` and `table` helpers that always include the right icon and color, and that switch to plain text for `--json`, `--quiet`, `NO_COLOR` or non-TTY.
- `--help` headings carry icons too (custom clap help template).

## 1.4 CLI skeleton (`helio`)
Define the **entire** command tree now with `clap`, with real help text, examples, and aliases. Commands implemented in later phases must exist in the tree but return a clear "not implemented yet (Phase N)" error with exit code 1. Do not fake their output. Tree:

```
helio                              # (Phase 4) opens the TUI; until then prints help
helio tui                          # (Phase 4)
helio login [--elyby | --offline <NAME>] [--client-id ID] [--no-browser]
helio logout [ACCOUNT]
helio account list | use <ACCOUNT> | remove <ACCOUNT> | refresh [ACCOUNT] | info [ACCOUNT]
helio skin [ACCOUNT] [--save FILE]                       # prints skin/head in terminal, can save PNG
helio instance create <NAME> [-v VERSION] [--loader L] [--loader-version V] [--icon FILE]
helio instance list | info <I> | rename <I> <NEW> | delete <I> | duplicate <I> [NEW] | open <I> | set <I> KEY VALUE | export <I> FILE | import FILE
helio launch <I> [--account A] [--dry-run] [--detach] [--logs] [--kill-on-exit] [--demo]
helio kill <I|--all>        helio ps        helio logs <I> [-f] [-n N] [--level L] [--grep PATTERN] [--last]
helio versions [--snapshots] [--old]     helio loader versions <LOADER> <MC>
helio java list | install <MAJOR> | use <PATH|MAJOR> | rescan | test <PATH>
helio search <QUERY> [--type mod|modpack|resourcepack|shader|datapack] [--version V] [--loader L] [-n N]
helio info <MODRINTH_SLUG>  (project details)
helio mod add <I> <SLUG...> [--version V] | remove <I> <NAME> | list <I> | enable|disable <I> <NAME> | update <I> [--all]
helio modpack install <SLUG|FILE.mrpack> [--name N] [--version V]
helio pack add|remove|list|enable|disable <I> ...        # resource packs
helio shader add|remove|list|select <I> ...
helio world list <I> | backup <I> <WORLD> | delete <I> <WORLD>
helio screenshot list <I> | open <I> <FILE> | delete <I> <FILE>
helio stats [--watch] [--json]        helio disk [--json] [--clear-cache cache|logs|thumbnails]
helio icons [--mode nerd|unicode|ascii] [--json]
helio config ...   helio doctor [--icons]   helio completions <SHELL>   helio version [--banner]
```
`<I>` accepts the instance slug, the exact name, a unique prefix, or the id. Ambiguity prints the candidates and exits with code 2. Every command that lists or shows data supports `--json`. Shell completions (`bash`, `zsh`, `fish`, `powershell`) must really generate, and dynamic completion of instance names is a bonus (document if not done).

## 1.5 Accounts (`helio-core::accounts`)
**Models:** `Account { id, kind: Offline|Elyby, username, uuid, added_at }` stored in `accounts.json` (**metadata only, no tokens**). Active account id in config.

**Offline:** `validate_username` (3-16, `^[A-Za-z0-9_]+$`, specific messages). `offline_uuid(name)`: MD5 of UTF-8 `"OfflinePlayer:" + name`, set `b[6] = (b[6] & 0x0f) | 0x30`, `b[8] = (b[8] & 0x3f) | 0x80`, format hyphenated. **Unit test with a reference vector you compute independently by a second method (record how).** Reject duplicates (case-insensitive).

**Secrets (`keychain.rs`):** `keyring` wrapper (service `Helio`, key `elyby:<uuid>`), with set/get/delete. If the keychain is unavailable (headless Linux, etc.): fall back to an **AES-GCM encrypted file** `runtime/secrets.bin` with a key file `runtime/.key` (restrictive permissions) and print a one-time warning. Never plaintext. Test set/get/delete here.

**Ely.by OAuth2 (`elyby.rs`):** First read the current Ely.by developer docs and record in `PROGRESS.md`: authorize URL, token URL, user-info URL, scopes (`account_info`, `minecraft_server_session`, others), redirect-URI rules, PKCE support, whether a client secret is needed for native apps, and how a Minecraft access token is obtained. Then implement the flow for a CLI:
1. Client ID from `--client-id`, `HELIO_ELYBY_CLIENT_ID`, or `config.elyby_client_id`. If missing, fail with exact setup instructions (register an app named Helio at `account.ely.by/dev/applications`, which redirect URI to register, how to pass the id). Never hard-code secrets.
2. Bind a one-shot HTTP listener on `127.0.0.1` only. If Ely.by requires an exact registered redirect URI, use a fixed documented port (with a short fallback list) and say so in the instructions. Generate a random `state` (and PKCE if supported). Open the authorize URL in the system browser (`open`/`webbrowser` crate), and always **also print the URL** (for copy-paste).
3. Serve a small success page ("Signed in. You can return to the terminal."), validate `state` (reject mismatch), time out after 4 minutes, Ctrl+C cancels and frees the port.
4. **`--no-browser` / SSH fallback:** print the URL; the user opens it anywhere and pastes the final redirected URL (or the `code`) back into the terminal; the CLI validates `state` and continues. This must work on a machine with no browser.
5. Exchange the code, fetch the profile (uuid, username), store tokens in the keychain, save the account metadata, set it active if it is the first.
6. `ensure_fresh_token(account)`: refresh when expiry is under 5 minutes away; typed `Auth` error (exit code 4) with the hint `helio login --elyby` on failure.
7. **Launch credentials:** determine from the docs whether the OAuth token (with the session scope) can be used directly as the Minecraft access token against Ely.by's session server. If yes, use it. If not, implement the documented alternative (Ely.by Yggdrasil via `authserver.ely.by`) behind the same function (hidden password prompt via `rpassword`, **never stored or logged**), and tell me exactly what was needed and why. Expose `launch_credentials(account) -> { username, uuid, access_token, user_type, is_elyby }` (offline: token `0`; user type `legacy` for versions before 1.7, `msa` otherwise; leave a note for Phase 2).
8. **authlib-injector:** determine the official artifact metadata URL, implement `ensure_authlib_injector()` which downloads to `runtime/authlib-injector.jar`, verifies the published checksum, and returns the path; allowlist its host. Determine the exact Ely.by injector API URL (expected `https://authserver.ely.by/api/authlib-injector`; verify).

**Skins (`skins.rs`):** `get_skin(account) -> { skin_png, cape_png?, model }`. Ely.by: use the Ely.by skin system (verify `skinsystem.ely.by/textures/<name>` and `/skins/<name>.png` and the cape endpoint), cached under `cache/skins/` with a TTL. Offline: bundled default Steve/Alex chosen by UUID parity. Provide `render_head(skin, scale)` and `render_body(skin)` that output a grid of RGBA pixels (for terminal rendering).

## 1.6 CLI behavior for accounts
- `helio login` with no flags and a TTY: an interactive chooser (arrow keys): "Ely.by (browser)" or "Offline (enter a username)". With `--no-input` and no flags: error exit 2.
- `helio login --offline Steve` and `helio login --elyby [--no-browser]` as above; progress lines on stderr (`opening browser`, `waiting for approval`, `exchanging code`, `fetching profile`).
- `helio account list` shows a table: active marker, username, type badge (`elyby`/`offline` with icon), UUID, added. `--json` outputs an array. `helio account use`, `remove` (confirm unless `-y`, removes keychain entries), `refresh`, `info`.
- `helio logout [ACCOUNT]` is an alias for remove of the account (default: active).
- `helio skin` renders the head (and optionally the full body, `--body`) in the terminal using **Unicode half blocks with 24-bit color** (fallback to 256 colors, then to ASCII shading with `--ascii`), and `--save` writes the PNG.
- `helio doctor` (basic now): reports data dir, config validity, keychain availability, Java detection results (a simple `java -version` for now), network reachability of the allowlisted hosts (HEAD/GET with a short timeout), terminal capabilities (color depth, Unicode, mouse), and prints PASS/WARN/FAIL per check with hints.
- `helio version` prints version, commit, target.

## Definition of Done (Phase 1)
- [ ] `helio icons` prints the registry in all three modes; the first-run icon check works (answer yes and no paths); no emoji code points exist (test); table and border alignment tests pass in all modes.
- [ ] Name check results and the final package and binary names are recorded in `PROGRESS.md`; the banner renders in all three icon modes; `helio version --banner` works.
- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` all pass. `helio --help` and every subcommand `--help` render. Unimplemented commands fail honestly.
- [ ] Unit tests pass: `safe_join`, `sanitize_name`, `unique_slug`, offline UUID (independent vector), username validation, allowlist rejection (including redirect), config validation, instance-ref resolution if you wrote it.
- [ ] `helio login --offline Steve` works; invalid names and duplicates fail with exit code 2/1 and clear text; `helio account list --json` is valid JSON.
- [ ] **Real Ely.by login** completes via the browser path **and** via the `--no-browser` paste path; tokens are in the keychain and `accounts.json` contains none (show both). A restart refreshes silently. A tampered `state` is rejected (test). Ctrl+C frees the port (rerun login immediately).
- [ ] authlib-injector downloaded and checksum verified; Ely.by credential mechanism documented in `PROGRESS.md` with source URLs.
- [ ] `helio skin` renders an Ely.by skin and an offline default in the terminal; `--save` writes a valid PNG.
- [ ] `helio doctor` runs and reports truthfully; no secrets appear in `helio.log`.
- [ ] Committed; `PROGRESS.md` and the report written.

---

# PHASE 2: Engine: downloads, Java, instances, launch, process control

**Goal:** `helio launch` runs vanilla Minecraft for real, with proper progress, log streaming, detach, kill and crash detection.

## 2.1 Version manifest and models
- `manifest.rs`: fetch `https://launchermeta.mojang.com/mc/game/version_manifest_v2.json`, cache with a 1 h TTL, use stale cache offline with a warning. `helio versions` lists `id, type, release date` (filters for snapshots and old), `--json`, `latest` markers.
- `version_json.rs`: serde models tolerant of every variant: `id, type, mainClass, inheritsFrom, assetIndex, assets, downloads, javaVersion, libraries (downloads.artifact/classifiers, rules, natives, extract.exclude, name+url only), arguments.{game,jvm} (string or {rules, value}), minecraftArguments, logging.client, releaseTime`. Save real fixtures for **1.8.9, 1.12.2, 1.16.5, 1.20.1, 1.21.x** in `tests/fixtures/` and test that all deserialize.
- `rules.rs`: `evaluate(rules, ctx)` (no rules allow; last matching rule wins; `os.name`, `os.arch`, features `is_demo_user`, `has_custom_resolution`, quick-play flags default false). Thorough unit tests (Windows-only native, macOS-only lib, disallow precedence).

## 2.2 Downloader (`download.rs`)
Parallel (semaphore from config), per item: skip if present and hash (or size) matches; else download to `.part` (resume via `Range` when supported), verify sha1/sha256/sha512, atomic rename; 3 retries with exponential backoff and jitter (hash mismatch counts as a failure and deletes the part); aggregate progress events; cancellation token with no corrupt files left behind. **Tests with a local mock HTTP server:** success, hash mismatch then retry, resume, cancel, 404.

## 2.3 Libraries, natives, assets, orchestrator
- `libraries.rs`: apply rules, pick artifacts, derive maven paths for `name+url`-only libs (unit-test `maven_path` with classifiers and `@ext`), handle legacy `natives` classifiers (including `${arch}`) and modern per-OS natives on the classpath (1.19+), `extract_natives(target)` zip-slip-safe honoring `extract.exclude`.
- `assets.rs`: index to `assets/indexes/<id>.json`, objects to `assets/objects/<aa>/<hash>` from `resources.download.minecraft.net`; handle `virtual` and `map_to_resources` indexes for old versions.
- `install.rs`: `prepare_version(id)`: version JSON (verify sha1; resolve `inheritsFrom`), client jar, libraries, logging XML, asset index and objects; idempotent (a second run does not hit the network); stages `manifest, client, libraries, logging, assets, done`.

## 2.4 Java (`java/`)
- **Detect:** Windows registry (`HKLM\SOFTWARE\JavaSoft`, `Eclipse Adoptium`, `Microsoft\JDK`, `Azul Systems`, `Amazon Corretto`; 32 and 64-bit views), `JAVA_HOME`, `PATH`, common dirs (`Program Files\{Java,Eclipse Adoptium,Microsoft,Zulu,Amazon Corretto}`, `~/.jdks`, `/usr/lib/jvm`, `~/.sdkman/candidates/java`), and `runtime/java/*`. For each candidate run `java -version` (5 s timeout), parse `1.8.0_392`, `17.0.9`, `21`, `21.0.1`, capture major, vendor, arch; dedupe by canonical path; cache to `cache/java.json`. **Unit-test the parser** on real outputs for Java 8, 11, 17, 21, 25.
- **Select:** requirement from `javaVersion.majorVersion` (fallback table: <= 1.16.5 uses 8, 1.17 prefers 17, 1.18-1.20.4 uses 17, >= 1.20.5 uses 21). Precedence: instance override, global path if compatible, best auto-detected (exact major, else a higher major only where safe, never lower). On failure return `OmError::Java` with the hint `helio java install <N>` (exit code 6).
- **Install:** `helio java install <MAJOR>`: verify the Adoptium API for the latest GA JRE for this OS/arch, download, verify the checksum, extract zip-slip-safe into `runtime/java/temurin-<major>/`, rescan. `helio java list` table (major, vendor, arch, path, source, selected marker); `use`, `rescan`, `test`. Launching offers (interactive TTY only) to install the missing Java automatically.

## 2.5 Instances (`instances/`)
`.helio.json` (schema 1): `id, name, slug, mc_version, loader (vanilla for now), loader_version, icon, java_path?, min_ram_mb?, max_ram_mb?, jvm_args?, resolution?, created_at, last_played, total_playtime_secs, modpack?, account_id?`. Commands: `create` (sanitized name, unique slug, folder tree `mods resourcepacks shaderpacks saves logs screenshots config`, optional icon PNG validated by magic bytes and resized to 128x128), `list` (table: name, version, loader, last played (relative), playtime, running marker), `info`, `rename`, `delete` (confirm; refuse while running; `safe_join`), `duplicate` (progress for large folders), `open` (open in the OS file manager), `set KEY VALUE` for per-instance overrides (`java`, `min-ram`, `max-ram`, `jvm-args`, `resolution`, `account`) with validation, `export`/`import` (zip, excluding `logs`, `natives`, `crash-reports`; import is zip-slip safe). `helio instance create` with no `-v` uses the latest release; interactive prompts only on a TTY.

## 2.6 Launch builder (`launch.rs`)
`build_command(ctx) -> LaunchCommand { java, args, cwd, env }`:
- Classpath: libraries in order, deduplicated by `group:artifact` (the later wins), then the client jar; separator `;` on Windows, `:` elsewhere.
- Natives extracted per launch to `instances/<slug>/natives/<uuid>/`, removed on exit.
- **Argument order:** (1) authlib-injector `-javaagent:<jar>=<ely.by api>` first, Ely.by accounts only; (2) `-Xms/-Xmx` (instance, else global; clamp to system RAM); (3) user JVM args parsed with a shell-style tokenizer into separate argv entries, never via a shell; (4) version JVM args (modern) or legacy defaults (`-Djava.library.path=${natives_directory} -cp ${classpath}`); (5) the logging argument if present; (6) main class; (7) game args (modern `arguments.game` or legacy `minecraftArguments`); (8) resolution args when set.
- Placeholders: `auth_player_name, auth_uuid, auth_access_token, auth_session, auth_xuid, clientid, user_type, user_properties ({}), version_name, version_type (Helio), game_directory, assets_root, assets_index_name, game_assets, natives_directory, library_directory, classpath, classpath_separator, launcher_name, launcher_version, resolution_width, resolution_height`. Unknown placeholders warn and become empty. Legacy `auth_session` is `token:<accessToken>:<uuid>`.
- On Windows, if the command line would exceed about 30000 characters, write a Java `@argfile` (Java 9+).
- Working directory and `game_directory` are the instance folder.
- `--dry-run` prints the exact command (access token replaced by `***`), as shell-quoted text, or as JSON with `--json`.
- **Tests** against fixtures (1.8.9 legacy, 1.12.2, 1.20.1, 1.21.x) with a fake offline account: assert main class, `-Djava.library.path`, `--username`, `--uuid`, `--version`, `--assetsDir`, `--assetIndex`, no leftover `${...}`, correct separator, javaagent first for Ely.by, absent for offline.

## 2.7 Process control (`process.rs`)
Because the CLI process may exit while the game keeps running (`--detach`), **state lives on disk**:
- On spawn write `runtime/run/<instance-id>.json` with `{ pid, start_time (process creation time from sysinfo), started_at, log_path, command_hash }`. `helio ps` and every liveness check read these files and validate them by `pid` **and** process start time (guards against PID reuse), removing stale entries.
- Launching a running instance returns a friendly error (exit 7).
- **Flow of `helio launch <I>`:** resolve the instance and account (`--account`, instance override, active), refresh the Ely.by token, `prepare_version` with a progress bar (stages and bytes), select Java (error with a hint, or the offer to install), build the command, spawn (Windows: `CREATE_NO_WINDOW` only for helpers, not the game; Unix: new process group), write the run file, then:
  - default (attached): print "Launched <name> (pid N)". With `--logs` stream the game's log lines to the terminal; otherwise wait quietly and print the exit status when the game ends. **Ctrl+C while preparing** cancels the downloads cleanly. **Ctrl+C while the game runs** detaches (the game keeps running; print that) unless `--kill-on-exit`.
  - `--detach`: return right after spawn. The detached game's stdout/stderr must still be captured to `instances/<slug>/logs/helio-<timestamp>.log` (redirect the pipes to the file at spawn time for detached mode, or use a tiny supervisor; document your choice and prove logs are complete).
- **Logs:** parse levels from log4j patterns (`[INFO]`, `[WARN]`, `[ERROR]`, `[DEBUG]`, `[FATAL]`; stderr maps to error). `helio logs <I>` shows the most recent session log (`--last` selects the last session explicitly, `-n N` tails N lines, `-f` follows live, `--level`, `--grep`), colorized on a TTY, plain with `--no-color` or when piped.
- **Exit handling:** an attached launch reports exit code and crash detection (non-zero exit, or a new file in `crash-reports/`, whose path is printed). Update `last_played` and playtime (for detached games, compute on the next `helio ps`/`helio launch` when the process is found dead, using the stored start time and the log's last write time; document the approximation).
- **Kill:** `helio kill <I>` and `helio kill --all`: Windows `taskkill /PID <pid> /T /F` (kill the tree), Unix `killpg(SIGTERM)` then `SIGKILL` after 5 s; verify the process is gone; remove the run file. Must work on a hung game.
- `helio ps` table: instance, pid, uptime, memory (RSS), CPU %, account; `--json`.

## 2.8 Stats and disk (backs the Resources screen)
- `helio stats`: Helio process CPU % and RSS, system RAM used/total, global CPU, disk free for the data drive, and a row per running game (instance, pid, uptime, RSS, CPU %). CPU needs two samples: keep a long-lived `sysinfo::System` and refresh incrementally. `--watch` redraws in place every second (icons, bars) until Ctrl+C; `--json` emits one JSON object (or NDJSON with `--watch`).
- `helio disk`: sizes of the data root broken into `instances`, `libraries`, `assets`, `versions`, `runtime`, `cache`, `logs`, plus a per-instance table sorted by size, computed on a blocking thread with a progress spinner and cached for 60 s; `--clear-cache cache|logs|thumbnails` frees space (confirm unless `-y`, prints the freed size; never touches libraries or assets).
- Both live in `helio-core` (`system.rs`) so the TUI Resources screen reuses them.

## Definition of Done (Phase 2)
- [ ] `helio stats --watch` numbers match Task Manager within reason (compare against a running game); `helio disk` totals match Explorer for the instances folder; `--clear-cache` frees the reported size.
- [ ] Fixtures deserialize; rules, downloader (mock server), maven-path, Java-parser and command-builder tests pass; clippy clean.
- [ ] `helio instance create test --version 1.20.1`, then `helio launch test` with an **offline** account: real progress bar, the game window opens and reaches the title screen, `helio logs test -f` streams, `helio ps` shows it, `helio kill test` ends the game **and** the Java process (verify with `tasklist`), the run file is removed.
- [ ] A second `helio launch` with everything cached starts with zero network requests (show the logs).
- [ ] `helio launch --detach` returns immediately; the game keeps running after the terminal is closed; logs are complete; `helio ps` and `helio kill` work from a new terminal.
- [ ] 1.8.9 launches with Java 8 (legacy args). A missing Java gives exit code 6 with the hint; `helio java install 21` works and then the launch succeeds.
- [ ] Ely.by launch: `--dry-run` shows authlib-injector first; in game the Ely.by skin appears (or document exactly what you saw). Offline dry run has no injector.
- [ ] Ctrl+C during download cancels cleanly and resumes next time; Ctrl+C while running detaches; `--kill-on-exit` kills.
- [ ] Corrupting a library on disk causes a re-download on the next launch. Libraries and assets are shared between versions.
- [ ] Crash detection works (force one by an invalid JVM arg or a deliberately bad mod if you have one); playtime and `last_played` update.
- [ ] `helio doctor` now checks Java properly. Committed; `PROGRESS.md` and the report written.

---

# PHASE 3: Loaders, Modrinth, content management

**Goal:** Fabric, Quilt, Forge, NeoForge instances launch; mods, modpacks, resource packs, shaders and worlds are fully manageable from the CLI. Do the loaders strictly in order and gate each.

## 3.1 Loader framework and inheritance merge
`Loader { Vanilla, Fabric, Quilt, Forge, NeoForge }`: `list_versions(mc)` (with `stable`/`recommended` flags; cached; stale cache offline) and `install(mc, loader_version)` which produces a profile JSON under `versions/<profile_id>/` using `inheritsFrom`. Implement `merge(parent, child)`: libraries (child first, dedupe by `group:artifact[:classifier]`, child wins), arguments (parent then child appended for `jvm` and `game`), `mainClass` (child), `assetIndex/assets/downloads/javaVersion/logging` (child if present, else parent), `minecraftArguments` (child if present). Tests with real Fabric and Forge profile fixtures. `prepare_version` and `build_command` work on the merged JSON using the parent's client jar. Loader installs are idempotent and cached. `helio loader versions <loader> <mc>` lists them (`--json`).

## 3.2 Fabric, then Quilt
Verify the meta APIs. Fabric: `https://meta.fabricmc.net/v2/versions/loader/<mc>` and `.../<loader>/profile/json`. Quilt: `https://meta.quiltmc.org/v3/versions/loader/...`. **Gate:** Fabric 1.20.1 and Quilt 1.20.1 launch; the log shows the loader starting.

## 3.3 Forge, then NeoForge
- Verify the official sources (Forge promotions + maven metadata; NeoForge versions API at `maven.neoforged.net` and installer coordinates; note NeoForge 1.20.1 used different coordinates).
- Download the installer to `cache/installers/` (verify checksums where published) and run it **headless**: `<java> -jar <installer> --installClient <tmp_dir>` where `<tmp_dir>` contains a minimal `launcher_profiles.json`. Use the Java the installer needs (Java 17+ for modern, Java 8 for 1.12 and below). Stream installer output as verbose progress; on failure show the last 30 lines.
- Move the produced version JSON and libraries into the shared `versions/` and `libraries/`, delete the temp dir. Handle legacy (1.12.2: `--tweakClass`, universal jar) and modern (1.13+: `-DlibraryDirectory`, `--launchTarget forgeclient`, `${library_directory}`, `${classpath_separator}`).
- **Gate:** Forge 1.20.1, Forge 1.12.2, NeoForge (latest stable you can verify) launch. Document precisely which versions work and which fail and why. Do not claim untested support.
- `helio instance create --loader fabric --loader-version latest` accepts `latest`, `recommended` or an explicit version, and fails with a clear message when a loader has no build for that MC version.

## 3.4 Modrinth API (`modrinth/`)
Base `https://api.modrinth.com/v2` (verify). Project User-Agent. Respect rate limits (`X-Ratelimit-*`, retry on 429 after the indicated delay); short in-memory cache. Typed models for search hits, projects, versions (`files[{url, filename, primary, hashes{sha1,sha512}, size}]`, `dependencies`). Search builds facets correctly (OR within a group, AND across groups): `[["project_type:mod"],["versions:1.20.1"],["categories:fabric"]]`; supports `index` (`relevance|downloads|follows|newest|updated`), `limit`, `offset`.
- `helio search <query> --type ... --version ... --loader ... -n 20` prints a table (title, author, downloads, loaders, latest version, slug); `--json`. `helio info <slug>` shows the project details (description rendered as plain text from the markdown, links, license, versions).

## 3.5 Content installation
- `helio mod add <I> <slug...>`: pick the latest release version compatible with the instance's MC version and loader (mods filter by loader; resource packs, shaders and data packs by game version only; shaders do not filter by loader). **Dependency plan:** resolve `required` dependencies recursively (cycle-safe), skip already-installed ones (match via sha1 of installed jars and `/version_files`), print the plan (install, skipped, incompatible) and ask for confirmation (skip with `-y`; `--dry-run` prints only). Download with hash verification into `mods/` (or `resourcepacks/`, `shaderpacks/`, a world's `datapacks/`). Clear errors for "no compatible version for MC X / loader Y".
- `helio mod list <I>`: parse each jar (`fabric.mod.json`, `quilt.mod.json`, `META-INF/mods.toml`, `META-INF/neoforge.mods.toml`, legacy `mcmod.info`; fall back to the filename; never panic on malformed jars): name, id, version, authors, enabled marker; `--json`. `enable/disable` rename to or from `.jar.disabled`. `remove` (to the recycle bin via the `trash` crate, with confirm). `update <I> [--all]` hashes installed files, uses `/version_files/update`, shows what would change and applies on confirm.
- `helio pack ...` (resource packs: list with `pack.mcmeta` description and format, enable/disable by editing `options.txt` `resourcePacks` safely preserving other lines and order, add/remove) and `helio shader ...` (list, add, remove, `select` writes `optionsshaders.txt`). `helio world list/backup/delete` (parse `level.dat` NBT for the world name, last played, size; backup zips to `instances/<slug>/backups/`).
- All adds also accept a local file path or `--file` (validate: a jar must have a mod descriptor, a pack must have `pack.mcmeta`, a shader zip must contain `shaders/`).

## 3.6 Modpacks
`helio modpack install <slug|file.mrpack> [--name N] [--version V]`: download the `.mrpack` (verify hash) or open the local file safely; parse `modrinth.index.json`; read `dependencies` (`minecraft` and exactly one of `fabric-loader`, `quilt-loader`, `forge`, `neoforge`); **create a new instance** with the right MC version, loader and loader version; download every `files[]` entry to its `path` (validate with `safe_join`, only from `cdn.modrinth.com`, `github.com`, `raw.githubusercontent.com`, `gitlab.com`), verify sha1/sha512, skip `env.client == unsupported`, in parallel with a progress bar; apply `overrides/` then `client-overrides/` (zip-slip safe); use the project `icon_url` as the instance icon; record `modpack` metadata. On cancel or failure delete the half-built instance (only if this task created it). Tests: parse a real index fixture; a zip-slip `.mrpack` is rejected.

## Definition of Done (Phase 3)
- [ ] Merge tests pass; Fabric 1.20.1, Quilt 1.20.1, Forge 1.20.1, Forge 1.12.2, NeoForge (latest stable) each launched via `helio launch` with log evidence. A version matrix (worked/failed and why) is in `PROGRESS.md`. A second launch of each does not reinstall.
- [ ] `helio search` returns correct results for each project type and filter; a simulated 429 is handled.
- [ ] `helio mod add fabric1201 fabric-api sodium` shows the dependency plan and installs; the game launches with them.
- [ ] `helio modpack install <a real small Fabric modpack>`: new instance, icon, mods, overrides; it launches. A zip-slip `.mrpack` is rejected. Cancel removes the half-built instance.
- [ ] `helio mod list` parses Fabric, Quilt, Forge, NeoForge and legacy jars correctly (show a results table); enable/disable, remove (recycle bin), update all work.
- [ ] Resource pack enable/disable edits `options.txt` and preserves other lines (verify in game); shader select writes `optionsshaders.txt`; world backup produces a valid zip.
- [ ] Every listing command has valid `--json`. Committed; `PROGRESS.md` and the report written.

---

# PHASE 4: The TUI (`helio`), mouse support, polish, release

**Goal:** Running `helio` opens a modern, mouse-driven, full-screen terminal UI that can do **everything** the commands can, built on the same `helio-core` API. Then polish, tests, CI, docs and release packaging.

## 4.1 TUI foundation (`crates/helio/src/tui/`)
- `ratatui` + `crossterm`. Enable **mouse capture** (click, double-click, right-click, drag, scroll wheel, hover/move), **bracketed paste**, **focus events**, alternate screen, raw mode. A `TerminalGuard` restores the terminal on exit, on panic (panic hook) and on Ctrl+C. Handle resize live. Minimum 80x24 (show a friendly "terminal too small" screen below it).
- Architecture: an `App` state machine with a single event loop that merges terminal events, a 30 fps tick (only redraw when dirty), and messages from background tasks over a channel. **The UI thread never blocks:** every core call runs in a `tokio` task and reports back with messages (progress, results, errors). Hit-testing: every interactive widget registers a `Rect` plus an action in a per-frame **hit map**; mouse events are resolved through it (click, hover, scroll targets). Separate `view` (pure rendering) from `update` (state transitions) and write unit tests for `update` and the hit map.
- Theme: the semantic tokens and the three palettes from `theme.rs` (Sun default, Midnight, Daylight) with truecolor, 256-color and 16-color variants, dark and light. The accent is the warm Helio amber. Style: boxed panels with chunky borders (`BorderType::Thick` or double), a clear focus highlight, consistent spacing, and the Helio sun banner on the splash and login screens. Icons come only from `icons.rs` and appear on **every** element: sidebar entries, instance rows, tab labels, table headers, buttons, menu items, toasts, status-bar hints (each hint has an icon), badges, progress stages, dialogs and empty states. Icon mode is switchable live from Settings with an instant preview.
- **Background grid of squares with hover highlight (optional toggle, default on when the terminal is truecolor):** render a faint grid pattern in empty areas; the cells near the mouse pointer light up softly and fade out over about 300 ms after the pointer leaves. Pre-compute, dirty-redraw only affected cells, never exceed ~2% CPU idle, disable automatically over SSH or at low color depth, honor `--no-motion` and `NO_MOTION`.

## 4.2 Layout (the Helio layout)
- **Left sidebar, collapsible/expandable** (click the toggle, or `Ctrl+B`, animate by stepping width over a few frames; remember state): expanded shows icon + label, collapsed shows icons only, with a hover **tooltip** line in the status bar (or a floating label). Top to bottom:
  1. **Home** (icon): instances as cards or rows, recently played, quick launch.
  2. **Browse** (icon): Modrinth browser (tabs: Modpacks, Mods, Resource Packs, Shaders, Data Packs).
  3. **New** `+` (icon): the create-instance wizard.
  4. **Avatar** (icon): terminal skin viewer.
  5. **Resources** (icon): live resource usage.
  6. **Breaker** (a chunky divider line with a small label).
  7. **Instance list**: icon (a colored loader glyph; custom icons shown as a small half-block thumbnail if the terminal supports truecolor), the name when expanded, a running indicator (animated dot). Scrollable. **Right-click menu**: Launch, Kill, Open folder, Duplicate, Delete.
  8. **Settings** (icon), directly above the **account picker** at the very bottom (skin head rendered with half blocks, username, type badge, click to open a dropdown: switch account, add Ely.by, add offline, sign out).
- Main area changes with the selection. A **status bar** at the bottom shows context hints (clickable key hints), background task progress (a compact progress indicator), and the clock if wanted. A **toast** area shows notifications (success, info, warn, error) that auto-dismiss.
- **Command palette** (`Ctrl+P` or `:`): fuzzy-search over every action and over equivalent `helio` commands; also accepts a typed command like `launch my-pack` and runs it in-process, showing output in an overlay. A **help overlay** (`?`) lists all key bindings and mouse gestures. Every mouse action has a keyboard equivalent (arrows, Tab, Enter, Esc, vim keys optional); everything is operable without a mouse.

## 4.3 Screens
- **Home:** greeting with the active account, stats (instances, total playtime, running). Instance grid or list with cards (icon, name, version, loader badge, last played, playtime, Launch/Kill button). Sort and search (`/`). Empty state with icon and actions ("Create instance", "Browse modpacks"). Double-click or Enter launches; single click selects; right-click opens the context menu.
- **Instance screen** (opened by selecting an instance): header (icon, name, version, loader badge, playtime, last played, running dot), a large **Launch** button that becomes **Kill** when running, with a progress bar and stage label during preparation (from core events), a **More** menu (View logs, Kill, Open folder, Copy launch command, Duplicate, Export, Delete with confirm). **Icon tabs** (clickable, `Tab`/`Shift+Tab`, number keys): Overview, Worlds, Mods, Resource Packs, Shaders, Screenshots (list with file info; `Enter` opens in the OS viewer; optional inline half-block preview), Logs, Settings. Each content tab lists items (name, version, enabled switch you can click), supports search, multi-select (Space, Ctrl+click, Shift+click), bulk enable/disable/delete, **Add** (opens the Browse view locked to this instance's version and loader, or a path prompt), Open folder, and drag-to-scroll scrollbars.
  - **Logs tab:** live, virtualized, level colors, filter chips (clickable), search with next/prev, follow toggle that pauses when you scroll up with a "jump to latest" button, copy (to the clipboard via OSC 52 and the `arboard` crate where available), clear, open file. Smooth at 20k lines.
  - **Settings tab:** per-instance overrides with sliders (click or drag on the track) and text inputs (`tui-input` or `tui-textarea`), "use global default" toggles, dirty tracking and unsaved-changes prompt.
- **Browse:** search box, filter bar (version, loader, category, sort), results list with infinite scroll (load more near the bottom), a detail pane (description, links, versions), **Install** with the dependency-plan confirm dialog and progress. Modpacks install as a new instance.
- **New instance wizard:** name (live slug preview and uniqueness check), version list (search, Release/Snapshot/Old toggles), loader chooser (five clickable cards with icons) and loader-version list (recommended/latest tags), icon (a file path prompt), advanced (memory, Java, JVM args), Create, then "Launch now". Also **Import .mrpack** (path prompt).
- **Avatar:** the account's skin rendered with half blocks, with **front/back/left/right rotation** by clicking arrow buttons, dragging horizontally with the mouse (rotates a pseudo-3D turntable by switching between pre-rendered views), or keys; toggles for cape and outer layers; load a local skin PNG for preview; account info panel (name, UUID with copy, type badge).
- **Resources:** live (1 s) CPU and RAM for the launcher and the system, running instances' PID, RSS, CPU, uptime with a Kill button, sparklines (`ratatui` `Sparkline`), and a stacked disk usage bar by category (instances, libraries, assets, versions, runtime, cache, logs) with a per-instance table and a Clear cache action (confirm). Disk usage is computed on a background task and cached.
- **Settings:** Java (detected list, install buttons, test), memory sliders capped at system RAM, JVM args with presets (Aikar's flags, G1GC basic, low memory; cite the source), game resolution, launcher options (theme, sidebar state, squares background, icons mode, close on launch, concurrent downloads), storage (data dir display, open), About and `helio doctor` results, reset to defaults (confirm). Autosave with a "saved" indicator.
- **Dialogs:** confirm dialogs for all destructive actions (mouse buttons and `y/n`), text input prompts, file pickers (a simple path input with tab completion, optionally a mini directory browser), progress modals with Cancel.
- **First run:** run the icon check first (if not yet confirmed), then, if no account exists, open a full-screen login screen with two big options: **Sign in with Ely.by** (shows the stage, the URL for copy, and a Cancel; supports the paste fallback inline) and **Continue Offline** (username field with live validation).
- **Drag and drop:** terminals deliver dropped files as pasted paths. Detect a bracketed paste containing existing file paths while an instance screen is open and install them by type (jar to `mods/`, resource pack zip, shader zip, `.mrpack` import prompt) with a summary toast.

## 4.4 Behavior details
- Launching from the TUI uses the exact same core path as `helio launch` (progress events become a progress bar; game logs feed the Logs tab). Running state is derived from the on-disk run files, so games started by `helio launch --detach` in another terminal show up as running in the TUI, and killing works for them.
- Keep TUI-launched games alive when the TUI quits (ask on quit if games are running: "Keep running / Kill all / Cancel").
- Background refresh on focus-gained events; periodic liveness polling every few seconds.
- Errors always appear as a toast or inline text with the same hints as the CLI. Nothing prints raw to the screen outside the buffer (log to the file).
- Mouse details: hover highlights, pointer-style feedback via highlight changes, scroll wheel scrolls the widget under the cursor, drag on scrollbars and sliders, double-click, right-click context menus that stay inside the screen. Verify the behavior in **Windows Terminal**, and note limitations in legacy `conhost` and over SSH/tmux (mouse passthrough settings).

## 4.5 Polish and robustness
- Accessibility: full keyboard operation, a high-contrast mode, `--ascii` mode with no color dependence for meaning (status shown with text/icon and color), screen-reader friendly plain CLI output.
- Robustness: no network at startup (cached data + clear banners), network lost mid-download (resume), corrupt `config.json`/`accounts.json`/`.helio.json` (recover via `.bak`), instance folder deleted externally while the TUI is open, expired/revoked Ely.by token, two launchers touching the same data dir (use a lock file for mutating operations with a clear message), very long and unicode instance names, rapid resize, `SIGWINCH`.
- Performance: measure and report idle CPU of the TUI (target under 2%), startup time (`helio` to first frame under 150 ms with a warm cache), the log view at 20k lines, the mod list with 500 jars, and memory use. Fix jank (dirty-region redraws, virtualization, debouncing).

## 4.6 Tests, CI, release, docs
- **Tests:** keep and extend all earlier unit tests. Add TUI tests using `ratatui`'s `TestBackend` and snapshot tests (`insta`) for the main screens at 80x24 and 140x40, plus `update`-function tests for navigation, hit-testing and mouse event handling. Add CLI integration tests (`assert_cmd`) for `--help`, `--json` validity of every listing command against fixtures, exit codes, and `--dry-run` output.
- **CI (`.github/workflows/ci.yml`):** matrix Windows + Linux (+ macOS if feasible): `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build --release`. A manual/tagged **release workflow** that builds `helio` for `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu` (and `aarch64` targets if feasible), zips each with a SHA256 checksums file, and creates a GitHub Release (`permissions: contents: write`).
- **Install:** an `install.ps1` (downloads the latest release, verifies the checksum, puts `helio.exe` in `%LOCALAPPDATA%\Programs\helio` and adds it to the user `PATH`) and an `install.sh`; also document `cargo install --path crates/helio`. Generate shell completions on install if possible. Document the result of the name check and how to alias the command if the user wants a shorter one.
- **Docs:** `README.md` (what it is, a short demo as asciinema-style text or described screenshots, install, quick start: `helio login`, `helio instance create`, `helio launch`, the TUI key and mouse reference, the Ely.by OAuth app registration steps, the full command reference generated from `clap`, troubleshooting, terminal compatibility notes), `docs/COMMANDS.md` (generated), `docs/ARCHITECTURE.md` (crate layout, event flow, run-file design, launch pipeline with mermaid diagrams), `PROGRESS.md` final.

## 4.7 Icon coverage audit (deliverable)
Produce `docs/ICON_AUDIT.md`: for every screen, tab, menu, dialog, toast kind, status and CLI table, list each element and the semantic icon it uses (generated from the enums so it cannot drift), plus screenshots or described renders in all three icon modes. Any element without an icon is a failure.

## 4.7b Final acceptance checklist (run ALL; report PASS/FAIL with evidence)
1. Fresh start (`helio` with no accounts): login screen; offline login; invalid names are blocked.
2. Ely.by login in the TUI (browser path and paste path); keychain holds the tokens; restart refreshes silently.
3. Create a vanilla 1.20.1 instance from the TUI wizard with a custom icon; it appears in the sidebar in both sidebar states.
4. Launch from the TUI with mouse only (click the instance, click Launch): progress bar, game opens, logs stream in the Logs tab, Kill works from the instance page and the sidebar right-click menu.
5. The same flow through the CLI: `helio launch`, `helio logs -f`, `helio ps`, `helio kill`; `--detach` game appears as running in the TUI.
6. Switch accounts from the account picker; the head updates; an Ely.by launch uses authlib-injector (dry run) and the skin shows in game.
7. Fabric instance: add Fabric API + Sodium from the TUI Browse (dependency plan dialog) and from the CLI; launch.
8. Install a `.mrpack` from the TUI Browse and from `helio modpack install`; verify instance, icon, mods, overrides; launch.
9. Forge 1.20.1, Forge 1.12.2, NeoForge launch (or documented failures with reasons). Legacy 1.8.9 with Java 8 launches.
10. Paste (drag-drop) a jar path onto an instance screen; it lands in `mods/`; toggle disable and enable with a click.
11. Collapse and expand the sidebar with the mouse and with `Ctrl+B`; the state persists.
12. Resources screen shows live CPU/RAM, disk breakdown and a running game's memory; the numbers roughly match Task Manager.
13. Settings changes (memory, JVM args) appear in `helio launch --dry-run`.
14. Command palette runs `launch`, `mod add` and `search`; the help overlay lists all bindings.
15. Offline mode (network off): the TUI starts; installed instances launch with offline accounts; missing downloads show clear errors.
16. 80x24 and 140x40 render correctly; resize works; the terminal is restored after quit, panic (simulate one) and Ctrl+C.
17. `--json` output is valid for every listing command (validate with a JSON parser); exit codes match the table; `NO_COLOR` and `--ascii` work.
18. CI green on Windows and Linux; release workflow produces zips with checksums; `install.ps1` installs and `helio` runs from a new terminal.
19. Repository-wide search finds zero emojis, zero `TODO`/`FIXME`/`unimplemented!` in shipped code, and **zero occurrences of the legacy name** (`OmLauncher`, `omlauncher`, `Om Launcher`, `.omconfig`, `OM_`).
20. `docs/ICON_AUDIT.md` shows full icon coverage; the three icon modes render correctly; the first-run icon check works; no emoji code points exist.
21. The Helio banner, the Sun/Midnight/Daylight palettes and the terminal title work; the binary, installer, CI artifacts, keychain entry, data folder and User-Agent all say Helio.

## 4.8 Final report
Write `FINAL_REPORT.md` and show it: (a) what was built, (b) the acceptance checklist with PASS/FAIL and evidence, (c) the tested version matrix (MC versions, loaders, Java versions), (d) performance numbers, (e) security model (allowlist, secrets, path safety), (f) known limitations stated plainly (terminal and mouse quirks, loader coverage, detached-game playtime approximation, anything unfinished), (g) recommended next steps (Microsoft account login, auto-update for `helio`, profiles, shareable instance presets).

## Definition of Done (Phase 4)
- [ ] All 21 acceptance items pass or have honest documented exceptions.
- [ ] Idle CPU under 2%, startup under 150 ms warm (or the measured numbers with an explanation).
- [ ] Tests, clippy, fmt pass; CI green; release artifacts and installer verified.
- [ ] Docs complete; `FINAL_REPORT.md` written; committed and tagged `v0.1.0`.

---

# RESUME PROMPT (paste if the agent lost context)

```
You are continuing work on Helio, a terminal Minecraft launcher (Rust workspace: helio-core library + helio binary with clap commands and a ratatui TUI; icon-heavy by design; brand constants in helio-core::brand). Re-orient first:
1. Read PROGRESS.md, README.md, and `git log --oneline -20`.
2. Run `git status`, `cargo check --workspace`, and `cargo test --workspace`, and report the results.
3. Tell me: the last completed phase, the current phase, anything half-done, and the known issues.
4. The Standing Orders still apply: no emojis (Nerd Font icons on every element with Unicode and ASCII fallbacks), no stubs or TODOs, no silent failures, all network through the allowlisted core client, verify external APIs against docs, safe paths, secrets only in the keychain, --json and exit codes, run what you build, commit per phase, honest PASS/FAIL reports.
Do not start new work until you have reported this. Then wait for me to paste the next phase.
```
