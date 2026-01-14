use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState};

use crate::app::App;

pub fn draw_synonym_popup(frame: &mut Frame, app: &mut App) {
    if app.synonyms.is_empty() {
        return;
    }

    let area = centered_rect(30, 40, frame.area());
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .synonyms
        .iter()
        .enumerate()
        .map(|(i, syn)| {
            let style = if i == app.synonym_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!("  {}  ", syn)).style(style)
        })
        .collect();

    let word = app.synonym_word.as_deref().unwrap_or("word");
    let title = format!(" Synonyms: {} ", word);

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::DarkGray))
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut state = ListState::default();
    state.select(Some(app.synonym_index));

    frame.render_stateful_widget(list, area, &mut state);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

// Simple thesaurus - embedded synonyms for common words
pub fn get_synonyms(word: &str) -> Vec<String> {
    let word_lower = word.to_lowercase();
    let synonyms: &[&str] = match word_lower.as_str() {
        "good" => &["excellent", "fine", "great", "superb", "wonderful", "pleasant"],
        "bad" => &["terrible", "awful", "poor", "dreadful", "horrible", "dire"],
        "big" => &["large", "huge", "enormous", "vast", "immense", "massive"],
        "small" => &["tiny", "little", "minute", "miniature", "compact", "petite"],
        "happy" => &["joyful", "cheerful", "content", "elated", "delighted", "pleased"],
        "sad" => &["unhappy", "sorrowful", "melancholy", "dejected", "gloomy", "mournful"],
        "fast" => &["quick", "rapid", "swift", "speedy", "hasty", "fleet"],
        "slow" => &["sluggish", "unhurried", "leisurely", "gradual", "plodding", "languid"],
        "beautiful" => &["gorgeous", "stunning", "lovely", "exquisite", "radiant", "elegant"],
        "ugly" => &["hideous", "unsightly", "grotesque", "repulsive", "ghastly", "dreadful"],
        "old" => &["ancient", "aged", "elderly", "antique", "vintage", "venerable"],
        "new" => &["fresh", "recent", "modern", "novel", "current", "contemporary"],
        "dark" => &["dim", "shadowy", "murky", "gloomy", "dusky", "obscure"],
        "light" => &["bright", "luminous", "radiant", "brilliant", "gleaming", "glowing"],
        "cold" => &["chilly", "frigid", "icy", "frosty", "frozen", "gelid"],
        "hot" => &["warm", "scorching", "blazing", "sweltering", "boiling", "searing"],
        "walk" => &["stroll", "amble", "saunter", "wander", "trudge", "stride"],
        "run" => &["sprint", "dash", "race", "bolt", "hurry", "rush"],
        "say" => &["speak", "utter", "declare", "state", "pronounce", "articulate"],
        "look" => &["gaze", "stare", "glance", "peer", "observe", "watch"],
        "think" => &["ponder", "consider", "contemplate", "reflect", "muse", "deliberate"],
        "feel" => &["sense", "perceive", "experience", "detect", "notice", "touch"],
        "make" => &["create", "produce", "construct", "build", "form", "fashion"],
        "take" => &["grab", "seize", "grasp", "snatch", "capture", "acquire"],
        "come" => &["arrive", "approach", "reach", "appear", "emerge", "materialize"],
        "go" => &["leave", "depart", "proceed", "advance", "travel", "journey"],
        "see" => &["observe", "witness", "perceive", "notice", "view", "behold"],
        "know" => &["understand", "comprehend", "realize", "recognize", "grasp", "fathom"],
        "want" => &["desire", "wish", "crave", "yearn", "long", "covet"],
        "love" => &["adore", "cherish", "treasure", "worship", "idolize", "revere"],
        "hate" => &["despise", "loathe", "detest", "abhor", "scorn", "disdain"],
        "fear" => &["dread", "terror", "fright", "horror", "panic", "alarm"],
        "house" => &["home", "dwelling", "residence", "abode", "domicile", "manor"],
        "door" => &["entrance", "portal", "gateway", "threshold", "aperture", "opening"],
        "room" => &["chamber", "space", "quarters", "compartment", "cell", "enclosure"],
        "night" => &["evening", "darkness", "nightfall", "dusk", "twilight", "midnight"],
        "day" => &["daylight", "daytime", "morning", "noon", "afternoon", "dawn"],
        "time" => &["moment", "instant", "period", "era", "epoch", "duration"],
        "place" => &["location", "spot", "site", "area", "position", "locale"],
        "world" => &["earth", "globe", "planet", "realm", "domain", "universe"],
        "man" => &["person", "individual", "human", "gentleman", "fellow", "figure"],
        "woman" => &["lady", "female", "person", "individual", "dame", "maiden"],
        "child" => &["kid", "youngster", "youth", "minor", "juvenile", "offspring"],
        "water" => &["liquid", "fluid", "aqua", "moisture", "H2O", "stream"],
        "fire" => &["flame", "blaze", "inferno", "conflagration", "combustion", "ember"],
        "earth" => &["ground", "soil", "land", "terrain", "dirt", "clay"],
        "air" => &["atmosphere", "breeze", "wind", "oxygen", "sky", "ether"],
        "strange" => &["peculiar", "odd", "bizarre", "unusual", "curious", "weird"],
        "quiet" => &["silent", "hushed", "still", "peaceful", "calm", "serene"],
        "loud" => &["noisy", "thunderous", "deafening", "booming", "clamorous", "raucous"],
        "empty" => &["vacant", "hollow", "void", "bare", "blank", "desolate"],
        "full" => &["complete", "filled", "packed", "stuffed", "loaded", "brimming"],
        _ => return vec![word.to_string()],
    };
    synonyms.iter().map(|s| s.to_string()).collect()
}
