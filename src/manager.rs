use std::io::Stdout;
use std::sync::Arc;

use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::{Clear, ClearType, LeaveAlternateScreen};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

use crate::{GitState, UiMessage};

#[derive(Debug)]
pub enum ManagerMessage {
    Quit,
    Snooze,
    Git { git_state: GitState },
}

pub fn manager_loop(
    mut stdout: Stdout,
    mut rx_app: Receiver<ManagerMessage>,
    tx_ui_manager: Sender<UiMessage>,
) -> JoinHandle<()> {
    let last_git_state: Arc<Option<GitState>> = Arc::new(None);

    tokio::spawn(async move {
        while let Some(cmd) = rx_app.recv().await {
            match cmd {
                ManagerMessage::Quit => {
                    disable_raw_mode().unwrap();
                    let mut stdout = std::io::stdout();
                    execute!(stdout, Clear(ClearType::All)).unwrap();
                    execute!(stdout, LeaveAlternateScreen).unwrap();
                    std::process::exit(0);
                }
                ManagerMessage::Snooze => {
                    execute!(stdout, Clear(ClearType::All)).unwrap();
                }
                ManagerMessage::Git { git_state } => {
                    if !git_state.compare_with_prev(last_git_state.clone()) {
                        tx_ui_manager
                            .send(UiMessage::GitUpdate { git_state })
                            .await
                            .unwrap();
                    }
                }
            }
        }
    })
}
