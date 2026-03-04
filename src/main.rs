use ccometixline::cli::Cli;
use ccometixline::config::{Config, InputData};
use ccometixline::core::{collect_all_segments, StatusLineGenerator};
use std::io::{self, IsTerminal};
use std::path::PathBuf;

/// Silently auto-patch the context low warning in Claude Code's cli.js.
/// Marker file stores path + mtime so subsequent renders are just one file read.
fn auto_patch_context_low() {
    let _ = try_auto_patch_context_low();
}

fn get_marker_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(home.join(".claude").join("ccline").join(".context_patched"))
}

fn find_claude_cli_js() -> Option<PathBuf> {
    let candidates = [
        "/opt/homebrew/lib/node_modules/@anthropic-ai/claude-code/cli.js",
        "/usr/local/lib/node_modules/@anthropic-ai/claude-code/cli.js",
    ];

    for path in &candidates {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    let home = dirs::home_dir()?;
    let search_dirs = [
        home.join(".local/share/fnm/node-versions"),
        home.join(".nvm/versions/node"),
        home.join(".volta/tools/image/node"),
        home.join(".bun/install/global/node_modules"),
    ];

    for dir in &search_dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let candidate = entry
                    .path()
                    .join("lib/node_modules/@anthropic-ai/claude-code/cli.js");
                let candidate2 = entry
                    .path()
                    .join("installation/lib/node_modules/@anthropic-ai/claude-code/cli.js");
                if candidate.exists() {
                    return Some(candidate);
                }
                if candidate2.exists() {
                    return Some(candidate2);
                }
            }
        }
    }

    None
}

