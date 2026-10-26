# PROGRESS.md — Helio (helio-mc)

## Current phase
**Phase 1** — Foundation, CLI skeleton, accounts (login) — **IN PROGRESS** (not complete)

## Completed phases
- None yet (no tags, no phase commits)

## Commit history
- `239d990` chore: init repo, add spec doc and ignore files (initial repo only, no code)

## Decisions (with reasons)
| Decision | Reason |
|---|---|
| Package names `helio-mc` / `helio-mc-core` | User selected `heliomc`/`helio-mc` style; crates.io `helio` is taken by an unrelated crate. Binary stays `helio`. |
| No legacy name traces | Original project was `OmLauncher`; zero `Om*` occurrences in code/comments. |
| mc-launcher-core v0.1.2 added as dependency | Handles vanilla/Fabric/Quilt/Forge/NeoForge install + launch command building (Phase 2+). Helio keeps its own Ely.by OAuth, keychain, icons, CLI. |
| `OmError` renamed to `HelioError` | Spec says `OmError` but brand rules forbid legacy name traces. |
| No `dirs` crate; env-var-based data dir | `dirs::local_dir`/`data_dir` unavailable in this toolchain version; use `LOCALAPPDATA`/`XDG_DATA_HOME`/`HOME` directly. |

## Known issues
- `cargo clippy -- -D warnings` has 22 remaining clippy errors in `helio-mc-core` (banner, http, paths, theme, elyby, keychain, skins modules). Not yet fixed.
- `helio-mc` binary has 20 clippy warnings (unused variables in stub commands). Not yet fixed.
- `cargo test --workspace` passes for `helio-mc-core` (45 tests) but binary stub commands return "not implemented" errors as designed.
- `helio login --elyby` browser/paste flow is implemented but **not tested end-to-end** (needs a real Ely.by client ID + interactive browser).
- `helio skin`, `helio doctor --icons`, `helio version --banner` not yet exercised.
- No PROGRESS.md existed before this file; no `FINAL_REPORT.md`, no `docs/`, no CI workflows.

## Test evidence
- `cargo test -p helio-mc-core` → **45 passed, 0 failed** (paths, config, theme, icons, accounts/offline/keychain/skins/elyby unit tests).
- `cargo check --workspace` → **compiles clean** (0 errors, warnings only).
- Offline UUID reference vector independently verified: `offline_uuid("Steve")` = `5627dd98-e6be-3c21-b8a8-e92344183641` (MD5 of `OfflinePlayer:Steve` = `5627dd98e6bebc21f8a8e92344183641`, version/variant bits applied).
- `safe_join` rejects `../x`, `..\x`, `a/../../b`, absolute paths, drive paths, reserved Windows names (`CON`, `NUL`, `COM1`, `aux.txt`).
- `sanitize_name` collapses spaces→dashes, truncates to 40 chars, handles reserved names (`NUL`→`NUL_`), rejects `!!!`→`unnamed`.
- `unique_slug` appends `-2`, `-3`, ... on collision.
- `read_json_or_default` recovers corrupt JSON via `.bak`.
- `write_json_atomic` writes temp then rename.
- Icon registry: 100+ icons, no emoji code points, Nerd glyphs all in PUA range, width-correct in all three modes.
- Ely.by OAuth endpoints verified against docs.ely.by (2026-10-26): authorize `https://account.ely.by/oauth2/v1`, token `https://account.ely.by/api/oauth2/v1/token`, user-info `https://account.ely.by/api/account/v1/info`, scopes `account_info offline_access minecraft_server_session`.
- authlib-injector: official GitHub releases at `github.com/yushijinhun/authlib-injector`, Ely.by docs reference `https://authlib-injector.yushi.moe/~download` and javaagent `=ely.by`.
- Winget Nerd Font package ID verified: `DEVCOM.JetBrainsMonoNerdFont`.

## External facts verified (endpoint, source URL, date)
| Fact | Source | Date |
|---|---|---|
| Ely.by OAuth authorize URL | https://docs.ely.by/en/oauth.html | 2026-10-26 |
| Ely.by OAuth token URL | https://docs.ely.by/en/oauth.html | 2026-10-26 |
| Ely.by user-info URL | https://docs.ely.by/en/oauth.html | 2026-10-26 |
| Ely.by scopes (`account_info`, `offline_access`, `minecraft_server_session`) | https://docs.ely.by/en/oauth.html | 2026-10-26 |
| Ely.by authlib-injector javaagent `=ely.by` | https://docs.ely.by/en/authlib-injector.html | 2026-10-26 |
| authlib-injector download API | https://authlib-injector.yushi.moe/ | 2026-10-26 |
| Ely.by skins system URLs | https://docs.ely.by/en/skins-system.html | 2026-10-26 |
| mc-launcher-core v0.1.2 API | https://docs.rs/mc-launcher-core/0.1.2 | 2026-10-26 |
| Winget Nerd Font package ID | https://winstall.app/apps/DEVCOM.JetBrainsMonoNerdFont | 2026-10-26 |

## What you need before the next phase
1. A real Ely.by OAuth client ID + secret (app named "Helio" at account.ely.by/dev/applications) to test the browser + paste login flows end-to-end.
2. Fix the 22 clippy errors in `helio-mc-core` and 20 warnings in `helio-mc` (run `cargo clippy --workspace -- -D warnings`).
3. Java 8 for Phase 2 legacy 1.8.9 launch tests (can use `helio java install 8` via Adoptium).
4. Decide whether to keep package names `helio-mc`/`helio-mc-core` or switch to `helio-launcher`/`helio-launcher-core` if crates.io `helio` is confirmed taken.

## Files written so far (Phase 1, partial)
```
Cargo.toml                      # workspace root
crates/helio-mc-core/Cargo.toml # core lib manifest
crates/helio-mc-core/build.rs   # repo URL at build time
crates/helio-mc-core/src/lib.rs
crates/helio-mc-core/src/brand.rs
crates/helio-mc-core/src/error.rs
crates/helio-mc-core/src/paths.rs
crates/helio-mc-core/src/config.rs
crates/helio-mc-core/src/http.rs
crates/helio-mc-core/src/events.rs
crates/helio-mc-core/src/logging.rs
crates/helio-mc-core/src/logredact.rs
crates/helio-mc-core/src/ui.rs
crates/helio-mc-core/src/icons.rs
crates/helio-mc-core/src/banner.rs
crates/helio-mc-core/src/theme.rs
crates/helio-mc-core/src/accounts.rs
crates/helio-mc-core/src/accounts/models.rs
crates/helio-mc-core/src/accounts/keychain.rs
crates/helio-mc-core/src/accounts/offline.rs
crates/helio-mc-core/src/accounts/elyby.rs
crates/helio-mc-core/src/accounts/skins.rs
crates/helio-mc/Cargo.toml      # binary manifest
crates/helio-mc/src/main.rs     # full CLI skeleton + login/account/config/doctor/icons/version/completions
```
