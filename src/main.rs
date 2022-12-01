use chrono::Local;
use regex::Regex;
use soloud::*;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::process::Command;
use std::time::Duration;
use std::{env, str};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color};
use tokio::{spawn, time};

// fn terminal_bell() {
//     print!("\x07");
// }

fn play_sound() {
    let sl = Soloud::default().unwrap();
    let mut wav = Wav::default();
    wav.load_mem(include_bytes!("./387533__soundwarf__alert-short.wav"))
        .unwrap();
    sl.play(&wav);
    while sl.voice_count() > 0 {
        std::thread::sleep(Duration::from_millis(100));
    }
}

// a change
async fn run(line_count: i32) -> Result<(), Box<dyn Error>> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--shortstat")
        .output()?;

    let stdout = str::from_utf8(&output.stdout)?;
    let re = Regex::new(r"((\d+)\D+)((\d+)\D+)?((\d+)?\D+)?")?;
    let captures = re.captures(stdout).ok_or("No match");

    let total: i32;
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

            total = insertions + deletions;
        }
        Err(_) => {
            total = 0;
        }
    }

    let date = Local::now();

    println!(
        "{} - You've changed {:?} lines\r",
        date.format("%H:%M:%S"),
        total
    );
    if total > line_count {
        println!(
            "{yellow}!!!{lightRed} TIME TO COMMIT {yellow}!!!{reset}\r",
            lightRed = color::Fg(color::LightRed),
            yellow = color::Fg(color::LightYellow),
            reset = color::Fg(color::Reset)
        );
        play_sound();
    } else {
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout().into_raw_mode().unwrap();

    let args: Vec<String> = env::args().collect();
    let loop_time: u64;
    let line_count: i32;
    match args.len() {
        1 => {
            loop_time = 10;
            line_count = 100;
        }
        2 => {
            loop_time = args[1].parse::<u64>().unwrap();
            line_count = 100;
        }
        3 => {
            loop_time = args[1].parse::<u64>().unwrap();
            line_count = args[2].parse::<i32>().unwrap();
        }
        _ => {
            loop_time = 10;
            line_count = 100;
        }
    }

    splash_screen(loop_time, line_count);

    let forever = spawn(async move {
        let mut interval = time::interval(Duration::from_secs(loop_time));

        loop {
            interval.tick().await;
            run(line_count).await.unwrap();
        }
    });

    let quit = move || {
        println!("\r");
        println!("Quitting\r");
        //turn off raw mode

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
                        println!("Snoozing!\r")
                    }
                    Key::Char('l') => {
                        print!("Lalalalala");
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

fn splash_screen(loop_time: u64, line_count: i32) {
    println!("{}", clear::All);
    println!(
        "\n{red}COMMIT REMINDER!{reset}\r\n\n",
        red = color::Fg(color::Red),
        reset = color::Fg(color::Reset)
    );

    println!(
        "{blue}Loop time      : {white}{loop_time:?} seconds{reset}\r",
        blue = color::Fg(color::Blue),
        white = color::Fg(color::White),
        loop_time = loop_time,
        reset = color::Fg(color::Reset)
    );

    println!(
        "{blue}Changes allowed: {white}{line_count:?} seconds{reset}\r\n\n",
        blue = color::Fg(color::Blue),
        white = color::Fg(color::White),
        line_count = line_count,
        reset = color::Fg(color::Reset)
    );
}
