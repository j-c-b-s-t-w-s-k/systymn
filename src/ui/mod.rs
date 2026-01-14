mod editor;
mod suggestions;
pub mod synonyms;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Clear};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &mut App) {
    // Adjust layout based on whether search bar is visible
    let chunks = if app.search.is_active {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .split(frame.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(3),
            ])
            .split(frame.area())
    };

    editor::draw_editor(frame, app, chunks[0]);

    if app.search.is_active {
        draw_search_bar(frame, app, chunks[1]);
        draw_status_bar(frame, app, chunks[2]);
    } else {
        draw_status_bar(frame, app, chunks[1]);
    }

    if app.show_synonyms {
        synonyms::draw_synonym_popup(frame, app);
    }

    if app.show_help {
        draw_help_popup(frame);
    }
}

fn draw_search_bar(frame: &mut Frame, app: &App, area: Rect) {
    use crate::search::SearchMode;

    let mode_label = match app.search.mode {
        SearchMode::Find => "Find",
        SearchMode::Replace => "Replace",
    };

    let case_indicator = if app.search.case_sensitive { "[Aa]" } else { "[aa]" };
    let match_count = app.search.matches.len();
    let current = if match_count > 0 { app.search.current_match + 1 } else { 0 };

    let search_text = if app.search.mode == SearchMode::Replace && !app.search.replace_text.is_empty() {
        format!(
            " {}: {} -> {} {} ({}/{})",
            mode_label, app.search.query, app.search.replace_text, case_indicator, current, match_count
        )
    } else {
        format!(
            " {}: {} {} ({}/{})",
            mode_label, app.search.query, case_indicator, current, match_count
        )
    };

    let style = if match_count > 0 {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else if !app.search.query.is_empty() {
        Style::default().fg(Color::White).bg(Color::Red)
    } else {
        Style::default().fg(Color::White).bg(Color::Blue)
    };

    let search_para = Paragraph::new(search_text).style(style);
    frame.render_widget(search_para, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (x, y) = app.buffer.cursor();

    // Build mode indicator
    let mode = if app.show_synonyms {
        "SYNONYM"
    } else if app.command_preview.is_some() {
        "COMMAND"
    } else if app.emoji_mode {
        "EMOJI"
    } else {
        "INSERT"
    };

    // AI status with provider, mode, and model info
    let ai_status = if app.api_loading {
        format!("{} \u{23F3}", app.config.ai_mode)
    } else {
        format!("{} | {} [{}]",
            app.config.ai_mode,
            app.config.ai_provider,
            app.config.current_model_display()
        )
    };

    // Word and character count
    let word_count = app.word_count();
    let char_count = app.char_count();
    let line_count = app.line_count();

    // Build status line
    let left_status = format!(
        " {} | Ln {}, Col {} | {} words, {} chars, {} lines ",
        mode,
        y + 1,
        x + 1,
        word_count,
        char_count,
        line_count,
    );

    // Build right status (AI status and shortcuts)
    let right_status = format!("{} | F1:Help | Ctrl+Q:Quit ", ai_status);

    // Show status message if present
    let status_line = if let Some(msg) = &app.status_message {
        format!("{} | {} | {}", left_status, msg, right_status)
    } else {
        format!("{} | {}", left_status, right_status)
    };

    // Create two lines for the status bar
    let line1 = Line::from(vec![
        Span::styled(status_line, Style::default().fg(Color::White)),
    ]);

    // Second line shows quick help hints
    let hints = if app.search.is_active {
        " Enter:Next | Shift+Enter:Prev | Ctrl+Enter:Replace | Esc:Close "
    } else if app.emoji_mode {
        " Tab:Accept | Ctrl+E:Exit Emoji | Ctrl+G:AI | Ctrl+Space:Sentence "
    } else if app.api_suggestion.is_some() {
        " AI ready - Tab:Accept | Ctrl+Space:Sentence | Ctrl+P:Provider | Ctrl+M:Model "
    } else {
        " Tab:Accept | Ctrl+F:Find | Ctrl+G:AI | Ctrl+P:Provider | Ctrl+M:Model | Ctrl+N:Mode "
    };

    let line2 = Line::from(vec![
        Span::styled(hints, Style::default().fg(Color::Gray)),
    ]);

    let status_paragraph = Paragraph::new(vec![line1, line2])
        .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(status_paragraph, area);
}

fn draw_help_popup(frame: &mut Frame) {
    let area = centered_rect(70, 85, frame.area());

    frame.render_widget(Clear, area);

    let help_text = r#"
  SYSTYMN - AI-Powered Text Editor
  =================================

  NAVIGATION
    Arrow keys       Move cursor
    Home/End         Line start/end
    Ctrl+Home/End    Document start/end
    PageUp/Down      Scroll page

  SELECTION & CLIPBOARD
    Shift+Arrows     Select text
    Ctrl+A           Select all
    Ctrl+C           Copy
    Ctrl+X           Cut
    Ctrl+V           Paste

  EDITING
    Ctrl+Z           Undo
    Ctrl+Y           Redo

  SEARCH & REPLACE
    Ctrl+F           Find
    Ctrl+H           Find & Replace
    Enter/F3         Next match
    Shift+Enter      Previous match
    Ctrl+Enter       Replace current
    Ctrl+I           Toggle case sensitivity

  AI SUGGESTIONS
    Tab              Accept word suggestion
    Ctrl+Space       Accept sentence suggestion
    Ctrl+G           Fetch AI suggestion
    Ctrl+P           Cycle AI provider (Local/OpenAI/Anthropic)
    Ctrl+M           Cycle AI model
    Ctrl+N           Cycle AI mode (Off/Local/API/Hybrid)
    Ctrl+T           Toggle auto-suggestions

  SYNONYMS
    Ctrl+S           Open synonym selector
    Ctrl+Up/Down     Navigate synonyms
    Enter            Select synonym

  EMOJI MODE
    Ctrl+E           Toggle emoji mode
    /emoji [cat]     Insert emoji (face, nature, animal, heart, food)

  GENERATOR COMMANDS
    /n               Random name (/nm male, /nf female, /nx neutral)
    /c               Random city
    /l               Location (/li interior, /lo outdoor)
    /e               Emotion (/e+ positive, /e- negative)
    /o               Random object
    /t               Time (/td day, /tn night)
    /a               Action verb

  FILES
    Ctrl+O           Open file
    Ctrl+W           Save file
    Ctrl+Q           Quit

  Set OPENAI_API_KEY and/or ANTHROPIC_API_KEY for cloud AI

  Press any key to close
"#;

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(" Help (F1) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black))
        );

    frame.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
