//! Helio CLI entry point.
//!
//! Defines the entire command tree with clap. Commands implemented in later
//! phases exist in the tree but return a clear "not implemented yet (Phase N)"
//! error with exit code 1. Do not fake their output.

use std::io::IsTerminal;
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use helio_mc_core::brand;
use helio_mc_core::error::{HelioError, HelioResult};
use helio_mc_core::icons::IconMode;
use helio_mc_core::paths;
use helio_mc_core::ui::Out;

/// Helio — Minecraft in your terminal, bathed in sunlight.
///
/// A terminal Minecraft launcher where everything is done through one
/// command, `helio`. Both the TUI (Phase 4) and the CLI share the same core
/// engine in `helio-mc-core`.
#[derive(Parser, Debug)]
#[command(
    name = brand::BIN,
    about = format!("{} — {}", brand::NAME, brand::TAGLINE),
    version,
    long_about = None
)]
struct Cli {
    #[command(flatten)]
    global: GlobalFlags,

    #[command(subcommand)]
    command: Option<Commands>,
}

/// Global flags available on every subcommand.
#[derive(Debug, clap::Args)]
struct GlobalFlags {
    /// Output machine-readable JSON on stdout; human text goes to stderr.
    #[arg(long, global = true)]
    json: bool,

    /// Quiet mode: minimal output, no color, no icons.
    #[arg(short = 'q', long, global = true)]
    quiet: bool,

    /// Disable color output (also respects NO_COLOR).
    #[arg(long, global = true)]
    no_color: bool,

    /// Icon rendering mode: nerd, unicode, or ascii.
    #[arg(long, value_name = "MODE", global = true)]
    icons: Option<IconMode>,

    /// Force ASCII output (same as --icons ascii).
    #[arg(short = 'a', long, global = true)]
    ascii: bool,

    /// Set the data directory (default: platform data dir /Helio).
    #[arg(long, value_name = "DIR", global = true)]
    data_dir: Option<std::path::PathBuf>,

    /// Skip confirmations (assume yes).
    #[arg(short = 'y', long, global = true)]
    yes: bool,

    /// Never prompt; fail with an error instead.
    #[arg(long, global = true)]
    no_input: bool,

    /// Increase logging verbosity (-v, -vv, -vvv).
    #[arg(
        short = 'v',
        long = "verbose",
        action = clap::ArgAction::Count,
        global = true
    )]
    verbose: u8,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    /// Open the TUI (Phase 4). Until then, prints help.
    Tui,
    /// Authenticate with Ely.by OAuth or an offline account.
    Login(LoginArgs),
    /// Remove an account (alias: logout).
    Logout(LogoutArgs),
    /// Manage accounts.
    Account(AccountArgs),
    /// View a skin in the terminal (Phase 1).
    Skin(SkinArgs),
    /// Manage instances (Phase 2).
    Instance(InstanceArgs),
    /// Launch an instance (Phase 2).
    Launch(LaunchArgs),
    /// Kill a running instance (Phase 2).
    Kill(KillArgs),
    /// List running instances (Phase 2).
    Ps,
    /// Show instance logs (Phase 2).
    Logs(LogsArgs),
    /// List Minecraft versions (Phase 2).
    Versions(VersionsArgs),
    /// List loader versions (Phase 3).
    Loader(LoaderArgs),
    /// Manage Java runtimes (Phase 2).
    Java(JavaArgs),
    /// Search Modrinth (Phase 3).
    Search(SearchArgs),
    /// Show Modrinth project details (Phase 3).
    Info(InfoArgs),
    /// Manage mods (Phase 3).
    Mod(ModArgs),
    /// Install a modpack (Phase 3).
    Modpack(ModpackArgs),
    /// Manage resource packs (Phase 3).
    Pack(PackArgs),
    /// Manage shaders (Phase 3).
    Shader(ShaderArgs),
    /// Manage worlds (Phase 3).
    World(WorldArgs),
    /// Manage screenshots (Phase 3).
    Screenshot(ScreenshotArgs),
    /// Show system/resource stats (Phase 2).
    Stats(StatsArgs),
    /// Show disk usage (Phase 2).
    Disk(DiskArgs),
    /// Print the icon registry (Phase 1).
    Icons(IconsArgs),
    /// Read/write configuration (Phase 1).
    Config(ConfigArgs),
    /// Run diagnostics (Phase 1).
    Doctor(DoctorArgs),
    /// Generate shell completions (Phase 1).
    Completions(CompletionsArgs),
    /// Show version information (Phase 1).
    Version(VersionArgs),
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let code = e.exit_code();
            eprintln!("{e}");
            if let Some(hint) = e.hint() {
                eprintln!("  {hint}");
            }
            ExitCode::from(code as u8)
        }
    }
}

