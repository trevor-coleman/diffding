use std::io::Stdout;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::{Clear, ClearType, LeaveAlternateScreen};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::bell::BellMessage;
use crate::{GitState, UiMessage};

#[derive(Debug)]
pub enum ManagerMessage {
    Quit,
    Snooze,
    Git { git_state: GitState },
    Bell,
}

pub struct AppState {
    pub ringing: bool,
    pub snoozed: bool,
}

impl AppState {
    fn start_ringing(&mut self) {
        self.ringing = true;
    }

    fn stop_ringing(&mut self) {
        self.ringing = false;
    }

    fn is_ringing(&self) -> bool {
        self.ringing
    }
}

pub async fn manager_loop(
    mut stdout: Stdout,
    mut rx_app: Receiver<ManagerMessage>,
    tx_ui_manager: Sender<UiMessage>,
    tx_bell_manager: Sender<BellMessage>,
) {
    let last_git_state: Arc<Option<GitState>> = Arc::new(None);
    let app_state = Arc::new(Mutex::new(AppState {
        ringing: false,
        snoozed: false,
    }));
    let manager_handle = tokio::spawn(async move {
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
                            .send(UiMessage::GitUpdate {
                                git_state: git_state.clone(),
                            })
                            .await
                            .unwrap();
                        //TODO: Check if bell is already ringing

                        if git_state.is_above_threshold()
                            && !app_state.as_ref().lock().unwrap().is_ringing()
                        {
                            tx_bell_manager.send(BellMessage::Start).await.unwrap();
                            app_state.as_ref().lock().unwrap().start_ringing();
                        } else if !git_state.is_above_threshold()
                            && app_state.as_ref().lock().unwrap().is_ringing()
                        {
                            tx_bell_manager.send(BellMessage::Stop).await.unwrap();
                            app_state.as_ref().lock().unwrap().stop_ringing();
                        }
                    }
                }
                ManagerMessage::Bell => {
                    tx_bell_manager.send(BellMessage::Start).await.unwrap();
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    tx_bell_manager.send(BellMessage::Stop).await.unwrap();
                }
            }
        }
    });
}
