use crate::{Changes, LoopState, Options};
use chrono::Local;
use termion::color;

pub fn draw_graph(changes: &Changes, threshold: &i32) {
    let graph_width = 40;
    let graph_threshold: i32 = (graph_width as f32 * 0.66) as i32;
    for i in 1..=graph_width {
        let _absolute_point = (i as f32) / (*&graph_width as f32);
        let relative_point: f32 = (i as f32) / (*&graph_threshold as f32);
        let current: f32 = (*&changes.total as f32) / (*threshold as f32);
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
        inserts = &changes.insertions,
        deletes = &changes.deletions,
        changes = &changes.total,
        threshold = threshold,
        reset = color::Fg(color::Reset)
    );
}

pub fn print_status_display(options: &Options, loop_state: &LoopState) {
    let date = Local::now();
    print!("{} -- ", date.format("%H:%M:%S"));
    draw_graph(&loop_state.changes, &options.threshold);
    println!("\r");
}