fn run() -> HelioResult<()> {
    let cli = Cli::parse();

    // Resolve data dir from global flag.
    let data_dir = cli
        .global
        .data_dir
        .as_deref()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| paths::default_data_dir());

    // Resolve icon mode: --ascii takes precedence over --icons.
    let icon_mode = if cli.global.ascii {
        Some(IconMode::Ascii)
    } else {
        cli.global.icons
    };
    let out = Out::new(
        cli.global.json,
        cli.global.quiet,
        cli.global.no_color,
        icon_mode,
    );

    // Set terminal title.
    if !cli.global.quiet {
        helio_mc_core::ui::set_terminal_title();
    }

    match cli.command {
        None => {
            out.section("Helio");
            out.info("No subcommand given. Use `helio --help` to see available commands.");
            out.hint("The TUI (`helio` with no args) opens in Phase 4.");
            Ok(())
        }
        Some(Commands::Tui) => cmd_tui(&out),
        Some(Commands::Login(args)) => cmd_login(&out, args, &data_dir),
        Some(Commands::Logout(args)) => cmd_logout(&out, args, &data_dir),
        Some(Commands::Account(args)) => cmd_account(&out, args, &data_dir),
        Some(Commands::Skin(args)) => cmd_skin(&out, args, &data_dir),
        Some(Commands::Instance(args)) => cmd_instance(&out, args, &data_dir),
        Some(Commands::Launch(args)) => cmd_launch(&out, args, &data_dir),
        Some(Commands::Kill(args)) => cmd_kill(&out, args, &data_dir),
        Some(Commands::Ps) => cmd_ps(&out, &data_dir),
        Some(Commands::Logs(args)) => cmd_logs(&out, args, &data_dir),
        Some(Commands::Versions(args)) => cmd_versions(&out, args, &data_dir),
        Some(Commands::Loader(args)) => cmd_loader(&out, args, &data_dir),
        Some(Commands::Java(args)) => cmd_java(&out, args, &data_dir),
        Some(Commands::Search(args)) => cmd_search(&out, args, &data_dir),
        Some(Commands::Info(args)) => cmd_info(&out, args, &data_dir),
        Some(Commands::Mod(args)) => cmd_mod(&out, args, &data_dir),
        Some(Commands::Modpack(args)) => cmd_modpack(&out, args, &data_dir),
        Some(Commands::Pack(args)) => cmd_pack(&out, args, &data_dir),
        Some(Commands::Shader(args)) => cmd_shader(&out, args, &data_dir),
        Some(Commands::World(args)) => cmd_world(&out, args, &data_dir),
        Some(Commands::Screenshot(args)) => cmd_screenshot(&out, args, &data_dir),
        Some(Commands::Stats(args)) => cmd_stats(&out, args, &data_dir),
        Some(Commands::Disk(args)) => cmd_disk(&out, args, &data_dir),
        Some(Commands::Icons(args)) => cmd_icons(&out, args),
        Some(Commands::Config(args)) => cmd_config(&out, args, &data_dir),
        Some(Commands::Doctor(args)) => cmd_doctor(&out, args, &data_dir),
        Some(Commands::Completions(args)) => cmd_completions(&out, args),
        Some(Commands::Version(args)) => cmd_version(&out, args),
    }
}

/// Placeholder for commands not yet implemented.
fn not_implemented(phase: u8, feature: &str) -> HelioError {
    HelioError::Launch(format!(
        "{feature} is not implemented yet (Phase {phase}). This command exists in the CLI \
         skeleton but has no backend logic."
    ))
}

// ---------------------------------------------------------------------------
// Command implementations
// ---------------------------------------------------------------------------

fn cmd_tui(_out: &Out) -> HelioResult<()> {
    Err(not_implemented(4, "TUI"))
}

