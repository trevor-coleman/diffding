use regex::Regex;
use soloud::*;
use std::error::Error;
use std::process::Command;
use std::time::Duration;
use std::{env, str};
use tokio::{task, time};

// fn terminal_bell() {
//     print!("\x07");
// }

fn play_sound() {
    let sl = Soloud::default().unwrap();
    let mut wav = audio::Wav::default();
    wav.load_mem(include_bytes!("./387533__soundwarf__alert-short.wav"))
        .unwrap();
    sl.play(&wav);
    while sl.voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
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
    let captures = re.captures(stdout).ok_or("No match")?;

    let additions = captures.get(4).ok_or("0");
    let deletions = captures.get(6).ok_or("0");

    //convert additions to i32
    let additions = additions.unwrap().as_str().parse::<i32>()?;
    let deletions = deletions.unwrap().as_str().parse::<i32>()?;

    let total = additions + deletions;

    println!("You've changed {:?} lines", total);
    if total > line_count {
        println!("TIME TO COMMIT");
        play_sound();
    } else {
        println!("NO WORRIES MATE");
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let loop_time: u64;
    let line_count: i32;
    match args.len() {
        0 => {
            loop_time = 10;
            line_count = 100;
        }
        1 => {
            loop_time = args[0].parse::<u64>().unwrap();
            line_count = 100;
        }
        2 => {
            loop_time = args[0].parse::<u64>().unwrap();
            line_count = args[1].parse::<i32>().unwrap();
        }
        _ => {
            loop_time = 10;
            line_count = 100;
        }
    }

    let forever = task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(loop_time));

        loop {
            interval.tick().await;
            run(line_count).await.unwrap();
        }
    });

    forever.await.unwrap();
}

//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
//comments
