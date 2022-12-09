use std::io;

use crossterm::{
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Terminal,
};

use crate::threshold_gauge::ThresholdGauge;
use crate::UiMessage;

pub async fn ui_loop(mut rx: tokio::sync::mpsc::Receiver<UiMessage>) {
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let threshold = 100.0;
    let max_value = 150.0;

    while let Some(cmd) = rx.recv().await {
        if let UiMessage::GitUpdate { git_state } = cmd {
            let insertions: u16 = git_state.git_changes.insertions as u16;
            let deletions: u16 = git_state.git_changes.deletions as u16;
            let total = git_state.git_changes.total;

            terminal
                .draw(|f| {
                    let overflow = ThresholdGauge::default()
                        .block(Block::default().borders(Borders::NONE).title("Progress"))
                        .gauge_style(
                            Style::default()
                                .bg(Color::Black)
                                .add_modifier(Modifier::ITALIC),
                        )
                        .value_and_max_value(total as f64, max_value)
                        .threshold(threshold)
                        .use_unicode(true)
                        .label(format!("+{} / -{}", insertions, deletions));

                    let size = f.size();

                    f.render_widget(
                        overflow,
                        Rect {
                            x: 2,
                            y: 2,
                            width: size.width - 2,
                            height: 3,
                        },
                    );
                })
                .unwrap();
        }
    }
}
