mod graph;
mod messages;
mod splash;

use crate::graph::draw_graph;
use crate::splash::splash_screen;
use chrono::Local;
use color::{Fg, LightYellow, Reset};
use config::{Config, File};
use regex::Regex;
use serde_derive::Deserialize;
use soloud::*;
use std::collections::HashMap;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::{env, str};
use termion::event::Key;
use termion::{clear, color, cursor};
use termion::{input::TermRead, raw::IntoRawMode};

use tokio::{spawn, time};

#[derive(Debug, Deserialize)]
pub struct Options {
    sound_path: Option<PathBuf>,
    threshold: i32,
    loop_time: u64,
    #[allow(dead_code)]
    volume: f32,
}

#[derive(Debug, Clone)]
pub struct LoopState {
    pub changes: Changes,
    pub is_snoozed: bool,
    pub snooze_time: Option<chrono::DateTime<chrono::Local>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut config_path = PathBuf::new();
    config_path.push(env::var("HOME").unwrap());
    config_path.push(".config");
    config_path.push("diffding");

    let settings = Config::builder()
        .add_source(File::from(config_path.join("config.toml")).required(false))
        .build()?;

    let settings = settings
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    println!("CONFIG: {:?}", settings);

    let config_options = Options {
        sound_path: match settings.get("sound") {
            Some(sound) => Some(PathBuf::from(config_path.join(sound))),
            None => None,
        },
        threshold: settings
            .get("threshold")
            .unwrap_or(&"".to_string())
            .parse::<i32>()
            .unwrap_or(100),
        loop_time: settings
            .get("interval")
            .unwrap_or(&"".to_string())
            .parse::<u64>()
            .unwrap_or(10),
        volume: settings
            .get("volume")
            .unwrap_or(&"".to_string())
            .parse::<f32>()
            .unwrap_or(1.0),
    };

    let mut stdout = stdout().into_raw_mode().unwrap();

    let args: Vec<String> = env::args().collect();

    let options: Options = match args.len() {
        1 => config_options,
        2 => Options {
            loop_time: args[1].parse::<u64>().unwrap(),
            sound_path: config_options.sound_path,
            threshold: config_options.threshold,
            volume: config_options.volume,
        },
        _ => Options {
            loop_time: args[1].parse::<u64>().unwrap(),
            sound_path: config_options.sound_path,
            threshold: args[2].parse::<i32>().unwrap(),
            volume: config_options.volume,
        },
    };

    splash_screen(&options);

    let forever = spawn(async move {
        let mut interval = time::interval(Duration::from_secs(options.loop_time));
        let mut loop_state: LoopState = LoopState {
            changes: Changes {
                insertions: 0,
                deletions: 0,
                total: 0,
            },

            is_snoozed: false,
            snooze_time: None,
        };

        loop {
            interval.tick().await;
            loop_state = alert_loop(&options, &mut loop_state).await.unwrap();
        }
    });

    let quit = move || {
        println!("\r");
        println!(
            "{lightYellow}Quitting!!{reset}\r",
            lightYellow = Fg(LightYellow),
            reset = Fg(Reset)
        );
        std::process::exit(0);
    };

    fn carriage_return() {
        print!("\r");
    }

    let listen_for_keypress = spawn(async move {
        let mut interval = time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            let stdin = stdin();
            for c in stdin.keys() {
                match c.unwrap() {
                    Key::Char('q') => {
                        stdout.suspend_raw_mode().unwrap();
                        quit();
                    }
                    Key::Char(' ') => {
                        println!(
                            "{blue}I would snooze, but it's not implemented yet!{reset}\r",
                            blue = Fg(color::Blue),
                            reset = Fg(Reset)
                        );
                    }
                    _ => {}
                }
                stdout.flush().unwrap();
            }
        }
    });

    forever.await.unwrap();
    listen_for_keypress.await.unwrap();

    Ok(())
}

fn play_sound(sound_path: &Option<PathBuf>) {
    let sl = Soloud::default().unwrap();
    let mut wav = Wav::default();
    match sound_path {
        None => {
            load_default_sound(&mut wav);
        }
        Some(path) => {
            if path.exists() {
                let result = wav.load(path);
                match result {
                    Err(_) => {
                        load_default_sound(&mut wav);
                    }
                    _ => {}
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
    wav.load_mem(include_bytes!("assets/387533__soundwarf__alert-short.wav"))
        .unwrap();
}

// a change
async fn alert_loop(
    options: &Options,
    last_state: &mut LoopState,
) -> Result<LoopState, Box<dyn Error>> {
    let state = last_state.clone();

    let total = state.changes.total;
    let last_total = &last_state.changes.total;

    /** TODO: check commit ID instead */
    if total == 0 && *last_total > 0 {
        messages::celebrate_commit();
    }

    print!("{}{}", cursor::Up(7), clear::CurrentLine);
    print_status_display(options, last_state);
    if total > options.threshold {
        on_threshold_exceeded();
    } else {
        on_below_threshold();
    }

    print_key_reminders();
    if { total > options.threshold } {
        play_sound(&options.sound_path);
    }

    Ok(state)
}

fn print_key_reminders() {
    messages::press_q_to_quit();
}

fn print_status_display(options: &Options, loop_state: &LoopState) {
    let date = Local::now();
    print!("{} -- ", date.format("%H:%M:%S"));
    draw_graph(&loop_state.changes, &options.threshold);
    println!("\r");
}

fn on_below_threshold() {
    messages::watching_for_changes();
    messages::keep_up_the_good_work();
}

fn on_threshold_exceeded() {
    messages::time_to_commit();
    messages::press_space_to_snooze();
}

#[derive(Debug, Clone)]
pub struct Changes {
    pub insertions: i32,
    pub deletions: i32,
    pub total: i32,
}

fn count_changes() -> Result<Changes, Box<(dyn Error + 'static)>> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--shortstat")
        .output()?;

    let stdout = str::from_utf8(&output.stdout)?;
    let re = Regex::new(r"((\d+)\D+)((\d+)\D+)?((\d+)?\D+)?")?;
    let captures = re.captures(stdout).ok_or("No match");

    match captures {
        Ok(captures) => {
            let insertions = captures
                .get(4)
                .map_or("0", |m| m.as_str())
                .parse::<i32>()
                .unwrap();
            let deletions = captures
                .get(6)
                .map_or("0", |m| m.as_str())
                .parse::<i32>()
                .unwrap();

            Ok(Changes {
                insertions,
                deletions,
                total: insertions + deletions,
            })
        }

        Err(_) => Ok(Changes {
            insertions: 0,
            deletions: 0,
            total: 0,
        }),
    }
}
