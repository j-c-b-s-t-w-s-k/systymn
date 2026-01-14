use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use unicode_width::UnicodeWidthStr;

use crate::app::App;

pub fn draw_editor(frame: &mut Frame, app: &mut App, area: Rect) {
    let inner = Block::default()
        .title(" Systymn ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .inner(area);

    frame.render_widget(
        Block::default()
            .title(" Systymn ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)),
        area,
    );

    let wrap_width = inner.width as usize;
    app.wrap_width = wrap_width;
    app.set_visible_height(inner.height as usize);

    // Build the display with ghost text and line wrapping
    let mut lines: Vec<Line> = Vec::new();
    let (cursor_x, cursor_y) = app.buffer.cursor();

    // Get wrapped lines
    let wrapped_lines = app.buffer.get_wrapped_lines(wrap_width);
    let mut visual_cursor_y = 0;
    let mut visual_cursor_x = 0;
    let mut found_cursor = false;

    // Track which original line we're on
    let mut current_orig_line = 0;
    let mut offset_in_orig_line = 0;

    // Get selection bounds if any - selection_range returns ((start_x, start_y), (end_x, end_y))
    let selection_bounds = app.buffer.get_selection_range();

    for (orig_line_idx, segment) in &wrapped_lines {
        if *orig_line_idx != current_orig_line {
            current_orig_line = *orig_line_idx;
            offset_in_orig_line = 0;
        }

        let mut spans: Vec<Span> = Vec::new();

        // Check if cursor is on this line
        let is_cursor_line = *orig_line_idx == cursor_y;
        let segment_start = offset_in_orig_line;
        let segment_end = offset_in_orig_line + segment.len();

        // Helper to apply highlighting (selection and search matches)
        let apply_highlighting = |text: &str, line_idx: usize, col_start: usize| -> Vec<Span> {
            let mut result: Vec<Span> = Vec::new();
            let text_chars: Vec<char> = text.chars().collect();
            let mut i = 0;

            while i < text_chars.len() {
                let abs_col = col_start + i;

                // Check for search match at this position
                let search_match = app.search.matches.iter().enumerate().find(|(_, m)| {
                    m.line == line_idx && abs_col >= m.start && abs_col < m.end
                });

                // Check for selection at this position - selection_bounds is ((start_x, start_y), (end_x, end_y))
                let in_selection = if let Some(((start_x, start_y), (end_x, end_y))) = &selection_bounds {
                    (line_idx > *start_y || (line_idx == *start_y && abs_col >= *start_x))
                        && (line_idx < *end_y || (line_idx == *end_y && abs_col < *end_x))
                } else {
                    false
                };

                if let Some((match_idx, m)) = search_match {
                    // Collect all chars in this match
                    let match_len = (m.end - m.start).min(text_chars.len() - i);
                    let match_text: String = text_chars[i..i + match_len].iter().collect();

                    let is_current = match_idx == app.search.current_match;
                    let style = if is_current {
                        Style::default().fg(Color::Black).bg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Black).bg(Color::Rgb(200, 200, 100))
                    };

                    result.push(Span::styled(match_text, style));
                    i += match_len;
                } else if in_selection {
                    // Find end of selection in this span
                    let mut sel_end = i;
                    while sel_end < text_chars.len() {
                        let check_col = col_start + sel_end;
                        if let Some((_, (end_x, end_y))) = &selection_bounds {
                            if line_idx > *end_y || (line_idx == *end_y && check_col >= *end_x) {
                                break;
                            }
                        }
                        sel_end += 1;
                    }
                    let sel_text: String = text_chars[i..sel_end].iter().collect();
                    result.push(Span::styled(sel_text, Style::default().bg(Color::Blue).fg(Color::White)));
                    i = sel_end;
                } else {
                    // Regular text - collect until next highlight
                    let mut plain_end = i + 1;
                    while plain_end < text_chars.len() {
                        let check_col = col_start + plain_end;

                        // Check if next char is search match
                        let is_search = app.search.matches.iter().any(|m| {
                            m.line == line_idx && check_col >= m.start && check_col < m.end
                        });

                        // Check if next char is selection
                        let is_sel = if let Some(((start_x, start_y), (end_x, end_y))) = &selection_bounds {
                            (line_idx > *start_y || (line_idx == *start_y && check_col >= *start_x))
                                && (line_idx < *end_y || (line_idx == *end_y && check_col < *end_x))
                        } else {
                            false
                        };

                        if is_search || is_sel {
                            break;
                        }
                        plain_end += 1;
                    }
                    let plain_text: String = text_chars[i..plain_end].iter().collect();
                    result.push(Span::raw(plain_text));
                    i = plain_end;
                }
            }

            if result.is_empty() {
                result.push(Span::raw(String::new()));
            }
            result
        };

        if is_cursor_line && !found_cursor {
            // Check if cursor falls within this segment
            if cursor_x >= segment_start && cursor_x <= segment_end {
                found_cursor = true;
                visual_cursor_y = lines.len();
                visual_cursor_x = segment[..cursor_x.saturating_sub(segment_start).min(segment.len())].width();

                // Split at cursor position
                let local_cursor = cursor_x.saturating_sub(segment_start).min(segment.len());
                let before_cursor = &segment[..local_cursor];
                let after_cursor = &segment[local_cursor..];

                // Apply highlighting to text before cursor
                spans.extend(apply_highlighting(before_cursor, *orig_line_idx, segment_start));

                // Add ghost text suggestion with pulsing effect
                if let Some(suggestion) = &app.api_suggestion {
                    // API suggestion takes priority - show in different color
                    let pulse = app.pulse_phase;
                    let intensity = ((pulse * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5) * 0.4 + 0.3;
                    let blue_level = (intensity * 255.0) as u8;

                    spans.push(Span::styled(
                        suggestion.text.clone(),
                        Style::default().fg(Color::Rgb(100, 150, blue_level + 100))
                    ));
                } else if let Some(suggestion) = &app.current_suggestion {
                    let pulse = app.pulse_phase;
                    let intensity = ((pulse * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5) * 0.4 + 0.3;
                    let gray_level = (intensity * 255.0) as u8;

                    spans.push(Span::styled(
                        suggestion.text.clone(),
                        Style::default().fg(Color::Rgb(gray_level, gray_level + 20, gray_level + 40))
                    ));
                }

                // Add command preview if present
                if let Some(preview) = &app.command_preview {
                    spans.push(Span::styled(
                        format!(" -> {}", preview),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC)
                    ));
                }

                // Apply highlighting to text after cursor
                spans.extend(apply_highlighting(after_cursor, *orig_line_idx, segment_start + local_cursor));
            } else {
                spans.extend(apply_highlighting(segment, *orig_line_idx, segment_start));
            }
        } else {
            spans.extend(apply_highlighting(segment, *orig_line_idx, segment_start));
        }

        // Add wrap indicator for continued lines
        if offset_in_orig_line > 0 {
            spans.insert(0, Span::styled(
                "\u{21B3} ",
                Style::default().fg(Color::DarkGray)
            ));
        }

        lines.push(Line::from(spans));
        offset_in_orig_line = segment_end;
    }

    // Add sentence suggestion below current line
    if let Some(sentence) = &app.sentence_suggestion {
        let pulse = app.pulse_phase;
        let intensity = ((pulse * std::f32::consts::PI * 2.0).sin() * 0.3 + 0.5) * 0.5;
        let gray = (intensity * 200.0) as u8;

        lines.push(Line::from(vec![
            Span::styled(
                format!("  \u{2192} {}", sentence.text),
                Style::default()
                    .fg(Color::Rgb(gray, gray + 30, gray))
                    .add_modifier(Modifier::ITALIC)
            )
        ]));
    }

    // Handle scrolling
    let visible_height = inner.height as usize;
    let scroll_offset = if visual_cursor_y >= app.scroll_offset + visible_height.saturating_sub(1) {
        visual_cursor_y.saturating_sub(visible_height.saturating_sub(2))
    } else if visual_cursor_y < app.scroll_offset {
        visual_cursor_y
    } else {
        app.scroll_offset
    };
    app.scroll_offset = scroll_offset;

    // Only show visible lines
    let visible_lines: Vec<Line> = lines
        .into_iter()
        .skip(scroll_offset)
        .take(visible_height)
        .collect();

    let paragraph = Paragraph::new(visible_lines);
    frame.render_widget(paragraph, inner);

    // Position cursor (accounting for scroll)
    let screen_cursor_y = visual_cursor_y.saturating_sub(scroll_offset);
    frame.set_cursor_position(Position::new(
        inner.x + visual_cursor_x as u16,
        inner.y + screen_cursor_y as u16,
    ));
}
