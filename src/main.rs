use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;
use std::process::Command;
use std::str;
use std::time::Duration;

use chrono::{DateTime, Local};
use color::{Fg, LightYellow, Reset};
use regex::Regex;
use serde_derive::Deserialize;
use soloud::*;
use termion::event::Key;
use termion::{clear, color, cursor};
use termion::{input::TermRead, raw::IntoRawMode};
use tokio::{spawn, time};

use crate::splash::splash_screen;

mod graph;
mod messages;
mod options;
mod splash;

#[derive(Debug, Deserialize, Clone)]
pub struct Options {
    sound_path: Option<PathBuf>,
    threshold: i32,
    loop_time: u64,
    #[allow(dead_code)]
    volume: f32,
    snooze_length: i64,
}

#[derive(Debug, Clone)]
pub struct LoopState {
    pub changes: Changes,
    pub snooze_time: Option<DateTime<Local>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout().into_raw_mode().unwrap();

    let options = options::get_options().unwrap();

    splash_screen(&options);

    let main_loop = spawn(async move {
        let mut interval = time::interval(Duration::from_secs(options.loop_time));
        let mut loop_state: LoopState = LoopState {
            changes: Changes {
                insertions: 0,
                deletions: 0,
                total: 0,
            },
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

    main_loop.await.unwrap();
    listen_for_keypress.await.unwrap();

    Ok(())
}

fn play_sound(options: &Options) {
    let sl = Soloud::default().unwrap();
    let mut wav = Wav::default();
    match &options.sound_path {
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
    let last_total = &last_state.changes.total;
    let changes = count_changes().unwrap();

    // TODO: check commit ID instead
    if has_committed(&changes.total, last_total) {
        messages::celebrate_commit();
    }

    let mut state = LoopState {
        changes: changes.clone(),
        snooze_time: None,
    };

    print!("{}{}", cursor::Up(7), clear::CurrentLine);
    graph::print_status_display(options, &state);
    if changes.total > options.threshold {
        on_threshold_exceeded(&last_state);
    } else {
        on_below_threshold();
    }

    print_key_reminders();
    if changes.total > options.threshold {
        play_sound(options);
    }

    Ok(state)
}

fn get_next_snooze_time(options: &Options, last_state: &&mut LoopState) -> Option<DateTime<Local>> {
    match should_unsnooze(options, &last_state) {
        true => None,
        false => last_state.snooze_time.clone(),
    }
}

fn should_unsnooze(options: &Options, last_state: &&mut LoopState) -> bool {
    let should_unsnooze = match &last_state.snooze_time {
        Some(snooze_time) => {
            let now = chrono::Local::now();
            let diff = snooze_time.clone() - now;
            diff.num_minutes() > options.snooze_length
        }
        None => false,
    };
    should_unsnooze
}

/** TODO: check commit ID instead */
fn has_committed(total: &i32, last_total: &i32) -> bool {
    *total == 0 && *last_total > 0
}

fn print_key_reminders() {
    messages::press_q_to_quit();
}

fn on_below_threshold() {
    messages::watching_for_changes();
    messages::keep_up_the_good_work();
}

fn on_threshold_exceeded(state: &LoopState) {
    if state.snooze_time == None {
        messages::time_to_commit();
        messages::press_space_to_snooze();
    } else {
        messages::snoozing(state);
    }
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

            let total: i32 = &insertions + &deletions;

            Ok(Changes {
                insertions,
                deletions,
                total,
            })
        }

        Err(_) => Ok(Changes {
            insertions: 0,
            deletions: 0,
            total: 0,
        }),
    }
}
