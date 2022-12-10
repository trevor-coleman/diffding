use std::io;
use std::io::{BufRead, Stdout};

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
    execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let threshold = 200.0;
    let max_value = threshold * 1.5;

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
                    .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                    .split(f.size());

                let bottom_split = Layout::default()
                    .direction(tui::layout::Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(50), Constraint::Min(64)].as_ref())
                    .split(top_split[1]);

                let overflow = ThresholdGauge::default()
                    .block(Block::default().borders(Borders::NONE).title(title))
                    .gauge_style(
                        Style::default()
                            .bg(Color::Black)
                            .add_modifier(Modifier::ITALIC),
                    )
                    .value_and_max_value(total as f64, max_value)
                    .threshold(threshold)
                    .use_unicode(true);

                f.render_widget(overflow, top_split[0]);
                crate::summary::summary(f, bottom_split[0], &git_state_draw);

                big_text(f, bottom_split[1], &git_state_draw);
            })
            .unwrap();
    }
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
