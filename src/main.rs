use chrono::Local;
use regex::Regex;
use soloud::*;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::process::Command;
use std::time::Duration;
use std::{env, str};
use termion::cursor::{DetectCursorPos, Goto};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
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
async fn run(threshold: i32, mut last_count: &i32) -> Result<i32, Box<dyn Error>> {
    let total = count_changes().unwrap();

    if total == 0 && last_count > &0 {
        println!(
            "{}-----{}🎉 COMMITTED 🎉{}-----{}",
            color::Fg(color::White),
            color::Fg(color::Blue),
            color::Fg(color::White),
            color::Fg(color::Reset)
        );
    }

    let date = Local::now();

    print!("{} -- ", date.format("%H:%M:%S"));
    draw_graph(total, threshold);
    println!(" -- You've made {:?} insertions and deletions\r", total);
    if total > threshold {
        println!(
            "{yellow}!!!{lightRed} TIME TO COMMIT {yellow}!!!{reset}\r",
            lightRed = color::Fg(color::LightRed),
            yellow = color::Fg(color::LightYellow),
            reset = color::Fg(color::Reset)
        );
        play_sound();
    }

    Ok(total)
}

fn count_changes() -> Result<i32, Box<(dyn std::error::Error + 'static)>> {
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

            Ok(insertions + deletions)
        }
        Err(_) => Ok(0),
    }
}

fn draw_graph(changes: i32, threshold: i32) {
    let graph_width = 40;
    let graph_threshold = graph_width / 2;
    for i in 1..=graph_width {
        let point: f32 = (i as f32) / 20_f32;
        let current: f32 = (changes as f32) / (threshold as f32);
        let ratio = current / point;

        if ratio > 1.0 {
            print!("{}{}", color::Fg(color::LightRed), "█");
        } else {
            print!("{}{}", color::Fg(color::LightGreen), "█");
        }

        print!("{}", color::Fg(color::Reset));
    }
}

struct Options {
    threshold: i32,
    loop_time: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout().into_raw_mode().unwrap();

    let args: Vec<String> = env::args().collect();
    let loop_time: u64;
    let threshold: i32;

    let options: Options;

    match args.len() {
        1 => {
            options = Options {
                threshold: 100,
                loop_time: 10,
            };
        }
        2 => {
            options = Options {
                threshold: 100,
                loop_time: args[1].parse::<u64>().unwrap(),
            };
        }
        3 => {
            options = Options {
                threshold: args[2].parse::<i32>().unwrap(),
                loop_time: args[1].parse::<u64>().unwrap(),
            };
        }
        _ => {
            options = Options {
                threshold: 100,
                loop_time: 10,
            };
        }
    }

    splash_screen(options.loop_time, options.threshold);

    let forever = spawn(async move {
        let mut interval = time::interval(Duration::from_secs(options.loop_time));
        let mut last_count = 0;

        loop {
            interval.tick().await;
            last_count = run(options.threshold, &last_count).await.unwrap();
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
        let mut i = 0;
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
