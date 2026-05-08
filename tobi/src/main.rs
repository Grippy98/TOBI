mod app;
mod board;
mod custom_image;
mod device;
mod installer;
mod manifest;
mod memory;
mod qr;
mod ui;

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use app::{App, Screen};
use clap::{Parser, ValueEnum};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    Clear as TerminalClear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
    disable_raw_mode, enable_raw_mode,
};
use device::DeviceMode;
use installer::RunMode;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

const DEFAULT_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/Grippy98/TOBI/master/tobi/sample/catalog.json";

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(long, default_value = DEFAULT_MANIFEST_URL)]
    manifest: String,

    #[arg(long, value_enum, default_value_t = CliRunMode::Mock)]
    mode: CliRunMode,

    #[arg(long)]
    target: Option<PathBuf>,

    #[arg(long)]
    allow_write: bool,

    #[arg(long)]
    proxy: Option<String>,
}

#[derive(Clone, Debug, ValueEnum)]
enum CliRunMode {
    Mock,
    Live,
}

impl From<CliRunMode> for RunMode {
    fn from(value: CliRunMode) -> Self {
        match value {
            CliRunMode::Mock => Self::Mock,
            CliRunMode::Live => Self::Live,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let run_mode = RunMode::from(args.mode);
    let (catalog, warning) = match manifest::load_catalog_with_proxy(
        &args.manifest,
        args.proxy.as_deref(),
    ) {
        Ok(catalog) => (catalog, None),
        Err(error) if manifest::is_remote_source(&args.manifest) => (
            manifest::fallback_catalog(),
            Some(format!(
                "Could not fetch the online OS catalog. You can still flash a custom local image from attached media. Press P to configure a proxy and retry, or Enter to continue.\n\n{error:#}"
            )),
        ),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to load manifest {}", args.manifest));
        }
    };
    let board = board::detect_board(run_mode, &catalog);
    let devices = device::list_devices(DeviceMode::from(run_mode), args.target.as_deref())
        .context("failed to enumerate install targets")?;

    let mut terminal = setup_terminal()?;
    let app_result = run_app(
        &mut terminal,
        App::new(
            catalog,
            board,
            devices,
            run_mode,
            args.allow_write,
            args.manifest,
            args.proxy,
            warning,
        ),
    );
    restore_terminal(&mut terminal)?;
    app_result
}

fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode().context("failed to enable terminal raw mode")?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, TerminalClear(ClearType::All))
        .context("failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("failed to create terminal")?;
    terminal.clear().context("failed to clear terminal")?;
    Ok(terminal)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> anyhow::Result<()> {
    disable_raw_mode().context("failed to disable terminal raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("failed to leave alternate screen")?;
    terminal.show_cursor().context("failed to show cursor")
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
) -> anyhow::Result<()> {
    loop {
        app.poll_install_events();
        app.tick_runner();
        app.refresh_system_status_if_due();
        app.auto_reboot_if_due();
        terminal.draw(|frame| {
            let area = frame.area();
            app.set_terminal_size(area.width, area.height);
            ui::render(frame, &app)
        })?;

        if event::poll(Duration::from_millis(80))? {
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc if app.can_quit() => {
                    return Ok(());
                }
                KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    return Ok(());
                }
                _ if app.has_warning() => match key.code {
                    KeyCode::Enter | KeyCode::Esc => app.dismiss_warning(),
                    KeyCode::Char('p') | KeyCode::Char('P') => app.start_proxy_config(),
                    _ => {}
                },
                _ if app.screen() == Screen::ProxyConfig => match key.code {
                    KeyCode::Enter => app.apply_proxy_config(),
                    KeyCode::Esc => app.cancel_proxy_config(),
                    KeyCode::Backspace => app.proxy_backspace(),
                    KeyCode::Char(ch) => app.proxy_push(ch),
                    _ => {}
                },
                _ if app.screen() == Screen::Installing => match key.code {
                    KeyCode::Char(' ') | KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                        app.runner_jump_or_restart();
                    }
                    _ => {}
                },
                _ if matches!(app.screen(), Screen::Complete | Screen::Error) => match key.code {
                    KeyCode::Enter => app.activate_selected(),
                    KeyCode::Char('r') | KeyCode::Char('R') => app.start_over(),
                    _ => {}
                },
                KeyCode::Right if app.screen() == Screen::TargetSelect => app.expand_target(),
                KeyCode::Left if app.screen() == Screen::TargetSelect => app.collapse_target(),
                KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => app.previous(),
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => app.next(),
                KeyCode::Enter => app.activate_selected(),
                KeyCode::Backspace => app.back(),
                KeyCode::Char('r') | KeyCode::Char('R') if app.screen() != Screen::Installing => {
                    app.refresh_devices()
                }
                _ => {}
            }
        }
    }
}
