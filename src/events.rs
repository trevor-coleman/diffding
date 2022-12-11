use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode};
use futures_timer::Delay;
use tokio::select;
use tokio::sync::mpsc::Sender;

use crate::{AppMessage, FutureExt, Options, StreamExt};

pub async fn keyboard_events(tx: Sender<AppMessage>, options: Arc<Options>) {
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
                                    }
                                }
                                KeyCode::Char(' ') => {
                                    tx.send(AppMessage::Snooze).await.unwrap();
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
