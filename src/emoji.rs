use rand::prelude::*;
use std::collections::HashMap;

pub struct EmojiEngine {
    categories: HashMap<String, Vec<&'static str>>,
    mood_emojis: HashMap<String, Vec<&'static str>>,
}

impl EmojiEngine {
    pub fn new() -> Self {
        let mut categories = HashMap::new();
        let mut mood_emojis = HashMap::new();

        // Faces & expressions
        categories.insert("face".to_string(), vec![
            "\u{1F600}", "\u{1F601}", "\u{1F602}", "\u{1F923}", "\u{1F60A}", "\u{1F60D}",
            "\u{1F929}", "\u{1F60E}", "\u{1F914}", "\u{1F928}", "\u{1F610}", "\u{1F644}",
            "\u{1F62D}", "\u{1F622}", "\u{1F625}", "\u{1F624}", "\u{1F621}", "\u{1F631}",
        ]);

        // Nature
        categories.insert("nature".to_string(), vec![
            "\u{1F331}", "\u{1F332}", "\u{1F333}", "\u{1F334}", "\u{1F335}", "\u{1F33B}",
            "\u{1F33A}", "\u{1F339}", "\u{1F337}", "\u{1F338}", "\u{1F340}", "\u{1F341}",
            "\u{1F342}", "\u{1F343}", "\u{1F308}", "\u{2600}", "\u{1F319}", "\u{2B50}",
        ]);

        // Animals
        categories.insert("animal".to_string(), vec![
            "\u{1F436}", "\u{1F431}", "\u{1F42D}", "\u{1F430}", "\u{1F98A}", "\u{1F43B}",
            "\u{1F43C}", "\u{1F428}", "\u{1F981}", "\u{1F42F}", "\u{1F984}", "\u{1F409}",
            "\u{1F40D}", "\u{1F422}", "\u{1F420}", "\u{1F433}", "\u{1F419}", "\u{1F98B}",
        ]);

        // Objects
        categories.insert("object".to_string(), vec![
            "\u{1F4D6}", "\u{270F}", "\u{1F58B}", "\u{1F5A5}", "\u{1F4F1}", "\u{1F4A1}",
            "\u{1F50E}", "\u{1F511}", "\u{1F512}", "\u{2699}", "\u{1F4E6}", "\u{1F381}",
            "\u{1F3B5}", "\u{1F3A8}", "\u{1F3AD}", "\u{1F3AE}", "\u{1F3B2}", "\u{2615}",
        ]);

        // Weather
        categories.insert("weather".to_string(), vec![
            "\u{2600}", "\u{1F324}", "\u{26C5}", "\u{1F325}", "\u{1F326}", "\u{1F327}",
            "\u{26C8}", "\u{1F329}", "\u{1F328}", "\u{1F32A}", "\u{1F32C}", "\u{1F308}",
        ]);

        // Food
        categories.insert("food".to_string(), vec![
            "\u{1F34E}", "\u{1F34F}", "\u{1F34A}", "\u{1F34B}", "\u{1F34C}", "\u{1F347}",
            "\u{1F353}", "\u{1F350}", "\u{1F351}", "\u{1F352}", "\u{1F95D}", "\u{1F345}",
            "\u{1F355}", "\u{1F354}", "\u{1F35F}", "\u{1F32E}", "\u{1F370}", "\u{1F382}",
        ]);

        // Hearts & symbols
        categories.insert("heart".to_string(), vec![
            "\u{2764}", "\u{1F9E1}", "\u{1F49B}", "\u{1F49A}", "\u{1F499}", "\u{1F49C}",
            "\u{1F5A4}", "\u{1F90D}", "\u{1F90E}", "\u{1F495}", "\u{1F496}", "\u{1F497}",
            "\u{1F498}", "\u{1F49D}", "\u{1F49E}", "\u{1F49F}", "\u{2763}", "\u{1F48B}",
        ]);

        // Actions/gestures
        categories.insert("gesture".to_string(), vec![
            "\u{1F44D}", "\u{1F44E}", "\u{1F44F}", "\u{1F64C}", "\u{1F44B}", "\u{270B}",
            "\u{1F91A}", "\u{1F590}", "\u{1F596}", "\u{1F918}", "\u{1F919}", "\u{1F91F}",
            "\u{270C}", "\u{1F91E}", "\u{1F90C}", "\u{1F448}", "\u{1F449}", "\u{261D}",
        ]);

        // Sparkles & magic
        categories.insert("magic".to_string(), vec![
            "\u{2728}", "\u{1F4AB}", "\u{1FA84}", "\u{1F52E}", "\u{1F9FF}", "\u{1F320}",
            "\u{1F387}", "\u{1F386}", "\u{1F9E8}", "\u{1F3C6}", "\u{1F451}", "\u{1F48E}",
        ]);

        // Mood-based suggestions
        mood_emojis.insert("happy".to_string(), vec![
            "\u{1F600}", "\u{1F60A}", "\u{1F604}", "\u{1F389}", "\u{2728}", "\u{1F31F}",
        ]);
        mood_emojis.insert("sad".to_string(), vec![
            "\u{1F622}", "\u{1F62D}", "\u{1F614}", "\u{1F625}", "\u{1F4A7}", "\u{1F327}",
        ]);
        mood_emojis.insert("love".to_string(), vec![
            "\u{2764}", "\u{1F60D}", "\u{1F970}", "\u{1F495}", "\u{1F48B}", "\u{1F49D}",
        ]);
        mood_emojis.insert("angry".to_string(), vec![
            "\u{1F621}", "\u{1F624}", "\u{1F620}", "\u{1F4A2}", "\u{1F525}", "\u{26A1}",
        ]);
        mood_emojis.insert("think".to_string(), vec![
            "\u{1F914}", "\u{1F4AD}", "\u{1F9E0}", "\u{1F4A1}", "\u{2753}", "\u{1F50D}",
        ]);
        mood_emojis.insert("cool".to_string(), vec![
            "\u{1F60E}", "\u{1F929}", "\u{1F525}", "\u{1F4AF}", "\u{1F3C6}", "\u{2728}",
        ]);

        Self { categories, mood_emojis }
    }

