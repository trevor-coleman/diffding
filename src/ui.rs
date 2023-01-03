use std::io;
use std::io::Stdout;
use std::sync::{Arc, Mutex};

use chrono::{Duration, Local};
use crossterm::{
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Paragraph, Wrap};
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Frame, Terminal,
};

use crate::manager::AppState;
use crate::threshold_gauge::ThresholdGauge;
use crate::{GitState, Options};

#[derive(Debug)]
pub enum UiMessage {
    GitUpdate {
        git_state: GitState,
        app_state: Arc<Mutex<AppState>>,
    },
}

pub async fn ui_loop<'ui>(mut rx: tokio::sync::mpsc::Receiver<UiMessage>, options: Arc<Options>) {
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let threshold = f64::from(options.threshold);
    let max_value = threshold * 1.5;

    let wide_width = 100;

    while let Some(ui_message) = rx.recv().await {
        use UiMessage::*;

        match ui_message {
            GitUpdate {
                git_state,
                app_state,
            } => {
                draw_ui(
                    options.clone(),
                    &mut terminal,
                    threshold,
                    max_value,
                    wide_width,
                    git_state,
                    app_state,
                );
            }
        }
    }
}

fn draw_ui(
    options: Arc<Options>,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    threshold: f64,
    max_value: f64,
    wide_width: u16,
    git_state: GitState,
    app_state: Arc<Mutex<AppState>>,
) {
    let git_state_draw = git_state.clone();
    let title = git_summary(git_state);

    terminal
        .draw(|f| {
            f.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Black)),
                f.size(),
            );
            let top_split = Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(2),
                        Constraint::Min(0),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let app_title_area = top_split[0];
            let bar_area = top_split[1];
            let data_display_area = top_split[3];
            let footer_area = top_split[4];

            let is_wide = f.size().width > wide_width;

            let data_display = get_data_display(data_display_area, is_wide);

            draw_app_title(f, app_title_area);

            draw_bar(
                threshold,
                max_value,
                git_state_draw.git_changes.total.clone(),
                title,
                f,
                bar_area,
            );

            let options_summary = options.clone();
            crate::summary::summary(f, data_display[2], &git_state_draw, options_summary);

            big_text(f, data_display[0], &git_state_draw);

            draw_footer(f, footer_area, app_state.clone(), options.snooze_length);

            // debug_info(f, is_wide);
        })
        .unwrap();
}

fn is_snoozed(app_state: Arc<Mutex<AppState>>) -> bool {
    app_state.lock().unwrap().snoozed
}

fn draw_footer(
    f: &mut Frame<CrosstermBackend<Stdout>>,
    footer_area: Rect,
    app_state: Arc<Mutex<AppState>>,
    snooze_length: i64,
) {
    let mut quit_command = command_prompt("Q".to_string(), "quit".to_string(), Color::LightRed);
    let snooze_duration = &Duration::seconds(snooze_length);

    let app_state1 = app_state.clone();
    let mut snooze_command = match get_time_left(app_state1, snooze_duration) {
        None => command_prompt(
            "<space>".to_string(),
            "snooze".to_string(),
            Color::LightCyan,
        ),
        Some((time_left_text, time_left_units)) => vec![
            Span::styled("Snoozed:", Style::default().fg(Color::LightCyan)),
            Span::styled(
                format!(" {time_left_text} "),
                Style::default().fg(Color::LightCyan),
            ),
            Span::styled(
                format!("{time_left_units} "),
                Style::default().fg(Color::LightCyan),
            ),
            Span::styled("remaining", Style::default().fg(Color::LightCyan)),
        ],
    };

    let mut spacer = vec![Span::styled(" / ", Style::default().fg(Color::White))];

    let commands = &mut Vec::<Span>::new();
    commands.append(quit_command.as_mut());
    commands.append(spacer.as_mut());
    commands.append(snooze_command.as_mut());
    let commands = Spans::from(commands.clone());

    let footer = Paragraph::new(commands)
        .block(Block::default().borders(Borders::NONE))
        .alignment(tui::layout::Alignment::Left)
        .style(Style::default().fg(Color::White).bg(Color::Indexed(237)));

    f.render_widget(footer, footer_area);
}

