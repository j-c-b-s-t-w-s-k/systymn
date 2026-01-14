// Ghost text suggestions are rendered inline in editor.rs
// This module contains helper functions for suggestion display

use crate::ai::Suggestion;

pub fn format_suggestion(suggestion: &Suggestion) -> String {
    suggestion.text.clone()
}

pub fn suggestion_color_intensity(pulse_phase: f32) -> f32 {
    // Sinusoidal pulse between 0.3 and 0.7
    (pulse_phase * std::f32::consts::PI * 2.0).sin() * 0.2 + 0.5
}
