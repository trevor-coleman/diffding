use std::io::stdout;

use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, Clear, ClearType, LeaveAlternateScreen};
use signal_hook_tokio::Signals;

use crate::{StreamExt, SIGHUP, SIGINT, SIGQUIT, SIGTERM};

pub async fn handle_signals(mut signals: Signals) {
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