fn get_time_left(
    app_state: Arc<Mutex<AppState>>,
    snooze_duration: &Duration,
) -> Option<(String, String)> {
    let app_state = app_state.lock().unwrap();
    if let Some(snoozed_at) = app_state.snoozed_at {
        let time_left = *snooze_duration - (Local::now() - snoozed_at);
        return if time_left.num_minutes() == 1 {
            Some((time_left.num_minutes().to_string(), "minute".to_string()))
        } else if time_left.num_minutes() > 0 {
            Some((time_left.num_minutes().to_string(), "minutes".to_string()))
        } else {
            Some(("less than 1".to_string(), "minute".to_string()))
        };
    }
    return None;
}

fn draw_bar(
    threshold: f64,
    max_value: f64,
    total: i32,
    title: Spans,
    f: &mut Frame<CrosstermBackend<Stdout>>,
    bar_area: Rect,
) {
    let bar_graph = ThresholdGauge::default()
        .block(Block::default().borders(Borders::NONE).title(title))
        .gauge_style(
            Style::default()
                .bg(Color::Indexed(237))
                .add_modifier(Modifier::ITALIC),
        )
        .value_and_max_value(total as f64, max_value)
        .threshold(threshold)
        .use_unicode(true);

    f.render_widget(bar_graph, bar_area);
}

fn draw_app_title(f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
    let app_title = Paragraph::new("DiffDing - Commit Reminder")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: true });

    f.render_widget(app_title, area);
}

fn get_data_display(area: Rect, is_wide: bool) -> Vec<Rect> {
    let direction = if is_wide {
        tui::layout::Direction::Horizontal
    } else {
        tui::layout::Direction::Vertical
    };

    let constraints = if is_wide {
        [
            Constraint::Length(64),
            Constraint::Length(5),
            Constraint::Length(30),
        ]
        .as_ref()
    } else {
        [
            Constraint::Length(5),
            Constraint::Length(2),
            Constraint::Length(5),
        ]
        .as_ref()
    };

    Layout::default()
        .direction(direction)
        .margin(1)
        .constraints(constraints)
        .split(area)
}

#[allow(dead_code)]
fn debug_info(f: &mut Frame<CrosstermBackend<Stdout>>, is_wide: bool) {
    let screen_dimension = Span::styled(
        format!(
            "is_wide: {}, Width {:?}\n\r`",
            is_wide,
            f.size().width.to_string()
        ),
        Style::default().fg(Color::LightMagenta),
    );

    let len = screen_dimension.content.len() as u16;
    let paragraph = Paragraph::new(Text::from(screen_dimension))
        .block(Block::default().borders(Borders::NONE))
        .alignment(tui::layout::Alignment::Right);

    f.render_widget(paragraph, Rect::new(0, 0, len + 5, 4));
}

fn big_text(f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, git_state: &GitState) {
    let message = match git_state.is_above_threshold() {
        true => "COMMIT!",
        false => "OK!",
    };

    let output = cfonts::render(cfonts::Options {
        text: String::from(message),
        font: cfonts::Fonts::FontBlock,
        line_height: area.height,
        spaceless: true,
        ..cfonts::Options::default()
    });
    let vec = output.vec;
    let mut max_width: u16 = 0;
    for line in vec {
        max_width = max_width.max(line.len() as u16);
    }

    let text = Text::from(output.text);

    let fg = match git_state.is_above_threshold() {
        true => Color::LightRed,
        false => Color::LightGreen,
    };

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(fg).bg(Color::Black))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn git_summary<'ui>(git_state: GitState) -> Spans<'ui> {
    let insertions: u16 = git_state.git_changes.insertions as u16;
    let deletions: u16 = git_state.git_changes.deletions as u16;

    let title: Spans<'ui> = Spans::from(vec![
        Span::styled(
            format!("{} ", git_state.current_commit_short),
            Style::default().fg(Color::White).bg(Color::Black),
        ),
        Span::styled(
            format!("+{}", insertions),
            Style::default().fg(Color::LightGreen).bg(Color::Black),
        ),
        Span::styled("/", Style::default().fg(Color::White).bg(Color::Black)),
        Span::styled(
            format!("-{}", deletions),
            Style::default().fg(Color::LightRed).bg(Color::Black),
        ),
    ]);
    title
}

fn command_prompt<'a>(key_name: String, action: String, color: Color) -> Vec<Span<'a>> {
    let prompt = vec![
        Span::styled("Press ", Style::default().fg(Color::LightYellow)),
        Span::styled(key_name, Style::default().fg(color)),
        Span::styled(
            format!(" to {}", action),
            Style::default().fg(Color::LightYellow),
        ),
    ];

    prompt
}
