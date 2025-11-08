use crate::git::CommitMetadata;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct FileTreePane;

impl FileTreePane {
    pub fn render(&self, f: &mut Frame, area: Rect, metadata: Option<&CommitMetadata>) {
        let block = Block::default()
            .title("File Tree")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let lines = if let Some(meta) = metadata {
            meta.changes
                .iter()
                .map(|change| {
                    let (status_char, color) = match change.status.as_str() {
                        "A" => ("+", Color::Green),
                        "D" => ("-", Color::Red),
                        "M" => ("~", Color::Yellow),
                        "R" => (">", Color::Blue),
                        _ => (" ", Color::White),
                    };

                    Line::from(vec![
                        Span::styled(
                            format!("{} ", status_char),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(&change.path),
                    ])
                })
                .collect()
        } else {
            vec![Line::from("No commit loaded")]
        };

        let content = Paragraph::new(lines).block(block);
        f.render_widget(content, area);
    }
}
