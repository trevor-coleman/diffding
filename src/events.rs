use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode};
use futures_timer::Delay;
use tokio::select;
use tokio::sync::mpsc::Sender;

use crate::manager::ManagerMessage;
use crate::{FutureExt, Options, StreamExt};

pub async fn keyboard_events(tx: Sender<ManagerMessage>, options: Arc<Options>) {
    let mut reader = EventStream::new();

    loop {
        let _delay = Delay::new(Duration::from_millis(100)).fuse();
        let event = reader.next().fuse();

        select! {
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        if let Event::Resize(_width, _height) = event{
                            tx.send(ManagerMessage::Redraw).await.unwrap();
                        }
                        if let Event::Key(key_event) = event {
                            match key_event.code {
                                KeyCode::Char('q') => {
                                    tx.send(ManagerMessage::Quit).await.unwrap();
                                },
                                KeyCode::Char('c') => {
                                    if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                                        tx.send(ManagerMessage::Quit).await.unwrap();
                                    }
                                },
                                KeyCode::Char(' ') => {
                                    tx.send(ManagerMessage::Snooze).await.unwrap();
                                },
                                KeyCode::Char('b') => {
                                    tx.send(ManagerMessage::Bell).await.unwrap();
                                },
                                _ => {}
                            }
                        }

                    }
                    Some(Err(e)) => println!("Error: {e:?}\r"),
                    None => break,
                }
            }
        }
    }
}
