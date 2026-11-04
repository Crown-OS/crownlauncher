//! One Open-Meteo GET per city. Open-Meteo needs no key and is sent nothing but the
//! coordinates.

use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

use super::{Condition, Report};
use crate::settings::{City, Units};

const FORECAST: &str = "https://api.open-meteo.com/v1/forecast";
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Deserialize)]
struct Response {
    current: Current,
    daily: Daily,
}

#[derive(Deserialize)]
struct Current {
    temperature_2m: f32,
    weather_code: u8,
    is_day: u8,
}

#[derive(Deserialize)]
struct Daily {
    temperature_2m_max: Vec<f32>,
    temperature_2m_min: Vec<f32>,
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .user_agent(concat!("crownlauncher/", env!("CARGO_PKG_VERSION")))
        .build()
        .new_agent()
}

/// The forecast of every city, skipping the ones that fail. Blocks; run it off the UI thread.
pub fn forecasts(cities: &[City], units: Units) -> Vec<Report> {
    let agent = agent();
    cities
        .iter()
        .filter_map(|city| {
            forecast(&agent, city, units)
                .inspect_err(|error| log::warn!("weather for {}: {error:#}", city.name))
                .ok()
        })
        .collect()
}

fn forecast(agent: &ureq::Agent, city: &City, units: Units) -> Result<Report> {
    let unit = match units {
        Units::Celsius => "celsius",
        Units::Fahrenheit => "fahrenheit",
    };
    let body = agent
        .get(FORECAST)
        .query("latitude", city.latitude.to_string())
        .query("longitude", city.longitude.to_string())
        .query("current", "temperature_2m,weather_code,is_day")
        .query("daily", "temperature_2m_max,temperature_2m_min")
        .query("timezone", "auto")
        .query("forecast_days", "1")
        .query("temperature_unit", unit)
        .call()?
        .body_mut()
        .read_to_string()?;
    parse(&city.name, &body)
}

fn parse(city: &str, body: &str) -> Result<Report> {
    let response: Response = serde_json::from_str(body).context("parse the forecast")?;
    let first = |values: &[f32]| values.first().copied().context("no daily forecast");
    Ok(Report {
        city: city.to_owned(),
        temperature: response.current.temperature_2m,
        high: first(&response.daily.temperature_2m_max)?,
        low: first(&response.daily.temperature_2m_min)?,
        condition: Condition::from_wmo(response.current.weather_code),
        night: response.current.is_day == 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_open_meteo_answer_becomes_a_report() {
        let body = r#"{"current":{"time":"2026-05-24T12:00","temperature_2m":18.4,"weather_code":2,"is_day":1},
            "daily":{"time":["2026-05-24"],"temperature_2m_max":[26.1],"temperature_2m_min":[3.2]}}"#;
        let report = match parse("Munich", body) {
            Ok(report) => report,
            Err(error) => panic!("{error:#}"),
        };
        assert_eq!(report.condition, Condition::PartlyCloudy);
        assert!(!report.night);
        assert_eq!(report.degrees(), ("18".to_owned(), "26°/3°".to_owned()));
    }

    #[test]
    fn an_answer_without_a_day_is_an_error() {
        let body = r#"{"current":{"temperature_2m":1.0,"weather_code":0,"is_day":0},
            "daily":{"temperature_2m_max":[],"temperature_2m_min":[]}}"#;
        assert!(parse("Nowhere", body).is_err());
    }
}
