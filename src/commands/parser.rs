#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Name(NameVariant),
    City,
    Location(LocationVariant),
    Emotion(EmotionVariant),
    Object,
    Time(TimeVariant),
    Action,
    Emoji(Option<String>), // Optional category
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NameVariant {
    Any,
    Male,
    Female,
    Neutral,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocationVariant {
    Any,
    Interior,
    Outdoor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EmotionVariant {
    Any,
    Positive,
    Negative,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeVariant {
    Any,
    Day,
    Night,
}

pub struct CommandParser;

impl CommandParser {
    pub fn parse(input: &str) -> Option<(Command, usize, usize)> {
        let trimmed = input.trim_end();

        // Find the last slash command in the text
        if let Some(slash_pos) = trimmed.rfind('/') {
            let after_slash = &trimmed[slash_pos + 1..];

            // Check if there's a space after the command (command is complete)
            let cmd_end = after_slash.find(' ').unwrap_or(after_slash.len());
            let cmd_str = &after_slash[..cmd_end];

            let command = Self::parse_command(cmd_str)?;
            let start = slash_pos;
            let end = slash_pos + 1 + cmd_end;

            Some((command, start, end))
        } else {
            None
        }
    }

    fn parse_command(s: &str) -> Option<Command> {
        let s = s.to_lowercase();
        match s.as_str() {
            // Names
            "n" => Some(Command::Name(NameVariant::Any)),
            "nm" => Some(Command::Name(NameVariant::Male)),
            "nf" => Some(Command::Name(NameVariant::Female)),
            "nx" => Some(Command::Name(NameVariant::Neutral)),

            // City
            "c" => Some(Command::City),

            // Location
            "l" => Some(Command::Location(LocationVariant::Any)),
            "li" => Some(Command::Location(LocationVariant::Interior)),
            "lo" => Some(Command::Location(LocationVariant::Outdoor)),

            // Emotion
            "e" => Some(Command::Emotion(EmotionVariant::Any)),
            "e+" => Some(Command::Emotion(EmotionVariant::Positive)),
            "e-" => Some(Command::Emotion(EmotionVariant::Negative)),

            // Object
            "o" => Some(Command::Object),

            // Time
            "t" => Some(Command::Time(TimeVariant::Any)),
            "td" => Some(Command::Time(TimeVariant::Day)),
            "tn" => Some(Command::Time(TimeVariant::Night)),

            // Action
            "a" => Some(Command::Action),

            // Emoji
            "emoji" => Some(Command::Emoji(None)),

            _ => {
                // Handle emoji with category (e.g., "emoji face", "emoji nature")
                if s.starts_with("emoji ") {
                    let category = s.strip_prefix("emoji ").map(String::from);
                    Some(Command::Emoji(category))
                } else if !s.is_empty() {
                    Some(Command::Unknown(s))
                } else {
                    None
                }
            }
        }
    }

    pub fn is_partial_command(input: &str) -> Option<&str> {
        let trimmed = input.trim_end();
        if let Some(slash_pos) = trimmed.rfind('/') {
            let after = &trimmed[slash_pos + 1..];
            if !after.contains(' ') && after.len() <= 3 {
                return Some(after);
            }
        }
        None
    }
}