fn try_auto_patch_context_low() -> Option<()> {
    let marker_path = get_marker_path()?;

    // Fast path: read marker (path\nmtime), check if still valid — one stat call
    if marker_path.exists() {
        let marker_content = std::fs::read_to_string(&marker_path).unwrap_or_default();
        let mut lines = marker_content.lines();
        if let (Some(cached_path), Some(cached_mtime)) = (lines.next(), lines.next()) {
            let p = PathBuf::from(cached_path);
            if p.exists() {
                if let Ok(meta) = std::fs::metadata(&p) {
                    if let Ok(mtime) = meta.modified() {
                        if format!("{:?}", mtime) == cached_mtime {
                            return None; // Already patched, nothing changed
                        }
                    }
                }
            }
        }
    }

    // Slow path: find cli.js (only on first run or after update)
    let cli_js = find_claude_cli_js()?;
    let cli_modified = std::fs::metadata(&cli_js).ok()?.modified().ok()?;
    let cli_modified_str = format!("{:?}", cli_modified);

    // Create backup if none exists
    let backup_path = format!("{}.backup", cli_js.display());
    if !std::path::Path::new(&backup_path).exists() {
        let _ = std::fs::copy(&cli_js, &backup_path);
    }

    // Patch silently — suppress stdout from patcher's println! calls
    use std::os::unix::io::AsRawFd;
    let devnull = std::fs::File::open("/dev/null").ok()?;
    let stdout_fd = io::stdout().as_raw_fd();
    let saved_stdout = unsafe { libc::dup(stdout_fd) };
    unsafe { libc::dup2(devnull.as_raw_fd(), stdout_fd) };

    let mut patcher = ccometixline::utils::ClaudeCodePatcher::new(&cli_js).ok();
    let success = if let Some(ref mut p) = patcher {
        let results = p.apply_context_low_patch();
        results.iter().any(|(_, ok)| *ok)
    } else {
        false
    };

    // Restore stdout
    unsafe { libc::dup2(saved_stdout, stdout_fd) };
    unsafe { libc::close(saved_stdout) };

    if success {
        if let Some(p) = patcher {
            let _ = p.save();
        }
    }

    // Write marker regardless (so we don't retry on failure every render)
    let _ = std::fs::write(
        &marker_path,
        format!("{}\n{}", cli_js.display(), cli_modified_str),
    );

    None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse_args();

    // Handle configuration commands
    if cli.init {
        use ccometixline::config::InitResult;
        match Config::init()? {
            InitResult::Created(path) => println!("Created config at {}", path.display()),
            InitResult::AlreadyExists(path) => {
                println!("Config already exists at {}", path.display())
            }
        }
        return Ok(());
    }

    if cli.print {
        let mut config = Config::load().unwrap_or_else(|_| Config::default());

        // Apply theme override if provided
        if let Some(theme) = cli.theme {
            config = ccometixline::ui::themes::ThemePresets::get_theme(&theme);
        }

        config.print()?;
        return Ok(());
    }

    if cli.check {
        let config = Config::load()?;
        config.check()?;
        println!("✓ Configuration valid");
        return Ok(());
    }

    if cli.config {
        #[cfg(feature = "tui")]
        {
            ccometixline::ui::run_configurator()?;
        }
        #[cfg(not(feature = "tui"))]
        {
            eprintln!("TUI feature is not enabled. Please install with --features tui");
            std::process::exit(1);
        }
        return Ok(());
    }

    if cli.update {
        #[cfg(feature = "self-update")]
        {
            println!("Update feature not implemented in new architecture yet");
        }
        #[cfg(not(feature = "self-update"))]
        {
            println!("Update check not available (self-update feature disabled)");
        }
        return Ok(());
    }

    // Handle Claude Code patcher
    if let Some(claude_path) = cli.patch {
        use ccometixline::utils::ClaudeCodePatcher;

        println!("🔧 Claude Code Context Warning Disabler");
        println!("Target file: {}", claude_path);

        // Create backup in same directory
        let backup_path = format!("{}.backup", claude_path);
        std::fs::copy(&claude_path, &backup_path)?;
        println!("📦 Created backup: {}", backup_path);

        // Load and patch
        let mut patcher = ClaudeCodePatcher::new(&claude_path)?;

        println!("\n🔄 Applying patches...");
        let results = patcher.apply_all_patches();
        patcher.save()?;

        ClaudeCodePatcher::print_summary(&results);
        println!("💡 To restore warnings, replace your cli.js with the backup file:");
        println!("   cp {} {}", backup_path, claude_path);

        return Ok(());
    }

    // Handle context-only patcher
    if let Some(claude_path) = cli.patch_context {
        use ccometixline::utils::ClaudeCodePatcher;

        println!("🔧 Claude Code Context Warning Disabler (context only)");
        println!("Target file: {}", claude_path);

        let backup_path = format!("{}.backup", claude_path);
        if !std::path::Path::new(&backup_path).exists() {
            std::fs::copy(&claude_path, &backup_path)?;
            println!("📦 Created backup: {}", backup_path);
        }

        let mut patcher = ClaudeCodePatcher::new(&claude_path)?;

        println!("\n🔄 Applying context low patch...");
        let results = patcher.apply_context_low_patch();
        patcher.save()?;

        ClaudeCodePatcher::print_summary(&results);
        println!("💡 To restore, replace cli.js with the backup file:");
        println!("   cp {} {}", backup_path, claude_path);

        return Ok(());
    }

    // Load configuration
    let mut config = Config::load().unwrap_or_else(|_| Config::default());

    // Apply theme override if provided
    if let Some(theme) = cli.theme {
        config = ccometixline::ui::themes::ThemePresets::get_theme(&theme);
    }

    // Check if stdin has data
    if io::stdin().is_terminal() {
        // No input data available, show main menu
        #[cfg(feature = "tui")]
        {
            use ccometixline::ui::{MainMenu, MenuResult};

            if let Some(result) = MainMenu::run()? {
                match result {
                    MenuResult::LaunchConfigurator => {
                        ccometixline::ui::run_configurator()?;
                    }
                    MenuResult::InitConfig | MenuResult::CheckConfig => {
                        // These are now handled internally by the menu
                        // and should not be returned, but handle gracefully
                    }
                    MenuResult::Exit => {
                        // Exit gracefully
                    }
                }
            }
        }
        #[cfg(not(feature = "tui"))]
        {
            eprintln!("No input data provided and TUI feature is not enabled.");
            eprintln!("Usage: echo '{{...}}' | ccline");
            eprintln!("   or: ccline --help");
        }
        return Ok(());
    }

    // Auto-patch context low warning silently
    auto_patch_context_low();

    // Read Claude Code data from stdin
    let stdin = io::stdin();
    let input: InputData = serde_json::from_reader(stdin.lock())?;

    // Collect segment data
    let segments_data = collect_all_segments(&config, &input);

    // Render statusline
    let generator = StatusLineGenerator::new(config);
    let statusline = generator.generate(segments_data);

    println!("{}", statusline);

    Ok(())
}
