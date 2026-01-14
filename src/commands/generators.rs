use rand::prelude::*;
use super::parser::{Command, NameVariant, LocationVariant, EmotionVariant, TimeVariant};

const NAMES_MALE: &str = include_str!("../../data/names_male.txt");
const NAMES_FEMALE: &str = include_str!("../../data/names_female.txt");
const NAMES_NEUTRAL: &str = include_str!("../../data/names_neutral.txt");
const CITIES: &str = include_str!("../../data/cities.txt");
const LOCATIONS_INTERIOR: &str = include_str!("../../data/locations_interior.txt");
const LOCATIONS_OUTDOOR: &str = include_str!("../../data/locations_outdoor.txt");
const EMOTIONS_POSITIVE: &str = include_str!("../../data/emotions_positive.txt");
const EMOTIONS_NEGATIVE: &str = include_str!("../../data/emotions_negative.txt");
const OBJECTS: &str = include_str!("../../data/objects.txt");
const TIMES_DAY: &str = include_str!("../../data/times_day.txt");
const TIMES_NIGHT: &str = include_str!("../../data/times_night.txt");
const ACTIONS: &str = include_str!("../../data/actions.txt");

pub struct Generators;

impl Generators {
    pub fn generate(command: &Command) -> String {
        let mut rng = thread_rng();

        match command {
            Command::Name(variant) => {
                let pool = match variant {
                    NameVariant::Male => NAMES_MALE,
                    NameVariant::Female => NAMES_FEMALE,
                    NameVariant::Neutral => NAMES_NEUTRAL,
                    NameVariant::Any => {
                        match rng.gen_range(0..3) {
                            0 => NAMES_MALE,
                            1 => NAMES_FEMALE,
                            _ => NAMES_NEUTRAL,
                        }
                    }
                };
                Self::random_line(pool, &mut rng)
            }

            Command::City => Self::random_line(CITIES, &mut rng),

            Command::Location(variant) => {
                let pool = match variant {
                    LocationVariant::Interior => LOCATIONS_INTERIOR,
                    LocationVariant::Outdoor => LOCATIONS_OUTDOOR,
                    LocationVariant::Any => {
                        if rng.gen_bool(0.5) { LOCATIONS_INTERIOR } else { LOCATIONS_OUTDOOR }
                    }
                };
                Self::random_line(pool, &mut rng)
            }

            Command::Emotion(variant) => {
                let pool = match variant {
                    EmotionVariant::Positive => EMOTIONS_POSITIVE,
                    EmotionVariant::Negative => EMOTIONS_NEGATIVE,
                    EmotionVariant::Any => {
                        if rng.gen_bool(0.5) { EMOTIONS_POSITIVE } else { EMOTIONS_NEGATIVE }
                    }
                };
                Self::random_line(pool, &mut rng)
            }

            Command::Object => Self::random_line(OBJECTS, &mut rng),

            Command::Time(variant) => {
                let pool = match variant {
                    TimeVariant::Day => TIMES_DAY,
                    TimeVariant::Night => TIMES_NIGHT,
                    TimeVariant::Any => {
                        if rng.gen_bool(0.5) { TIMES_DAY } else { TIMES_NIGHT }
                    }
                };
                Self::random_line(pool, &mut rng)
            }

            Command::Action => Self::random_line(ACTIONS, &mut rng),

            // Emoji is handled separately in app.rs via EmojiEngine
            Command::Emoji(_) => "\u{2728}".to_string(),

            Command::Unknown(_) => "???".to_string(),
        }
    }

    fn random_line(data: &str, rng: &mut ThreadRng) -> String {
        let lines: Vec<&str> = data.lines().filter(|l| !l.is_empty()).collect();
        lines.choose(rng).map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string())
    }

    pub fn preview(command: &Command) -> String {
        match command {
            Command::Name(NameVariant::Male) => "[male name]".to_string(),
            Command::Name(NameVariant::Female) => "[female name]".to_string(),
            Command::Name(NameVariant::Neutral) => "[neutral name]".to_string(),
            Command::Name(NameVariant::Any) => "[name]".to_string(),
            Command::City => "[city]".to_string(),
            Command::Location(LocationVariant::Interior) => "[interior]".to_string(),
            Command::Location(LocationVariant::Outdoor) => "[outdoor]".to_string(),
            Command::Location(LocationVariant::Any) => "[location]".to_string(),
            Command::Emotion(EmotionVariant::Positive) => "[positive emotion]".to_string(),
            Command::Emotion(EmotionVariant::Negative) => "[negative emotion]".to_string(),
            Command::Emotion(EmotionVariant::Any) => "[emotion]".to_string(),
            Command::Object => "[object]".to_string(),
            Command::Time(TimeVariant::Day) => "[daytime]".to_string(),
            Command::Time(TimeVariant::Night) => "[nighttime]".to_string(),
            Command::Time(TimeVariant::Any) => "[time]".to_string(),
            Command::Action => "[action]".to_string(),
            Command::Emoji(Some(cat)) => format!("[emoji: {}]", cat),
            Command::Emoji(None) => "[emoji]".to_string(),
            Command::Unknown(s) => format!("[unknown: {}]", s),
        }
    }
}
