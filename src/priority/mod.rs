use serde::{Deserialize, Serialize};
use ratatui::style::Color;

#[derive(Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    pub fn get_color(&self) -> Color {
        match self {
            Priority::Low => Color::Rgb(9, 245, 33),
            Priority::Medium => Color::Rgb(245, 151, 9),
            Priority::High => Color::Rgb(245, 9, 9),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Priority::Low => Priority::Medium,
            Priority::Medium => Priority::High,
            Priority::High => Priority::Low,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
        }
    }
}
