use crate::Options;
use termion::{clear, color};

pub fn splash_screen(options: &Options) {
    println!("{}", clear::All);
    println!(
        "{title_color}DIFF DING: COMMIT REMINDER!{reset}\r",
        title_color = color::Fg(color::LightCyan),
        reset = color::Fg(color::Reset)
    );
    println!(
        "{underline_color}----------------------------{reset}\r\n",
        underline_color = color::Fg(color::LightCyan),
        reset = color::Fg(color::Reset)
    );

    print_option("Loop Time", &options.loop_time.to_string(), Some("seconds"));
    print_option(
        "Threshold",
        &options.threshold.to_string(),
        Some("inserts and deletes"),
    );
    print_option(
        "Sound",
        match &options.sound {
            None => "Default",
            Some(path) => &path.to_str().unwrap(),
        },
        None,
    );
    print_option("Volume", &options.volume.to_string(), Some("/1.0"));

    println!("\n\n\n\n\n\n\n\n\n\r")
}

fn print_option(name: &str, value: &str, description: Option<&str>) {
    let description = match description {
        Some(desc) => desc,
        None => "",
    };
    println!(
        "{blue}{name:10}: {lightWhite}{value} {white}{description}{reset}\r",
        blue = color::Fg(color::Blue),
        lightWhite = color::Fg(color::LightWhite),
        white = color::Fg(color::White),
        name = name,
        value = value,
        reset = color::Fg(color::Reset),
        description = description
    );
}
