mod entries;
mod ui;

use std::{
    fmt::Debug,
    fs,
    io::{self, Stdout},
    os::unix::process::CommandExt,
    process::{self},
};

use anyhow::{Context, Result};
use clap::{arg, command, Parser};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use entries::{get_entries, BaseEntry, RawEntry};
use indexmap::IndexMap;
use serde::Deserialize;

use tui::{backend::CrosstermBackend, Terminal};
use ui::{input_ui, render_ui, UiStyle};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    show: String,
}

#[derive(Deserialize, Debug)]
struct ConfigFile {
    global: BaseEntry,
    #[serde(flatten, default)]
    entries: IndexMap<String, RawEntry>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let config_path = format!("~/.config/qop/{}.toml", args.show);
    let config_path =
        shellexpand::full(&config_path).context(format!("Error expanding path {config_path}"))?;
    let config_file = toml::from_str::<ConfigFile>(
        &fs::read_to_string(config_path.to_string())
            .context(format!("Config file '{config_path}' not found"))?,
    )
    .context(format!("Failed to deserialize file '{config_path}'"))?;

    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to start terminal backend.")?;

    let mut current = String::new();
    let mut style = UiStyle::default();
    if let Some(color) = config_file.global.background.clone() {
        style.background = color.as_str().into();
    }
    if let Some(color) = config_file.global.foreground.clone() {
        style.foreground = color.as_str().into();
    }
    if let Some(color) = config_file.global.accent.clone() {
        style.accent = color.as_str().into();
    }
    if let Some(color) = config_file.global.faded.clone() {
        style.faded = color.as_str().into();
    }
    if let Some(color) = config_file.global.highlight.clone() {
        style.highlight = color.as_str().into();
    }
    if let Some(color) = config_file.global.highlight_accent.clone() {
        style.highlight_accent = color.as_str().into();
    }
    let (entries, lengths) = get_entries(config_file);

    loop {
        if let Err(e) = render_ui(&current, &entries, &mut terminal, &lengths, &style) {
            stop_term(&mut terminal)?;
            eprintln!("{e}");
            process::exit(1);
        };
        if let Some(found) = entries.iter().find(|entry| entry.key == current) {
            stop_term(&mut terminal)?;
            let path: String = shellexpand::full(&found.path)
                .expect(&format!("Error expanding path {}", found.path))
                .into();
            let mut cmd = &mut process::Command::new(&found.exec);
            cmd = cmd.envs(found.env.clone()).args(&found.args);
            if found.path.len() > 0 {
                cmd = cmd.arg(path);
            }
            cmd.process_group(0).spawn()?;
            process::exit(0);
        }
        if let Err(e) = input_ui(&mut current, &entries) {
            stop_term(&mut terminal)?;
            eprintln!("{e}");
            process::exit(1);
        };
    }
}

fn stop_term(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;
    disable_raw_mode()?;

    Ok(())
}
