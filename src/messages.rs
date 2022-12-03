use termion::color;
use termion::color::{Fg, Green, LightCyan, LightRed, LightYellow, Reset, White};

pub fn celebrate_commit() {
    println!(
        "\n\n\r{}-----{}🎉 COMMITTED 🎉{}-----{}\n\n\r",
        Fg(White),
        Fg(color::Blue),
        Fg(White),
        Fg(Reset)
    );
}

pub fn time_to_commit() {
    println!(
        "\n\r{yellow}!!!{lightRed} TIME TO COMMIT {yellow}!!!{reset}\n\r",
        lightRed = Fg(LightRed),
        yellow = Fg(LightYellow),
        reset = Fg(Reset)
    );
}

pub fn press_space_to_snooze() {
    println!(
        "{white}Press space to snooze for {lightCyan}5 {white}minutes. {reset}\r",
        white = Fg(White),
        reset = Fg(Reset),
        lightCyan = Fg(LightCyan)
    );
}

pub fn watching_for_changes() {
    println!(
        "\n\r{white}Watching for changes...{reset}\n\r",
        white = Fg(White),
        reset = Fg(Reset)
    );
}

pub fn keep_up_the_good_work() {
    println!(
        "{green}👍🏻 Keep up the good work!{reset}\r",
        green = Fg(Green),
        reset = Fg(Reset)
    );
}

pub fn press_q_to_quit() {
    println!(
        "\n\r{lightWhite}Press {red}Q{lightWhite} to quit{reset}\r",
        red = Fg(LightCyan),
        reset = Fg(Reset),
        lightWhite = Fg(color::LightWhite)
    );
}