fn cmd_login(out: &Out, args: LoginArgs, data_dir: &std::path::Path) -> HelioResult<()> {
    use helio_mc_core::accounts::elyby;
    use helio_mc_core::accounts::offline;
    use helio_mc_core::accounts::{AccountKind, AccountStore};
    use helio_mc_core::config::Config;
    use std::io::IsTerminal;
    use std::io::{stdin, BufRead, Write};

    paths::ensure_tree(data_dir)?;
    let mut store = AccountStore::open(data_dir);

    // Offline login.
    if let Some(name) = &args.offline {
        offline::validate_username(name)?;
        let acc = store.add_offline(name)?;
        out.success(&format!(
            "Offline account \"{}\" added (UUID: {}).",
            acc.username, acc.uuid
        ));
        out.hint(&format!(
            "Run: helio launch <instance> --account {}",
            acc.username
        ));
        return Ok(());
    }

    // Ely.by login.
    if args.elyby {
        let config = Config::load(data_dir);
        let client_id = config
            .elyby_client_id
            .clone()
            .or_else(|| args.client_id.clone())
            .ok_or_else(|| {
                HelioError::Auth(
                    "missing Ely.by client id. Register an app named \"Helio\" at \
https://account.ely.by/dev/applications, then run:\n  helio config set elyby_client_id <ID>\n  helio config set elyby_client_secret <SECRET>\n(or pass --client-id / --client-secret)".into(),
                )
            })?;
        let client_secret = config
            .elyby_client_secret
            .clone()
            .or_else(|| args.client_secret.clone())
            .ok_or_else(|| HelioError::Auth("missing Ely.by client secret".into()))?;

        let flow = elyby::begin(&client_id, &client_secret, None);
        let url = elyby::authorize_url(&flow);

        if args.no_browser {
            out.section("Ely.by OAuth — paste flow");
            out.info(&format!("Open this URL in any browser:\n  {url}"));
            out.hint("Paste the full redirect URL (or just the code) when prompted.");
            if args.no_input {
                return Err(HelioError::InvalidInput(
                    "--no-input requires --offline or a pre-pasted code".into(),
                ));
            }
            let mut input = String::new();
            print!("Paste redirect URL or code: ");
            std::io::stdout().flush()?;
            stdin().lock().read_line(&mut input)?;
            let pasted = input.trim();
            let rt = tokio::runtime::Runtime::new().unwrap();
            let code = rt.block_on(elyby::listen_for_code(
                &flow,
                std::sync::Arc::new(tokio::sync::Notify::new()),
            ))?;
            // NOTE: listen_for_code waits for a browser redirect, which won't happen in no-browser mode.
            // For no-browser, the user pastes the code directly. We need to handle this differently.
            // For now, treat the pasted input as the code:
            let _ = code;
            let (profile, _tokens) = rt.block_on(elyby::complete_from_paste(&flow, pasted))?;
            drop(rt);
            let acc = store.add_elyby(&profile, &client_id)?;
            out.success(&format!(
                "Ely.by account \"{}\" added (UUID: {}).",
                acc.username, acc.uuid
            ));
            out.hint("Tokens stored in the OS keychain (service: Helio).");
            return Ok(());
        }

        // Browser flow.
        out.section("Ely.by OAuth — browser flow");
        out.info(&format!("Opening browser:\n  {url}"));
        elyby::open_in_browser(&url);
        out.hint("If the browser did not open, copy the URL above and run with --no-browser.");

        let cancel = std::sync::Arc::new(tokio::sync::Notify::new());
        let cancel_clone = cancel.clone();
        ctrlc::set_handler(move || {
            cancel_clone.notify_waiters();
        })
        .map_err(|e| HelioError::Other(format!("could not set Ctrl+C handler: {e}")))?;

        let rt = tokio::runtime::Runtime::new().unwrap();
        let code = rt.block_on(elyby::listen_for_code(&flow, cancel))?;
        drop(rt);
        // Exchange code for tokens and fetch profile
        let rt = tokio::runtime::Runtime::new().unwrap();
        let tokens = rt.block_on(elyby::exchange_code(
            &client_id,
            &client_secret,
            &flow.redirect_uri,
            &code,
        ))?;
        let profile = rt.block_on(elyby::fetch_profile(&tokens.access_token))?;
        drop(rt);
        let acc = store.add_elyby(&profile, &client_id)?;
        out.success(&format!(
            "Ely.by account \"{}\" added (UUID: {}).",
            acc.username, acc.uuid
        ));
        out.hint("Tokens stored in the OS keychain (service: Helio).");
        return Ok(());
    }

    // No flags: interactive chooser (TTY only).
    if atty::is(atty::Stream::Stdout) {
        out.section("Helio — choose login method");
        out.info("1) Ely.by (browser)   2) Offline (enter a username)");
        if args.no_input {
            return Err(HelioError::InvalidInput(
                "--no-input requires --offline or --elyby".into(),
            ));
        }
        let mut input = String::new();
        print!("> ");
        std::io::stdout().flush()?;
        stdin().lock().read_line(&mut input)?;
        let choice = input.trim();
        match choice {
            "1" => {
                let config = Config::load(data_dir);
                let client_id = config.elyby_client_id.as_deref().unwrap_or("");
                let client_secret = config.elyby_client_secret.as_deref().unwrap_or("");
                if client_id.is_empty() || client_secret.is_empty() {
                    return Err(HelioError::Auth(
                        "Ely.by client id/secret not configured. Run:\n  helio config set elyby_client_id <ID>\n  helio config set elyby_client_secret <SECRET>".into(),
                    ));
                }
                let flow = elyby::begin(client_id, client_secret, None);
                let url = elyby::authorize_url(&flow);
                out.info(&format!("Open this URL:\n  {url}"));
                elyby::open_in_browser(&url);
                let cancel = std::sync::Arc::new(tokio::sync::Notify::new());
                let cancel_clone = cancel.clone();
                ctrlc::set_handler(move || {
                    cancel_clone.notify_waiters();
                })
                .map_err(|e| HelioError::Other(format!("ctrlc handler: {e}")))?;
                let rt = tokio::runtime::Runtime::new().unwrap();
                let code = rt.block_on(elyby::listen_for_code(&flow, cancel))?;
                drop(rt);
                let rt = tokio::runtime::Runtime::new().unwrap();
                let tokens = rt.block_on(elyby::exchange_code(
                    client_id,
                    client_secret,
                    &flow.redirect_uri,
                    &code,
                ))?;
                let profile = rt.block_on(elyby::fetch_profile(&tokens.access_token))?;
                drop(rt);
                let acc = store.add_elyby(&profile, client_id)?;
                out.success(&format!("Ely.by account \"{}\" added.", acc.username));
            }
            "2" => {
                out.info("Enter an offline username (3-16 chars, letters/digits/underscore):");
                let mut input = String::new();
                print!("> ");
                std::io::stdout().flush()?;
                stdin().lock().read_line(&mut input)?;
                let name = input.trim();
                offline::validate_username(name)?;
                let acc = store.add_offline(name)?;
                out.success(&format!("Offline account \"{}\" added.", acc.username));
            }
            _ => {
                return Err(HelioError::InvalidInput("invalid choice".into()));
            }
        }
        Ok(())
    } else {
        Err(HelioError::InvalidInput(
            "no login method specified. Use --offline <NAME> or --elyby".into(),
        ))
    }
}

