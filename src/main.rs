//! Demonstrates how to read events asynchronously with tokio.
//!
//! cargo run --features="event-stream" --example event-stream-tokio

use std::io::stdout;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::terminal::ClearType;
use crossterm::{
    cursor::position,
    event::{Event, EventStream, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen, LeaveAlternateScreen,
    },
    Result,
};
use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;
use serde_derive::Deserialize;
use tokio::sync::mpsc::Sender;

use crate::git::{git_loop, GitState};

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

async fn print_events(tx: Sender<AppMessage>) {
    let mut reader = EventStream::new();

    loop {
        let _delay = Delay::new(Duration::from_millis(100)).fuse();
        let mut event = reader.next().fuse();

        select! {
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        if let Event::Key(key_event) = event {
                            match key_event.code {
                                KeyCode::Char('q') => {
                                    tx.send(AppMessage::Quit).await.unwrap();
                                },
                                KeyCode::Char('c') => {
                                    if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                                        tx.send(AppMessage::Quit).await.unwrap();
                                    } else {
                                        let (x, y) = position().unwrap();
                                        println!("Cursor position: ({}, {})\r", x, y);
                                    }
                                    tx.send(AppMessage::Quit).await.unwrap();
                                }
                                _ => {}
                            }
                        }

                    }
                    Some(Err(e)) => println!("Error: {:?}\r", e),
                    None => break,
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let (tx_app, mut rx_app) = tokio::sync::mpsc::channel::<AppMessage>(32);
    let (tx_ui, rx_ui) = tokio::sync::mpsc::channel::<UiMessage>(32);

    let tx_manager = tx_ui.clone();
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
                AppMessage::GitUpdate { git_state } => {
                    tx_manager
                        .send(UiMessage::GitUpdate {
                            // git_state: GitState {
                            //     git_changes: GitChanges {
                            //         insertions: git_state.git_changes.insertions,
                            //         deletions: git_state.git_changes.deletions,
                            //         total: 500,
                            //     },
                            //     current_commit: git_state.current_commit,
                            //     last_commit: git_state.last_commit,
                            // },
                            git_state,
                        })
                        .await
                        .unwrap();
                }
            }
        }
    });

    let tx_kb = tx_app.clone();
    let events_handle = tokio::spawn(print_events(tx_kb));

    let tx_git = tx_app.clone();
    let git_handle = tokio::spawn(git_loop(tx_git));

    let ui_handle = tokio::spawn(ui::ui_loop(rx_ui));

    events_handle.await.unwrap();
    git_handle.await.unwrap();
    ui_handle.await.unwrap();
    manager.await.unwrap();

    disable_raw_mode()
}

#[derive(Debug)]
pub enum AppMessage {
    Quit,
    GitUpdate { git_state: GitState },
}

#[derive(Debug)]
pub enum UiMessage {
    GitUpdate { git_state: GitState },
}
