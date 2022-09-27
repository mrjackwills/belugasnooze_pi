use crate::env::AppEnv;

mod font;

use figlet_rs::FIGfont;
use font::{Color, FontName};

const RESET: &str = "\x1b[0m";

/// Convert input string to ASCII art
fn create_art(input: &str, fontname: FontName) -> String {
    if let Ok(font) = FIGfont::from_content(FontName::get(fontname)) {
        let figure = font.convert(input);
        figure.map_or_else(String::new, |text| text.to_string())
    } else {
        String::new()
    }
}

/// Add color to a given string
fn paint_text(text: &str, color: Color) -> String {
    let tint = Color::get(color);
    let painted = text
        .lines()
        .into_iter()
        .map(|i| format!("{tint}{i}\n"))
        .collect::<String>();
    format!("{}{}", painted, RESET)
}

/// Show the intro texts
fn display_intro(app_envs: &AppEnv) -> String {
    let beluga = paint_text(
        &create_art("Belugasnooze", FontName::Colossal),
        Color::Magenta,
    );
    let client = paint_text(&create_art("pi_client", FontName::Colossal), Color::Yellow);

    let author = env!("CARGO_PKG_AUTHORS");
    let semver = env!("CARGO_PKG_VERSION");
    let version = paint_text(&format!("v{semver}    {author}"), Color::Green);

    let mut output = format!("{beluga}{client}{version}");
    if app_envs.trace {
        output.push('\n');
        let debug = paint_text("!! TRACE MODE !!", Color::BgRed);
        for _ in 0..=2 {
            output.push_str(&debug);
        }
    } else if app_envs.debug {
        output.push('\n');
        let debug = paint_text("!! DEBUG MODE !!", Color::BgRed);
        for _ in 0..=2 {
            output.push_str(&debug);
        }
    }
    output
}

pub struct Intro {
    data: String,
}

impl Intro {
    pub fn new(app_envs: &AppEnv) -> Self {
        Self {
            data: display_intro(app_envs),
        }
    }
    pub fn show(self) {
        println!("{}", self.data);
    }
}

/// WordArt tests
///
/// cargo watch -q -c -w src/ -x 'test word_art -- --nocapture'
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::time::SystemTime;
    use time::UtcOffset;

    #[test]
    fn word_art_display_intro_trace() {
        let na = String::from("na");
        let args = AppEnv {
            trace: true,
            location_ip_address: na.clone(),
            location_log_combined: na.clone(),
            location_log_error: na.clone(),
            timezone: na.clone(),
            location_sqlite: na.clone(),
            debug: true,
            start_time: SystemTime::now(),
            utc_offset: UtcOffset::from_hms(0, 0, 0).unwrap(),
            ws_address: na.clone(),
            ws_apikey: na.clone(),
            ws_token_address: na.clone(),
            ws_password: na,
            sql_threads: 2,
        };

        let result = display_intro(&args);
        assert!(result.contains("!! TRACE"));
        assert!(!result.contains("!! DEBUG"));
    }

    #[test]
    fn word_art_display_intro_debug() {
        let na = String::from("na");
        let args = AppEnv {
            trace: false,
            location_ip_address: na.clone(),
            location_log_combined: na.clone(),
            location_log_error: na.clone(),
            timezone: na.clone(),
            location_sqlite: na.clone(),
            debug: true,
            start_time: SystemTime::now(),
            utc_offset: UtcOffset::from_hms(0, 0, 0).unwrap(),
            ws_address: na.clone(),
            ws_apikey: na.clone(),
            ws_token_address: na.clone(),
            ws_password: na,
            sql_threads: 2,
        };

        let result = display_intro(&args);
        assert!(!result.contains("!! TRACE"));
        assert!(result.contains("!! DEBUG"));
    }

    #[test]
    fn word_art_display_intro() {
        let na = String::from("na");
        let args = AppEnv {
            trace: false,
            timezone: na.clone(),
            location_ip_address: na.clone(),
            location_log_combined: na.clone(),
            location_log_error: na.clone(),
            location_sqlite: na.clone(),
            debug: false,
            start_time: SystemTime::now(),
            utc_offset: UtcOffset::from_hms(0, 0, 0).unwrap(),
            ws_address: na.clone(),
            ws_apikey: na.clone(),
            ws_token_address: na.clone(),
            ws_password: na,
            sql_threads: 2,
        };

        let result = display_intro(&args);
        assert!(!result.contains("!! DEBUG"));
        assert!(!result.contains("!! TRACE"));
    }
}
