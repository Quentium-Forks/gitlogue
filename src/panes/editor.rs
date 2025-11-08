use crate::animation::AnimationEngine;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct EditorPane;

impl EditorPane {
    pub fn render(&self, f: &mut Frame, area: Rect, engine: &AnimationEngine) {
        let block = Block::default()
            .title("Editor")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        // Get visible lines based on scroll offset and area height
        let content_height = area.height.saturating_sub(2) as usize; // Subtract borders
        let scroll_offset = engine.buffer.scroll_offset;
        let buffer_lines = &engine.buffer.lines;

        let visible_lines: Vec<Line> = buffer_lines
            .iter()
            .skip(scroll_offset)
            .take(content_height)
            .enumerate()
            .map(|(idx, line_content)| {
                let line_num = scroll_offset + idx;

                // Check if cursor is on this line
                if line_num == engine.buffer.cursor_line && engine.cursor_visible {
                    // Insert cursor character
                    let mut spans = Vec::new();
                    let cursor_col = engine.buffer.cursor_col;

                    if cursor_col > 0 {
                        spans.push(Span::raw(&line_content[..cursor_col.min(line_content.len())]));
                    }

                    // Cursor
                    let cursor_char = if cursor_col < line_content.len() {
                        line_content.chars().nth(cursor_col).unwrap_or(' ')
                    } else {
                        ' '
                    };

                    spans.push(Span::styled(
                        cursor_char.to_string(),
                        Style::default()
                            .bg(Color::White)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    ));

                    if cursor_col + 1 < line_content.len() {
                        spans.push(Span::raw(&line_content[cursor_col + 1..]));
                    }

                    Line::from(spans)
                } else {
                    Line::from(line_content.clone())
                }
            })
            .collect();

        let content = Paragraph::new(visible_lines).block(block);
        f.render_widget(content, area);
    }
}
