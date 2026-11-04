/// What the sky is doing, as coarsely as one icon can show it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Condition {
    #[default]
    Clear,
    PartlyCloudy,
    Cloudy,
    Rain,
    Thunder,
    Snow,
    Fog,
}

impl Condition {
    /// WMO 4677 present-weather code, as Open-Meteo reports it.
    pub fn from_wmo(code: u8) -> Self {
        match code {
            0 => Self::Clear,
            1 | 2 => Self::PartlyCloudy,
            45 | 48 => Self::Fog,
            51..=57 | 61..=67 | 80..=82 => Self::Rain,
            71..=77 | 85 | 86 => Self::Snow,
            95..=99 => Self::Thunder,
            _ => Self::Cloudy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wmo_families_collapse_to_one_icon_each() {
        for (code, expected) in [
            (0u8, Condition::Clear),
            (2, Condition::PartlyCloudy),
            (3, Condition::Cloudy),
            (48, Condition::Fog),
            (55, Condition::Rain),
            (81, Condition::Rain),
            (86, Condition::Snow),
            (96, Condition::Thunder),
            (200, Condition::Cloudy),
        ] {
            assert_eq!(Condition::from_wmo(code), expected, "code {code}");
        }
    }
}