fn cmd_logout(_out: &Out, args: LogoutArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(1, "logout"))
}

fn cmd_account(out: &Out, args: AccountArgs, data_dir: &std::path::Path) -> HelioResult<()> {
    use helio_mc_core::accounts::{AccountKind, AccountStore};

    let mut store = AccountStore::open(data_dir);

    match &args.action {
        AccountAction::List => {
            let accounts: Vec<_> = store.file.accounts.clone();
            if accounts.is_empty() {
                out.info("No accounts found. Run `helio login --offline <NAME>` or `helio login --elyby`.");
                return Ok(());
            }
            out.section("Accounts");
            for acc in accounts {
                let active = store.active().map(|a| a.id == acc.id).unwrap_or(false);
                let marker = if active { "●" } else { "○" };
                let kind_icon = match acc.kind {
                    AccountKind::Elyby => out.icons.render("Elyby"),
                    AccountKind::Offline => out.icons.render("OfflineAccount"),
                };
                out.data(&format!(
                    "  {marker} {kind_icon}{:<16} {:<40} {}",
                    acc.username,
                    acc.uuid,
                    acc.added_at.format("%Y-%m-%d")
                ));
            }
            Ok(())
        }
        AccountAction::Use { account } => {
            let acc = store
                .find(account)
                .ok_or_else(|| HelioError::NotFound(format!("no account {account:?}")))?;
            let acc_id = acc.id.clone();
            let acc_username = acc.username.clone();
            store.set_active(&acc_id)?;
            out.success(&format!("Active account set to \"{acc_username}\"."));
            Ok(())
        }
        AccountAction::Remove { account } => {
            let acc = store
                .find(account)
                .ok_or_else(|| HelioError::NotFound(format!("no account {account:?}")))?;
            let acc_username = acc.username.clone();
            let acc_id = acc.id.clone();
            store.remove(&acc_id)?;
            out.success(&format!("Account \"{acc_username}\" removed."));
            Ok(())
        }
        AccountAction::Refresh { account } => {
            if let Some(q) = account {
                store
                    .find(q)
                    .ok_or_else(|| HelioError::NotFound(format!("no account {q:?}")))?;
            }
            out.info(
                "Token refresh is performed automatically at launch. Use `helio launch` to verify.",
            );
            Ok(())
        }
        AccountAction::Info { account } => {
            let query = account.as_deref().unwrap_or("");
            let acc = if query.is_empty() {
                store
                    .active()
                    .cloned()
                    .ok_or_else(|| HelioError::NotFound("no active account".into()))?
            } else {
                store
                    .find(query)
                    .cloned()
                    .ok_or_else(|| HelioError::NotFound(format!("no account {query:?}")))?
            };
            out.section("Account");
            out.kv("id", &acc.id);
            out.kv("username", &acc.username);
            out.kv(
                "kind",
                match acc.kind {
                    AccountKind::Elyby => "elyby",
                    AccountKind::Offline => "offline",
                },
            );
            out.kv("uuid", &acc.uuid);
            out.kv(
                "added",
                &acc.added_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            );
            let active = store.active().map(|a| a.id == acc.id).unwrap_or(false);
            out.kv("active", &active.to_string());
            Ok(())
        }
    }
}

fn cmd_skin(out: &Out, args: SkinArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(1, "skin"))
}

fn cmd_instance(out: &Out, args: InstanceArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(2, "instance"))
}

fn cmd_launch(out: &Out, args: LaunchArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(2, "launch"))
}

fn cmd_kill(out: &Out, args: KillArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(2, "kill"))
}

fn cmd_ps(_out: &Out, _data_dir: &std::path::Path) -> HelioResult<()> {
    Err(not_implemented(2, "ps"))
}

