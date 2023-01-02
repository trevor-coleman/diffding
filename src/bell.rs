use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use soloud::{AudioExt, LoadExt, Soloud, Wav};
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use crate::Options;

#[derive(Debug)]
pub enum BellMessage {
    Start,
    Stop,
}

pub async fn bell_loop(mut rx: tokio::sync::mpsc::Receiver<BellMessage>, options: Arc<Options>) {
    let mut cancel_token = CancellationToken::new();

    let handle = tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            let options = options.clone();
            match cmd {
                BellMessage::Start => {
                    cancel_token = CancellationToken::new();
                    let child_token_2 = cancel_token.child_token();
                    tokio::spawn(ring_bell(options, child_token_2));
                }
                BellMessage::Stop => {
                    cancel_token.cancel();
                }
            }
        }
    });

    handle.await.unwrap();
}

async fn ring_bell(options: Arc<Options>, cancel_token: CancellationToken) {
    let mut interval = interval(Duration::from_millis(10000));
    interval.tick().await;
    while !cancel_token.is_cancelled() {
        ring(&options.sound_path);
        interval.tick().await;
    }
}

pub fn ring(sound_path: &Option<PathBuf>) {
    let sl = Soloud::default().unwrap();
    let mut wav = Wav::default();
    match sound_path {
        None => {
            load_default_sound(&mut wav);
        }
        Some(path) => {
            if path.exists() {
                let result = wav.load(path);
                if let Err(_) = result {
                    load_default_sound(&mut wav);
                }
            } else {
                load_default_sound(&mut wav);
            }
        }
    }
    sl.play(&wav);

    while sl.voice_count() > 0 {
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn load_default_sound(wav: &mut Wav) {
    wav.load_mem(include_bytes!(
        "./assets/387533__soundwarf__alert-short.wav"
    ))
    .unwrap();
}
