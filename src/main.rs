use regex::Regex;
use std::error::Error;
use std::process::Command;
use std::str;

fn run() -> Result<(), Box<dyn Error>> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--shortstat")
        .output()?;

    let stdout = str::from_utf8(&output.stdout)?;
    let re = Regex::new(r"((\d+)\D+)((\d+)\D+)?((\d+)?\D+)?")?;
    let captures = re.captures(stdout).ok_or("No match")?;

    println!("{:?}", captures);
    Ok(())
}

// some change
fn main() -> Result<(), Box<dyn Error>> {
    run()
}