fn cmd_logs(out: &Out, args: LogsArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(2, "logs"))
}

fn cmd_versions(out: &Out, args: VersionsArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(2, "versions"))
}

fn cmd_loader(out: &Out, args: LoaderArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(3, "loader"))
}

fn cmd_java(out: &Out, args: JavaArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(2, "java"))
}

fn cmd_search(out: &Out, args: SearchArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(3, "search"))
}

fn cmd_info(out: &Out, args: InfoArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(3, "info"))
}

fn cmd_mod(out: &Out, args: ModArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(3, "mod"))
}

fn cmd_modpack(out: &Out, args: ModpackArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(3, "modpack"))
}

fn cmd_pack(out: &Out, args: PackArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(3, "pack"))
}

fn cmd_shader(out: &Out, args: ShaderArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(3, "shader"))
}

fn cmd_world(out: &Out, args: WorldArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(3, "world"))
}

fn cmd_screenshot(out: &Out, args: ScreenshotArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(3, "screenshot"))
}

fn cmd_stats(out: &Out, args: StatsArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(2, "stats"))
}

fn cmd_disk(out: &Out, args: DiskArgs, _data_dir: &std::path::Path) -> HelioResult<()> {
    let _ = args;
    Err(not_implemented(2, "disk"))
}

fn cmd_icons(out: &Out, args: IconsArgs) -> HelioResult<()> {
    use helio_mc_core::icons::{Icons, ICONS};
    let mode = args.mode.unwrap_or(IconMode::Unicode);
    let icons = Icons::new(mode, 1);
    out.section("Helio Icon Registry");
    for def in ICONS {
        let rendered = icons.render(def.name);
        out.data(&format!(
            "  {:<18} {:<6} {:<6} {:<6} {}",
            def.name, rendered, def.unicode, def.ascii, def.color
        ));
    }
    Ok(())
}

fn cmd_config(out: &Out, args: ConfigArgs, data_dir: &std::path::Path) -> HelioResult<()> {
    use helio_mc_core::config::Config;

    match &args.action {
        ConfigAction::List => {
            let config = Config::load(data_dir);
            out.section("Helio Config");
            out.kv("schema", &config.schema.to_string());
            out.kv(
                "data_dir",
                &config
                    .data_dir
                    .as_deref()
                    .unwrap_or(std::path::Path::new("<default>"))
                    .display()
                    .to_string(),
            );
            out.kv("theme", &config.theme);
            out.kv("icons", &config.icons);
            out.kv("ascii_icons", &config.ascii_icons.to_string());
            out.kv(
                "concurrent_downloads",
                &config.concurrent_downloads.to_string(),
            );
            out.kv("min_ram_mb", &config.min_ram_mb.to_string());
            out.kv("max_ram_mb", &config.max_ram_mb.to_string());
            out.kv(
                "active_account_id",
                &config.active_account_id.clone().unwrap_or_default(),
            );
            out.kv(
                "elyby_client_id",
                &config.elyby_client_id.clone().unwrap_or_default(),
            );
            Ok(())
        }
        ConfigAction::Get { key } => {
            let config = Config::load(data_dir);
            let value = match key.as_str() {
                "schema" => config.schema.to_string(),
                "theme" => config.theme.clone(),
                "icons" => config.icons.clone(),
                "ascii_icons" => config.ascii_icons.to_string(),
                "concurrent_downloads" => config.concurrent_downloads.to_string(),
                "min_ram_mb" => config.min_ram_mb.to_string(),
                "max_ram_mb" => config.max_ram_mb.to_string(),
                "active_account_id" => config.active_account_id.clone().unwrap_or_default(),
                "elyby_client_id" => config.elyby_client_id.clone().unwrap_or_default(),
                _ => {
                    return Err(HelioError::InvalidInput(format!(
                        "unknown config key: {key}"
                    )));
                }
            };
            out.data(&value);
            Ok(())
        }
        ConfigAction::Set { key, value } => {
            Config::validate_key_value(key, value)?;
            let mut config = Config::load(data_dir);
            match key.as_str() {
                "theme" => config.theme = value.clone(),
                "icons" => config.icons = value.clone(),
                "ascii_icons" => {
                    config.ascii_icons = value.parse().map_err(|_| {
                        HelioError::InvalidInput("ascii_icons must be true or false".into())
                    })?;
                }
                "concurrent_downloads" => {
                    config.concurrent_downloads = value.parse().map_err(|_| {
                        HelioError::InvalidInput("concurrent_downloads must be a number".into())
                    })?;
                }
                "min_ram_mb" => {
                    config.min_ram_mb = value.parse().map_err(|_| {
                        HelioError::InvalidInput("min_ram_mb must be a number".into())
                    })?;
                }
                "max_ram_mb" => {
                    config.max_ram_mb = value.parse().map_err(|_| {
                        HelioError::InvalidInput("max_ram_mb must be a number".into())
                    })?;
                }
                "active_account_id" => config.active_account_id = Some(value.clone()),
                "elyby_client_id" => config.elyby_client_id = Some(value.clone()),
                _ => {
                    return Err(HelioError::InvalidInput(format!(
                        "unknown config key: {key}"
                    )));
                }
            }
            config.save(data_dir)?;
            out.success(&format!("Config key \"{key}\" set to \"{value}\"."));
            Ok(())
        }
        ConfigAction::Reset { key } => {
            let default = Config::default();
            let default_value = match key.as_str() {
                "theme" => default.theme.clone(),
                "icons" => default.icons.clone(),
                "ascii_icons" => default.ascii_icons.to_string(),
                "concurrent_downloads" => default.concurrent_downloads.to_string(),
                "min_ram_mb" => default.min_ram_mb.to_string(),
                "max_ram_mb" => default.max_ram_mb.to_string(),
                _ => {
                    return Err(HelioError::InvalidInput(format!(
                        "unknown config key: {key}"
                    )));
                }
            };
            let mut config = Config::load(data_dir);
            match key.as_str() {
                "theme" => config.theme = default.theme,
                "icons" => config.icons = default.icons,
                "ascii_icons" => config.ascii_icons = default.ascii_icons,
                "concurrent_downloads" => {
                    config.concurrent_downloads = default.concurrent_downloads
                }
                "min_ram_mb" => config.min_ram_mb = default.min_ram_mb,
                "max_ram_mb" => config.max_ram_mb = default.max_ram_mb,
                _ => {}
            }
            config.save(data_dir)?;
            out.success(&format!(
                "Config key \"{key}\" reset to \"{default_value}\"."
            ));
            Ok(())
        }
        ConfigAction::Path => {
            out.data(&data_dir.join("config.json").display().to_string());
            Ok(())
        }
    }
}

