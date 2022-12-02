use termion::{clear, color};

pub fn splash_screen(loop_time: u64, threshold: i32) {
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
