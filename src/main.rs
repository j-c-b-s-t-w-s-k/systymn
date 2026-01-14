mod app;
mod buffer;
mod ui;
mod ai;
mod commands;
mod config;
mod emoji;
mod search;

use std::io;
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use ratatui::Terminal;
use tokio::sync::mpsc;

use app::App;

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (api_tx, api_rx) = mpsc::channel(10);
    let app = App::new(api_tx);
    let res = run_app(&mut terminal, app, api_rx).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut api_rx: mpsc::Receiver<ai::ApiResponse>,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(50);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Check for API responses (non-blocking)
        while let Ok(response) = api_rx.try_recv() {
            app.handle_api_response(response);
        }

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Handle search mode separately
                if app.search.is_active {
                    match (key.modifiers, key.code) {
                        (KeyModifiers::CONTROL, KeyCode::Char('q')) => return Ok(()),
                        (_, KeyCode::Esc) => app.close_search(),
                        (_, KeyCode::Backspace) => app.search_backspace(),
                        (KeyModifiers::CONTROL, KeyCode::Char('i')) => app.toggle_search_case(),
                        (KeyModifiers::SHIFT, KeyCode::F(3)) => app.search_prev(),
                        (KeyModifiers::SHIFT, KeyCode::Enter) => app.search_prev(),
                        (KeyModifiers::CONTROL, KeyCode::Enter) => app.replace_current(),
                        (_, KeyCode::F(3)) => app.search_next(),
                        (_, KeyCode::Enter) => app.search_next(),
                        (_, KeyCode::Char(c)) => app.search_add_char(c),
                        _ => {}
                    }
                } else {
                    match (key.modifiers, key.code) {
                        (KeyModifiers::CONTROL, KeyCode::Char('q')) => return Ok(()),
                        (KeyModifiers::CONTROL, KeyCode::Char('s')) => app.toggle_synonym_selector(),
                        (KeyModifiers::CONTROL, KeyCode::Up) => app.synonym_up(),
                        (KeyModifiers::CONTROL, KeyCode::Down) => app.synonym_down(),
                        (KeyModifiers::CONTROL, KeyCode::Char(' ')) => app.accept_sentence_suggestion(),
                        (KeyModifiers::CONTROL, KeyCode::Char('o')) => app.open_file_dialog(),
                        (KeyModifiers::CONTROL, KeyCode::Char('w')) => app.save_file(),
                        (KeyModifiers::CONTROL, KeyCode::Char('e')) => app.toggle_emoji_mode(),
                        (KeyModifiers::CONTROL, KeyCode::Char('g')) => app.fetch_api_suggestion(),
                        (KeyModifiers::CONTROL, KeyCode::Char('p')) => app.cycle_ai_provider(),
                        (KeyModifiers::CONTROL, KeyCode::Char('m')) => app.cycle_ai_model(),
                        (KeyModifiers::CONTROL, KeyCode::Char('n')) => app.cycle_ai_mode(),
                        (KeyModifiers::CONTROL, KeyCode::Char('t')) => app.toggle_auto_suggest(),
                        (KeyModifiers::CONTROL, KeyCode::Char('z')) => app.undo(),
                        (KeyModifiers::CONTROL, KeyCode::Char('y')) => app.redo(),
                        (KeyModifiers::CONTROL, KeyCode::Char('c')) => app.copy(),
                        (KeyModifiers::CONTROL, KeyCode::Char('x')) => app.cut(),
                        (KeyModifiers::CONTROL, KeyCode::Char('v')) => app.paste(),
                        (KeyModifiers::CONTROL, KeyCode::Char('a')) => app.select_all(),
                        (KeyModifiers::CONTROL, KeyCode::Char('f')) => app.open_search(),
                        (KeyModifiers::CONTROL, KeyCode::Char('h')) => app.open_replace(),
                        (_, KeyCode::F(1)) => app.toggle_help(),
                        (_, KeyCode::F(3)) => app.search_next(),
                        (_, KeyCode::Tab) => app.accept_suggestion(),
                        (_, KeyCode::Esc) => app.dismiss_or_exit(),
                        (_, KeyCode::Enter) => app.handle_enter(),
                        (_, KeyCode::Backspace) => app.handle_backspace(),
                        (_, KeyCode::Delete) => app.handle_delete(),
                        (KeyModifiers::SHIFT, KeyCode::Left) => app.select_left(),
                        (KeyModifiers::SHIFT, KeyCode::Right) => app.select_right(),
                        (KeyModifiers::SHIFT, KeyCode::Up) => app.select_up(),
                        (KeyModifiers::SHIFT, KeyCode::Down) => app.select_down(),
                        (_, KeyCode::Left) => app.move_cursor_left(),
                        (_, KeyCode::Right) => app.move_cursor_right(),
                        (_, KeyCode::Up) => app.move_cursor_up(),
                        (_, KeyCode::Down) => app.move_cursor_down(),
                        (KeyModifiers::CONTROL, KeyCode::Home) => app.move_to_start(),
                        (KeyModifiers::CONTROL, KeyCode::End) => app.move_to_end(),
                        (_, KeyCode::Home) => app.move_cursor_home(),
                        (_, KeyCode::End) => app.move_cursor_end(),
                        (_, KeyCode::PageUp) => app.page_up(),
                        (_, KeyCode::PageDown) => app.page_down(),
                        (_, KeyCode::Char(c)) => app.insert_char(c),
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
        }
    }
}
