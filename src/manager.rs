use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Local};
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::{Clear, ClearType, LeaveAlternateScreen};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::bell::BellMessage;
use crate::{GitState, Options, UiMessage};

#[derive(Debug)]
pub enum ManagerMessage {
    Quit,
    Snooze,
    Git { git_state: GitState },
    Bell,
    Redraw,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppState {
    pub ringing: bool,
    pub snoozed: bool,
    pub snoozed_at: Option<DateTime<Local>>,
    pub snoozed_until: Option<DateTime<Local>>,
}

impl AppState {
    pub fn new(
        ringing: bool,
        snoozed: bool,
        snoozed_at: Option<DateTime<Local>>,
        snoozed_until: Option<DateTime<Local>>,
    ) -> Self {
        Self {
            ringing,
            snoozed,
            snoozed_at,
            snoozed_until,
        }
    }

    pub fn default() -> Self {
        Self {
            ringing: false,
            snoozed: false,
            snoozed_at: None,
            snoozed_until: None,
        }
    }

    fn start_ringing(&mut self) {
        self.ringing = true;
    }

    fn stop_ringing(&mut self) {
        self.ringing = false;
    }

    fn is_ringing(&self) -> bool {
        self.ringing
    }
    fn snooze(&mut self) {
        self.snoozed = true;
        self.ringing = false;
        self.snoozed_at = Some(Local::now());
    }
    fn unsnooze(&mut self) {
        self.snoozed = false;
        self.snoozed_at = None;
    }
}

pub async fn manager_loop(
    mut rx_app: Receiver<ManagerMessage>,
    tx_ui_manager: Sender<UiMessage>,
    tx_bell_manager: Sender<BellMessage>,
    options: Arc<Options>,
) {
    let mut last_git_state: Arc<Option<GitState>> = Arc::new(None);
    let app_state = Arc::new(Mutex::new(AppState::default()));
    let manager_handle = tokio::spawn(async move {
        while let Some(cmd) = rx_app.recv().await {
            match cmd {
                ManagerMessage::Redraw => {
                    let app_state_clone = app_state.clone();
                    if let Some(git_state) = Arc::clone(&last_git_state).as_ref() {
                        tx_ui_manager
                            .send(UiMessage::GitUpdate {
                                git_state: git_state.clone(),
                                app_state: app_state_clone,
                            })
                            .await
                            .unwrap();
                    }
                }
                ManagerMessage::Quit => {
                    disable_raw_mode().unwrap();
                    let mut stdout = std::io::stdout();
                    execute!(stdout, Clear(ClearType::All)).unwrap();
                    execute!(stdout, LeaveAlternateScreen).unwrap();
                    std::process::exit(0);
                }
                // ManagerMessage::Snooze => {
                //     // execute!(stdout, Clear(ClearType::All)).unwrap();
                // }
                ManagerMessage::Git { git_state } => {
                    if !git_state.compare_with_prev(last_git_state.clone()) {
                        tx_ui_manager
                            .send(UiMessage::GitUpdate {
                                git_state: git_state.clone(),
                                app_state: app_state.clone(),
                            })
                            .await
                            .unwrap();
                        //TODO: Check if bell is already ringing

                        interpret_state_and_send_messages(&tx_bell_manager, &app_state, &git_state)
                            .await;
                        last_git_state = Arc::new(Some(git_state.clone()));
                    }
                }
                ManagerMessage::Bell => {
                    tx_bell_manager.send(BellMessage::Start).await.unwrap();
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    tx_bell_manager.send(BellMessage::Stop).await.unwrap();
                }
                ManagerMessage::Snooze => {
                    let snooze_length = options.as_ref().snooze_length as u64;
                    let snooze_time = Duration::from_secs(snooze_length);
                    {
                        app_state.as_ref().lock().unwrap().snooze();
                    }
                    tx_bell_manager.send(BellMessage::Stop).await.unwrap();
                    if let Some(git_state) = Arc::clone(&last_git_state).as_ref() {
                        tx_ui_manager
                            .send(UiMessage::GitUpdate {
                                git_state: git_state.clone(),
                                app_state: app_state.clone(),
                            })
                            .await
                            .unwrap();
                    }
                    let app_state_clone = app_state.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(snooze_time).await;
                        {
                            app_state_clone.as_ref().lock().unwrap().unsnooze();
                        }
                    });

                    if let Some(git_state) = Arc::clone(&last_git_state).as_ref() {
                        interpret_state_and_send_messages(&tx_bell_manager, &app_state, &git_state)
                            .await;
                    }
                }
            }
        }
    });
}

async fn interpret_state_and_send_messages(
    tx_bell_manager: &Sender<BellMessage>,
    app_state: &Arc<Mutex<AppState>>,
    git_state: &GitState,
) {
    let is_ringing = app_state.as_ref().lock().unwrap().is_ringing();
    let is_snoozed = app_state.as_ref().lock().unwrap().snoozed;

    if git_state.is_above_threshold() && !is_ringing && !is_snoozed {
        tx_bell_manager.send(BellMessage::Start).await.unwrap();
        app_state.as_ref().lock().unwrap().start_ringing();
    } else if (!git_state.is_above_threshold() && is_ringing) || is_snoozed {
        tx_bell_manager.send(BellMessage::Stop).await.unwrap();
        app_state.as_ref().lock().unwrap().stop_ringing();
    }
}
