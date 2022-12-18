use std::io::stdout;

use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, Clear, ClearType, LeaveAlternateScreen};
use signal_hook_tokio::Signals;
use tokio::sync::mpsc::Sender;

use crate::bell::BellMessage;
use crate::{StreamExt, SIGHUP, SIGINT, SIGQUIT, SIGTERM};

pub async fn handle_signals(mut signals: Signals, tx_bell: Sender<BellMessage>) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGHUP => {
                tx_bell.send(BellMessage::Stop).await.unwrap();
                disable_raw_mode().unwrap();
                println!("SIGHUP");
            }
            SIGTERM | SIGINT | SIGQUIT => {
                tx_bell.send(BellMessage::Stop).await.unwrap();
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
