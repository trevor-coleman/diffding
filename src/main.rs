use std::io::stdout;
use std::path::PathBuf;

use crossterm::terminal::ClearType;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, LeaveAlternateScreen},
    Result,
};
use futures::{future::FutureExt, StreamExt};
use serde_derive::Deserialize;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

use crate::git::{git_loop, GitState};

mod bell;
mod events;
mod git;
mod options;
mod summary;
mod threshold_gauge;
mod ui;

#[derive(Debug, Deserialize, Clone)]
pub struct Options {
    sound_path: Option<PathBuf>,
    threshold: i32,
    loop_time: u64,
    #[allow(dead_code)]
    volume: f32,
    snooze_length: i64,
}

#[derive(Debug)]
pub enum AppMessage {
    Quit,
    Snooze,
}

#[derive(Debug)]
pub enum UiMessage {
    GitUpdate { git_state: GitState },
}

#[tokio::main]
async fn main() -> Result<()> {
    let signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
    let signals_handle = signals.handle();

    let signals_task = tokio::spawn(handle_signals(signals));

    let options = options::get_options().unwrap();

    let mut stdout = stdout();

    enable_raw_mode()?;

    let (tx_app, mut rx_app) = tokio::sync::mpsc::channel::<AppMessage>(32);
    // let (tx_ui, rx_ui) = tokio::sync::mpsc::channel::<UiMessage>(32);

    let (tx_git, rx_git) = tokio::sync::watch::channel(GitState::default());

    // let opt_manager = options.clone();
    let manager = tokio::spawn(async move {
        while let Some(cmd) = rx_app.recv().await {
            match cmd {
                AppMessage::Quit => {
                    disable_raw_mode().unwrap();
                    let mut stdout = std::io::stdout();
                    execute!(stdout, Clear(ClearType::All)).unwrap();
                    execute!(stdout, LeaveAlternateScreen).unwrap();
                    std::process::exit(0);
                }
                AppMessage::Snooze => {
                    execute!(stdout, Clear(ClearType::All)).unwrap();
                }
            }
        }
    });

    let tx_kb = tx_app.clone();
    let opt_kb = options.clone();
    let kb_handle = tokio::spawn(events::keyboard_events(tx_kb, opt_kb));

    let opt_git = options.clone();
    let git_handle = tokio::spawn(git_loop(tx_git, opt_git));

    let rx_git_ui = rx_git.clone();
    let opt_ui = options.clone();
    let ui_handle = tokio::spawn(ui::ui_loop(rx_git_ui, opt_ui));

    kb_handle.await.unwrap();
    git_handle.await.unwrap();
    ui_handle.await.unwrap();
    manager.await.unwrap();
    signals_handle.close();
    signals_task.await?;

    Ok(())
}

async fn handle_signals(mut signals: Signals) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGHUP => {
                disable_raw_mode().unwrap();
                println!("SIGHUP");
            }
            SIGTERM | SIGINT | SIGQUIT => {
                let mut stdout = stdout();
                execute!(stdout, Clear(ClearType::All)).unwrap();
                execute!(stdout, LeaveAlternateScreen).unwrap();
                disable_raw_mode().unwrap();
                let signal_name = match signal {
                    SIGTERM => "SIGTERM",
                    SIGINT => "SIGINT",
                    SIGQUIT => "SIGQUIT",
                    _ => unreachable!(),
                };
                println!("Received {signal_name:?} -- Exiting");
                std::process::exit(2);
            }
            _ => unreachable!(),
        }
    }
}