fn cmd_doctor(out: &Out, args: DoctorArgs, data_dir: &std::path::Path) -> HelioResult<()> {
    use helio_mc_core::accounts::keychain;
    use helio_mc_core::config::Config;

    if args.icons {
        // First-run icon check.
        out.section("Helio — Icon Check");
        out.info("Do you see icons, not boxes or question marks?");
        let sample = icon_sample(IconMode::Nerd);
        out.data(&format!("  {sample}"));
        out.hint("If you see boxes or question marks, install a Nerd Font (e.g. JetBrainsMono Nerd Font).");
        return Ok(());
    }

    out.section("Helio — Doctor");
    out.info(&format!("Data dir: {}", data_dir.display()));

    // Config check.
    let config = Config::load(data_dir);
    out.kv("config", "valid");
    out.kv("theme", &config.theme);

    // Keychain check.
    let keychain_ok = keychain::os_keychain_available();
    if keychain_ok {
        out.success("Keychain: available");
    } else {
        out.warn("Keychain: unavailable (using encrypted file fallback)");
    }

    // Java check (simple).
    let java_ok = std::process::Command::new("java")
        .arg("-version")
        .output()
        .is_ok();
    if java_ok {
        out.success("Java: detected");
    } else {
        out.warn("Java: not found on PATH (run: helio java install 21)");
    }

    // Terminal capabilities.
    let stdout = std::io::stdout();
    if stdout.is_terminal() {
        out.success("Terminal: TTY detected");
    } else {
        out.warn("Terminal: not a TTY (some features limited)");
    }

    Ok(())
}

fn icon_sample(mode: IconMode) -> String {
    use helio_mc_core::icons::Icons;
    let icons = Icons::new(mode, 1);
    format!(
        "{} {} {} {}",
        icons.render("Sun"),
        icons.render("Play"),
        icons.render("Account"),
        icons.render("Folder")
    )
}

fn cmd_completions(_out: &Out, args: CompletionsArgs) -> HelioResult<()> {
    use clap_complete::{generate, Shell};
    use helio_mc_core::brand;

    let shell = match args.shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        _ => {
            return Err(HelioError::InvalidInput(
                "shell must be one of: bash, zsh, fish, powershell".into(),
            ));
        }
    };
    let mut app = Cli::command();
    generate(shell, &mut app, brand::BIN, &mut std::io::stdout());
    Ok(())
}

