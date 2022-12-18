use std::io::stdout;
use std::path::PathBuf;

use crossterm::{terminal::enable_raw_mode, Result};
use futures::{future::FutureExt, StreamExt};
use serde_derive::Deserialize;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

use crate::bell::BellMessage;
use crate::git::{git_loop, GitState};
use crate::manager::ManagerMessage;
use crate::ui::UiMessage;

mod bell;
mod events;
mod git;
mod manager;
mod options;
mod signals;
mod summary;
mod threshold_gauge;
mod ui;

#[derive(Debug, Deserialize, Clone)]
pub struct Options {
    sound_path: Option<PathBuf>,
    threshold: i32,
    git_update_time: u64,
    #[allow(dead_code)]
    volume: f32,
    snooze_length: i64,
}

// TODO: implement bell_ringer and bell
// TODO: handle window-size-change events in the UI thread

#[tokio::main]
async fn main() -> Result<()> {
    let signals = Signals::new([SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
    let signals_handle = signals.handle();

    let (tx_bell, mut rx_bell) = tokio::sync::mpsc::channel::<BellMessage>(32);

    let tx_bell_signals = tx_bell.clone();
    let signals_task = tokio::spawn(signals::handle_signals(signals, tx_bell_signals));

    let options = options::get_options().unwrap();

    let mut stdout = stdout();

    enable_raw_mode()?;

    let (tx_app, mut rx_app) = tokio::sync::mpsc::channel::<ManagerMessage>(32);
    let (tx_ui, mut rx_ui) = tokio::sync::mpsc::channel::<UiMessage>(32);
    let (tx_bell, mut rx_bell) = tokio::sync::mpsc::channel::<BellMessage>(32);

    let tx_ui_manager = tx_ui.clone();
    let tx_bell_manager = tx_bell.clone();
    let manager = tokio::spawn(manager::manager_loop(
        stdout,
        rx_app,
        tx_ui_manager,
        tx_bell_manager,
    ));

    let tx_app_kb = tx_app.clone();
    let opt_kb = options.clone();
    let kb_handle = tokio::spawn(events::keyboard_events(tx_app_kb, opt_kb));

    let opt_git = options.clone();
    let tx_app_git = tx_app.clone();
    let git_handle = tokio::spawn(git_loop(tx_app_git, opt_git));

    let opt_ui = options.clone();
    let ui_handle = tokio::spawn(ui::ui_loop(rx_ui, opt_ui));

    let opt_bell = options.clone();
    let bell_handle = tokio::spawn(bell::bell_loop(rx_bell, opt_bell));

    bell_handle.await.unwrap();
    kb_handle.await.unwrap();
    git_handle.await.unwrap();
    ui_handle.await.unwrap();
    manager.await.unwrap();
    signals_handle.close();
    signals_task.await?;

    Ok(())
}
