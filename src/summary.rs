use std::sync::Arc;

use tui::backend::Backend;
use tui::layout::{Constraint, Rect};
use tui::style::{Modifier, Style};
use tui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use tui::Frame;

use crate::{GitState, Options};

pub fn summary<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    git_state: &GitState,
    options: Arc<Options>,
) {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let insertions = &git_state.git_changes.insertions.to_string();
    let deletions = &git_state.git_changes.deletions.to_string();
    let threshold = &git_state.threshold.to_string();
    let loop_time = &options.loop_time.to_string();
    let total = &git_state.git_changes.total.to_string();
    let total_string = &format!("{total} / {threshold}");
    let loop_time_string = &format!("{loop_time}ms");
    let items = vec![
        vec!["", ""],
        vec!["Insertions", insertions],
        vec!["Deletions", deletions],
        vec!["----------", "-----------------"],
        vec!["Total", total_string],
        vec!["", ""],
        vec!["Loop Time", loop_time_string],
    ];
    let rows = items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(*c));
        Row::new(cells).height(height as u16).bottom_margin(0)
    });
    let t = Table::new(rows)
        .block(Block::default().borders(Borders::NONE).title("STATUS"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[Constraint::Min(20), Constraint::Min(20)]);
    f.render_stateful_widget(t, area, &mut TableState::default());
}