    pub fn random_emoji(&self) -> &str {
        let mut rng = thread_rng();
        let all_categories: Vec<_> = self.categories.values().collect();
        if let Some(category) = all_categories.choose(&mut rng) {
            if let Some(emoji) = category.choose(&mut rng) {
                return emoji;
            }
        }
        "\u{2728}" // sparkles as fallback
    }

    pub fn emoji_by_category(&self, category: &str) -> Option<&str> {
        let mut rng = thread_rng();
        self.categories
            .get(category)
            .and_then(|emojis| emojis.choose(&mut rng).copied())
    }

    pub fn emoji_for_mood(&self, mood: &str) -> Option<&str> {
        let mut rng = thread_rng();
        self.mood_emojis
            .get(mood)
            .and_then(|emojis| emojis.choose(&mut rng).copied())
    }

    pub fn suggest_emoji(&self, text: &str) -> Option<&str> {
        let lower = text.to_lowercase();

        // Detect mood from text
        if lower.contains("happy") || lower.contains("joy") || lower.contains("great") || lower.contains("wonderful") {
            return self.emoji_for_mood("happy");
        }
        if lower.contains("sad") || lower.contains("cry") || lower.contains("tears") || lower.contains("miss") {
            return self.emoji_for_mood("sad");
        }
        if lower.contains("love") || lower.contains("heart") || lower.contains("adore") || lower.contains("dear") {
            return self.emoji_for_mood("love");
        }
        if lower.contains("angry") || lower.contains("hate") || lower.contains("furious") || lower.contains("mad") {
            return self.emoji_for_mood("angry");
        }
        if lower.contains("think") || lower.contains("wonder") || lower.contains("hmm") || lower.contains("question") {
            return self.emoji_for_mood("think");
        }
        if lower.contains("cool") || lower.contains("awesome") || lower.contains("amazing") || lower.contains("epic") {
            return self.emoji_for_mood("cool");
        }

        // Detect categories from text
        if lower.contains("cat") || lower.contains("dog") || lower.contains("animal") || lower.contains("bird") {
            return self.emoji_by_category("animal");
        }
        if lower.contains("tree") || lower.contains("flower") || lower.contains("nature") || lower.contains("forest") {
            return self.emoji_by_category("nature");
        }
        if lower.contains("rain") || lower.contains("sun") || lower.contains("cloud") || lower.contains("weather") {
            return self.emoji_by_category("weather");
        }
        if lower.contains("food") || lower.contains("eat") || lower.contains("hungry") || lower.contains("delicious") {
            return self.emoji_by_category("food");
        }
        if lower.contains("magic") || lower.contains("sparkle") || lower.contains("shine") || lower.contains("star") {
            return self.emoji_by_category("magic");
        }

        None
    }

    pub fn all_categories(&self) -> Vec<&str> {
        vec!["face", "nature", "animal", "object", "weather", "food", "heart", "gesture", "magic"]
    }
}

impl Default for EmojiEngine {
    fn default() -> Self {
        Self::new()
    }
}
