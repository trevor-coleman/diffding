use std::io;
use std::io::Stdout;

use cfonts;
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

use crate::threshold_gauge::ThresholdGauge;
use crate::{GitState, UiMessage};

pub async fn ui_loop<'ui>(mut rx: tokio::sync::mpsc::Receiver<UiMessage>) {
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let threshold = 200.0;
    let max_value = threshold * 1.5;

    let wide_width = 100;

    while let Some(cmd) = rx.recv().await {
        let UiMessage::GitUpdate { git_state } = cmd;

        let git_state_draw = git_state.clone();
        let total = git_state.git_changes.total;

        let title = git_summary::<'ui>(git_state);

        terminal
            .draw(|f| {
                let top_split = Layout::default()
                    .direction(tui::layout::Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Length(3),
                            Constraint::Length(2),
                            Constraint::Min(0),
                            Constraint::Length(2),
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

                draw_bar(threshold, max_value, total, title, f, bar_area);

                crate::summary::summary(f, data_display[2], &git_state_draw);

                big_text(f, data_display[0], &git_state_draw);

                let mut quit_command =
                    command_prompt("Q".to_string(), "quit".to_string(), Color::LightYellow);
                let mut snooze_command = command_prompt(
                    "<space>".to_string(),
                    "snooze".to_string(),
                    Color::LightCyan,
                );

                let mut spacer = vec![Span::styled(" / ", Style::default().fg(Color::White))];

                let commands = &mut Vec::<Span>::new();
                commands.append(quit_command.as_mut());
                commands.append(spacer.as_mut());
                commands.append(snooze_command.as_mut());
                let commands = Spans::from(commands.clone());

                let footer = Paragraph::new(commands)
                    .block(Block::default().borders(Borders::TOP))
                    .alignment(tui::layout::Alignment::Left)
                    .style(Style::default().fg(Color::White));

                f.render_widget(footer, footer_area);

                // debug_info(f, is_wide);
            })
            .unwrap();
    }
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
                .bg(Color::Black)
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

    let bottom_split = Layout::default()
        .direction(direction)
        .margin(1)
        .constraints(constraints)
        .split(area);
    bottom_split
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
        .style(Style::default().fg(fg).bg(Color::Reset))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn git_summary<'ui>(git_state: GitState) -> Spans<'ui> {
    let insertions: u16 = git_state.git_changes.insertions as u16;
    let deletions: u16 = git_state.git_changes.deletions as u16;

    let title: Spans<'ui> = Spans::from(vec![
        Span::styled(
            format!("{} ", git_state.current_commit_short),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            format!("+{}", insertions),
            Style::default().fg(Color::LightGreen),
        ),
        Span::styled("/", Style::default().fg(Color::White)),
        Span::styled(
            format!("-{}", deletions),
            Style::default().fg(Color::LightRed),
        ),
    ]);
    title
}

fn command_prompt<'a>(key_name: String, action: String, color: Color) -> Vec<Span<'a>> {
    let prompt = vec![
        Span::styled("Press ", Style::default().fg(Color::White)),
        Span::styled(key_name, Style::default().fg(color)),
        Span::styled(format!(" to {}", action), Style::default().fg(Color::White)),
    ];

    prompt
}
