use std::path::PathBuf;

use soloud::{AudioExt, LoadExt, Soloud, Wav};

pub fn ring(sound_path: Option<PathBuf>) {
    let sl = Soloud::default().unwrap();
    let mut wav = Wav::default();
    println!("getting sound path");
    match sound_path {
        None => {
            println!("No sound path found");
            load_default_sound(&mut wav);
        }
        Some(path) => {
            if path.exists() {
                println!("path exists");
                let result = wav.load(path);
                if let Err(_) = result {
                    load_default_sound(&mut wav);
                }
            } else {
                println!("path does not exist");
                load_default_sound(&mut wav);
            }
        }
    }
    sl.play(&wav);

    while sl.voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn load_default_sound(wav: &mut Wav) {
    wav.load_mem(include_bytes!(
        "./assets/387533__soundwarf__alert-short.wav"
    ))
    .unwrap();
}
