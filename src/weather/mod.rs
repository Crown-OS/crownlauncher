mod art;
mod condition;
mod fetch;

pub use art::svg;
pub use condition::Condition;
pub use fetch::forecasts;

/// One city's weather right now.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub city: String,
    pub temperature: f32,
    pub high: f32,
    pub low: f32,
    pub condition: Condition,
    pub night: bool,
}

impl Report {
    /// The temperature without its degree sign, which the tile sets smaller, and today's high
    /// over low.
    pub fn degrees(&self) -> (String, String) {
        let rounded = |value: f32| value.round() as i32;
        (
            rounded(self.temperature).to_string(),
            format!("{}°/{}°", rounded(self.high), rounded(self.low)),
        )
    }
}
