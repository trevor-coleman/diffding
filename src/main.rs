mod graph;
mod splash;

use crate::graph::draw_graph;
use crate::splash::splash_screen;
use chrono::Local;
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
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor};
use tokio::{spawn, time};

#[derive(Debug, Deserialize)]
pub struct Options {
    sound: Option<PathBuf>,
    threshold: i32,
    loop_time: u64,
    #[allow(dead_code)]
    volume: f32,
}

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
        sound: match settings.get("sound") {
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
            sound: config_options.sound,
            threshold: config_options.threshold,
            volume: config_options.volume,
        },
        _ => Options {
            loop_time: args[1].parse::<u64>().unwrap(),
            sound: config_options.sound,
            threshold: args[2].parse::<i32>().unwrap(),
            volume: config_options.volume,
        },
    };

    splash_screen(&options);

    let forever = spawn(async move {
        let mut interval = time::interval(Duration::from_secs(options.loop_time));
        let mut last_count = 0;

        loop {
            interval.tick().await;
            last_count = alert_loop(options.threshold, &last_count, &options.sound)
                .await
                .unwrap();
        }
    });

    let quit = move || {
        println!("\r");
        println!(
            "{lightGreen}Quitting!!{reset}\r",
            lightGreen = color::Fg(color::LightGreen),
            reset = color::Fg(color::Reset)
        );
        std::process::exit(0);
    };

    fn carriage_return() {
        print!("\r");
    }

    // listen for keypress and print to console
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
                            blue = color::Fg(color::Blue),
                            reset = color::Fg(color::Reset)
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
            wav.load_mem(include_bytes!("./387533__soundwarf__alert-short.wav"))
                .unwrap();
        }
        Some(path) => {
            if path.exists() {
                let result = wav.load(path);
                match result {
                    Ok(_) => {}
                    Err(_) => {
                        wav.load_mem(include_bytes!("./387533__soundwarf__alert-short.wav"))
                            .unwrap();
                    }
                }
            } else {
                wav.load_mem(include_bytes!("./387533__soundwarf__alert-short.wav"))
                    .unwrap();
            }
        }
    }
    sl.play(&wav);
    while sl.voice_count() > 0 {
        std::thread::sleep(Duration::from_millis(100));
    }
}

// a change
async fn alert_loop(
    threshold: i32,
    last_count: &i32,
    sound_path: &Option<PathBuf>,
) -> Result<i32, Box<dyn Error>> {
    let changes = count_changes().unwrap();

    let total = changes.total;

    /** TODO: check commit ID instead */
    if total == 0 && last_count > &0 {
        celebrate_commit();
    }

    let date = Local::now();
    print!("{}{}", cursor::Up(7), clear::CurrentLine);
    print_status_display(options);
    if total > threshold {
        on_threshold_exceeded();
    } else {
        on_below_threshold();
    }

    print_key_reminders();
    if { total > threshold } {
        play_sound(sound_path);
    }

    Ok(total)
}

fn print_key_reminders() {
    println!(
        "\n\r{lightWhite}Press {red}Q{lightWhite} to quit{reset}\r",
        red = color::Fg(color::LightCyan),
        reset = color::Fg(color::Reset),
        lightWhite = color::Fg(color::LightWhite)
    );
}

fn celebrate_commit() {
    println!(
        "\n\n\r{}-----{}🎉 COMMITTED 🎉{}-----{}\n\n\r",
        color::Fg(color::White),
        color::Fg(color::Blue),
        color::Fg(color::White),
        color::Fg(color::Reset)
    );
}

fn print_status_display(options: &Options) {
    let date = Local::now();
    print!("{} -- ", date.format("%H:%M:%S"));
    draw_graph(changes, options.threshold);
    println!("\r");
}

fn on_below_threshold() {
    println!(
        "\n\r{white}Watching for changes...{reset}\n\r",
        white = color::Fg(color::White),
        reset = color::Fg(color::Reset)
    );
    println!(
        "{green}👍🏻 Keep up the good work!{reset}\r",
        green = color::Fg(color::Green),
        reset = color::Fg(color::Reset)
    );
}

fn on_threshold_exceeded() {
    println!(
        "\n\r{yellow}!!!{lightRed} TIME TO COMMIT {yellow}!!!{reset}\n\r",
        lightRed = color::Fg(color::LightRed),
        yellow = color::Fg(color::LightYellow),
        reset = color::Fg(color::Reset)
    );
    println!(
        "{white}Press space to snooze for {lightCyan}5 {white}minutes. {reset}\r",
        white = color::Fg(color::White),
        reset = color::Fg(color::Reset)
    );
}

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
