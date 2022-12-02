use chrono::Local;
use config::{Config, File};
use regex::Regex;
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
use termion::{clear, color};
use tokio::{spawn, time};

struct Options {
    threshold: i32,
    loop_time: u64,
    #[allow(dead_code)]
    volume: f32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut config_path = PathBuf::new();
    config_path.push(env::var("HOME").unwrap());
    config_path.push(".config");
    config_path.push("diffding");

    let settings = Config::builder()
        // Start off by merging in the "default" configuration file
        .add_source(File::from(config_path.join("config.toml")).required(false))
        // You may also programmatically change settings
        .build()?;

    println!(
        "CONFIG: {:?}",
        settings
            .try_deserialize::<HashMap<String, String>>()
            .unwrap()
    );

    let mut stdout = stdout().into_raw_mode().unwrap();

    let args: Vec<String> = env::args().collect();

    let options: Options = match args.len() {
        1 => Options {
            threshold: 100,
            loop_time: 10,
            volume: 1_f32,
        },
        2 => Options {
            threshold: 100,
            loop_time: args[1].parse::<u64>().unwrap(),
            volume: 1_f32,
        },
        3 => Options {
            threshold: args[2].parse::<i32>().unwrap(),
            loop_time: args[1].parse::<u64>().unwrap(),
            volume: 1_f32,
        },
        4 => Options {
            threshold: args[2].parse::<i32>().unwrap(),
            loop_time: args[1].parse::<u64>().unwrap(),
            volume: args[3].parse::<f32>().unwrap(),
        },
        _ => Options {
            threshold: 100,
            loop_time: 10,
            volume: 1_f32,
        },
    };

    splash_screen(options.loop_time, options.threshold);

    let forever = spawn(async move {
        let mut interval = time::interval(Duration::from_secs(options.loop_time));
        let mut last_count = 0;

        loop {
            interval.tick().await;
            last_count = alert_loop(options.threshold, &last_count).await.unwrap();
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

fn splash_screen(loop_time: u64, threshold: i32) {
    println!("{}", clear::All);
    println!(
        "\n{red}DIFF DING: COMMIT REMINDER!{reset}\r\n",
        red = color::Fg(color::Red),
        reset = color::Fg(color::Reset)
    );

    println!(
        "{blue}Interval       : {lightWhite}{loop_time:?} {white}seconds{reset}\r",
        blue = color::Fg(color::Blue),
        lightWhite = color::Fg(color::LightWhite),
        white = color::Fg(color::White),
        loop_time = loop_time,
        reset = color::Fg(color::Reset)
    );

    println!(
        "{blue}Threshold      : {lightWhite}{threshold:?} {white}seconds{reset}\r\n\n",
        blue = color::Fg(color::Blue),
        lightWhite = color::Fg(color::LightWhite),
        white = color::Fg(color::White),
        threshold = threshold,
        reset = color::Fg(color::Reset)
    );

    println!(
        "{lightWhite}Press {red}Q{lightWhite} to quit{reset}\n\n\r",
        red = color::Fg(color::LightCyan),
        reset = color::Fg(color::Reset),
        lightWhite = color::Fg(color::LightWhite)
    );
}

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
async fn alert_loop(threshold: i32, last_count: &i32) -> Result<i32, Box<dyn Error>> {
    let changes = count_changes().unwrap();

    let total = changes.total;

    if total == 0 && last_count > &0 {
        println!(
            "{}-----{}🎉 COMMITTED 🎉{}-----{}\r",
            color::Fg(color::White),
            color::Fg(color::Blue),
            color::Fg(color::White),
            color::Fg(color::Reset)
        );
    }

    let date = Local::now();

    print!("{} -- ", date.format("%H:%M:%S"));
    draw_graph(changes, threshold);
    println!("\r");
    if total > threshold {
        println!(
            "{yellow}!!!{lightRed} TIME TO COMMIT {yellow}!!!{reset}\r",
            lightRed = color::Fg(color::LightRed),
            yellow = color::Fg(color::LightYellow),
            reset = color::Fg(color::Reset)
        );
        println!(
            "{red}Press space to snooze{reset}\r",
            red = color::Fg(color::Red),
            reset = color::Fg(color::Reset)
        );
        play_sound();
    }

    Ok(total)
}

struct Changes {
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

fn draw_graph(changes: Changes, threshold: i32) {
    let total = changes.total;
    let graph_width = 40;
    let graph_threshold: i32 = (graph_width as f32 * 0.66) as i32;
    for i in 1..=graph_width {
        let _absolute_point = (i as f32) / graph_width as f32;
        let relative_point: f32 = (i as f32) / (graph_threshold as f32);
        let current: f32 = (total as f32) / (threshold as f32);
        let ratio = current / relative_point;

        // print divider
        if (relative_point - 1.0).abs() < 0.001 {
            print!("{}█", color::Fg(color::LightWhite));
        } else if ratio > 1.0 {
            if relative_point > 1.0 {
                print!("{}█", color::Fg(color::LightRed));
            } else if relative_point > 0.66 {
                print!("{}█", color::Fg(color::LightYellow));
            } else {
                print!("{}█", color::Fg(color::LightGreen));
            }
        } else {
            print!("{}█", color::Fg(color::White));
        }
    }
    print!(
        " {lightWhite}({white}{changes}/{threshold} {lightWhite}| {green}+{inserts} {red}-{deletes}{lightWhite}){reset}",
        lightWhite = color::Fg(color::LightWhite),
        white = color::Fg(color::White),
        green = color::Fg(color::LightGreen),
        red = color::Fg(color::LightRed),
        inserts = changes.insertions,
        deletes = changes.deletions,
        changes = total,
        threshold = threshold,
        reset = color::Fg(color::Reset)
    );
}