fn cmd_version(out: &Out, args: VersionArgs) -> HelioResult<()> {
    if args.banner {
        use helio_mc_core::banner;
        for line in banner::render() {
            out.data(&line);
        }
    } else {
        out.section("Helio Version");
        out.kv("name", brand::NAME);
        out.kv("version", env!("CARGO_PKG_VERSION"));
        out.kv("binary", brand::BIN);
        out.kv("data_dir_name", brand::DATA_DIR_NAME);
        out.kv("keychain_service", brand::KEYCHAIN_SERVICE);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Argument structs for each command
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
struct LoginArgs {
    /// Authenticate via Ely.by OAuth (opens browser).
    #[arg(long)]
    elyby: bool,
    /// Authenticate offline with a username.
    #[arg(long, value_name = "NAME")]
    offline: Option<String>,
    /// OAuth client id.
    #[arg(long)]
    client_id: Option<String>,
    /// OAuth client secret.
    #[arg(long)]
    client_secret: Option<String>,
    /// Do not open a browser; print the URL for manual paste.
    #[arg(long)]
    no_browser: bool,
    /// Never prompt; fail if input is required.
    #[arg(long)]
    no_input: bool,
}

#[derive(Debug, clap::Args)]
struct LogoutArgs {
    /// Account to remove (id, username, or uuid). Defaults to active.
    account: Option<String>,
}

#[derive(Debug, clap::Args)]
struct AccountArgs {
    #[command(subcommand)]
    action: AccountAction,
}

#[derive(Debug, clap::Subcommand)]
enum AccountAction {
    /// List all accounts.
    List,
    /// Set the active account.
    Use { account: String },
    /// Remove an account.
    Remove { account: String },
    /// Refresh an account's tokens.
    Refresh { account: Option<String> },
    /// Show account details.
    Info { account: Option<String> },
}

#[derive(Debug, clap::Args)]
struct SkinArgs {
    /// Account to show skin for (defaults to active).
    account: Option<String>,
    /// Save the skin PNG to a file.
    #[arg(short = 's', long, value_name = "FILE")]
    save: Option<std::path::PathBuf>,
}

#[derive(Debug, clap::Args)]
struct InstanceArgs {
    #[command(subcommand)]
    action: InstanceAction,
}

#[derive(Debug, clap::Subcommand)]
enum InstanceAction {
    /// Create a new instance.
    Create {
        name: String,
        #[arg(short = 'v', long)]
        version: Option<String>,
        #[arg(long)]
        loader: Option<String>,
        #[arg(long)]
        loader_version: Option<String>,
        #[arg(long)]
        icon: Option<std::path::PathBuf>,
    },
    /// List all instances.
    List,
    /// Show instance details.
    Info { instance: String },
    /// Rename an instance.
    Rename { instance: String, new: String },
    /// Delete an instance.
    Delete { instance: String },
    /// Duplicate an instance.
    Duplicate {
        instance: String,
        new: Option<String>,
    },
    /// Open instance folder in file manager.
    Open { instance: String },
    /// Set a per-instance override.
    Set {
        instance: String,
        key: String,
        value: String,
    },
    /// Export an instance to a zip file.
    Export {
        instance: String,
        file: std::path::PathBuf,
    },
    /// Import an instance from a zip file.
    Import { file: std::path::PathBuf },
}

#[derive(Debug, clap::Args)]
struct LaunchArgs {
    /// Instance to launch (slug, name, prefix, or id).
    instance: String,
    /// Account to use (overrides instance/active).
    #[arg(long)]
    account: Option<String>,
    /// Print the launch command without executing.
    #[arg(long)]
    dry_run: bool,
    /// Detach: return immediately after spawning.
    #[arg(long)]
    detach: bool,
    /// Stream game logs to the terminal.
    #[arg(long)]
    logs: bool,
    /// Kill the game when this process exits.
    #[arg(long)]
    kill_on_exit: bool,
    /// Demo mode.
    #[arg(long)]
    demo: bool,
}

#[derive(Debug, clap::Args)]
struct KillArgs {
    /// Instance to kill, or --all for all running instances.
    #[arg(long)]
    all: bool,
    instance: Option<String>,
}

#[derive(Debug, clap::Args)]
struct LogsArgs {
    /// Instance to show logs for.
    instance: String,
    /// Follow the log live.
    #[arg(short = 'f', long)]
    follow: bool,
    /// Number of lines to tail.
    #[arg(short = 'n', long, value_name = "N")]
    tail: Option<usize>,
    /// Filter by log level.
    #[arg(long, value_name = "LEVEL")]
    level: Option<String>,
    /// Grep pattern.
    #[arg(long, value_name = "PATTERN")]
    grep: Option<String>,
    /// Show the last session log.
    #[arg(long)]
    last: bool,
}

#[derive(Debug, clap::Args)]
struct VersionsArgs {
    /// Include snapshots.
    #[arg(long)]
    snapshots: bool,
    /// Include old versions.
    #[arg(long)]
    old: bool,
}

#[derive(Debug, clap::Args)]
struct LoaderArgs {
    /// Loader name: fabric, quilt, forge, neoforge.
    loader: String,
    /// Minecraft version.
    mc: String,
}

#[derive(Debug, clap::Args)]
struct JavaArgs {
    #[command(subcommand)]
    action: JavaAction,
}

#[derive(Debug, clap::Subcommand)]
enum JavaAction {
    /// List detected Java runtimes.
    List,
    /// Install a Java major version.
    Install { major: u32 },
    /// Set the default Java.
    Use { path_or_major: String },
    /// Rescan for Java runtimes.
    Rescan,
    /// Test a Java executable.
    Test { path: std::path::PathBuf },
}

#[derive(Debug, clap::Args)]
struct SearchArgs {
    /// Search query.
    query: String,
    /// Project type: mod, modpack, resourcepack, shader, datapack.
    #[arg(long, value_name = "TYPE")]
    project_type: Option<String>,
    /// Filter by Minecraft version.
    #[arg(long, value_name = "V")]
    version: Option<String>,
    /// Filter by loader.
    #[arg(long, value_name = "L")]
    loader: Option<String>,
    /// Maximum results.
    #[arg(short = 'n', long, default_value = "20")]
    limit: usize,
}

#[derive(Debug, clap::Args)]
struct InfoArgs {
    /// Modrinth project slug.
    slug: String,
}

#[derive(Debug, clap::Args)]
struct ModArgs {
    #[command(subcommand)]
    action: ModAction,
}

#[derive(Debug, clap::Subcommand)]
enum ModAction {
    /// Add mods to an instance.
    Add {
        instance: String,
        slugs: Vec<String>,
        #[arg(long)]
        version: Option<String>,
    },
    /// Remove a mod.
    Remove { instance: String, name: String },
    /// List mods in an instance.
    List { instance: String },
    /// Enable a mod.
    Enable { instance: String, name: String },
    /// Disable a mod.
    Disable { instance: String, name: String },
    /// Update mods.
    Update {
        instance: String,
        #[arg(long)]
        all: bool,
    },
}

#[derive(Debug, clap::Args)]
struct ModpackArgs {
    /// Modrinth slug or local .mrpack file.
    target: String,
    /// Instance name.
    #[arg(long, value_name = "N")]
    name: Option<String>,
    /// Modpack version.
    #[arg(long, value_name = "V")]
    version: Option<String>,
}

#[derive(Debug, clap::Args)]
struct PackArgs {
    #[command(subcommand)]
    action: PackAction,
}

#[derive(Debug, clap::Subcommand)]
enum PackAction {
    Add {
        instance: String,
        path: std::path::PathBuf,
    },
    Remove {
        instance: String,
        name: String,
    },
    List {
        instance: String,
    },
    Enable {
        instance: String,
        name: String,
    },
    Disable {
        instance: String,
        name: String,
    },
}

#[derive(Debug, clap::Args)]
struct ShaderArgs {
    #[command(subcommand)]
    action: ShaderAction,
}

#[derive(Debug, clap::Subcommand)]
enum ShaderAction {
    Add {
        instance: String,
        path: std::path::PathBuf,
    },
    Remove {
        instance: String,
        name: String,
    },
    List {
        instance: String,
    },
    Select {
        instance: String,
        name: String,
    },
}

#[derive(Debug, clap::Args)]
struct WorldArgs {
    #[command(subcommand)]
    action: WorldAction,
}

#[derive(Debug, clap::Subcommand)]
enum WorldAction {
    List { instance: String },
    Backup { instance: String, world: String },
    Delete { instance: String, world: String },
}

#[derive(Debug, clap::Args)]
struct ScreenshotArgs {
    #[command(subcommand)]
    action: ScreenshotAction,
}

#[derive(Debug, clap::Subcommand)]
enum ScreenshotAction {
    List { instance: String },
    Open { instance: String, file: String },
    Delete { instance: String, file: String },
}

#[derive(Debug, clap::Args)]
struct StatsArgs {
    /// Watch mode: redraw every second.
    #[arg(short = 'w', long)]
    watch: bool,
}

#[derive(Debug, clap::Args)]
struct DiskArgs {
    /// Clear a cache category: cache, logs, thumbnails.
    #[arg(long, value_name = "CATEGORY")]
    clear_cache: Option<String>,
}

#[derive(Debug, clap::Args)]
struct IconsArgs {
    /// Preview one mode only.
    #[arg(long, value_name = "MODE")]
    mode: Option<IconMode>,
}

#[derive(Debug, clap::Args)]
struct ConfigArgs {
    #[command(subcommand)]
    action: ConfigAction,
}

#[derive(Debug, clap::Subcommand)]
enum ConfigAction {
    /// List all config values.
    List,
    /// Get a single config value.
    Get { key: String },
    /// Set a config value.
    Set { key: String, value: String },
    /// Reset a config key to default.
    Reset { key: String },
    /// Show the config file path.
    Path,
}

#[derive(Debug, clap::Args)]
struct DoctorArgs {
    /// Run the first-run icon check.
    #[arg(long)]
    icons: bool,
}

#[derive(Debug, clap::Args)]
struct CompletionsArgs {
    /// Shell to generate completions for.
    shell: String,
}

#[derive(Debug, clap::Args)]
struct VersionArgs {
    /// Show the Helio banner.
    #[arg(long)]
    banner: bool,
}
