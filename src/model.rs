use std::str::FromStr;
use serde::Deserialize;

#[derive(Deserialize)]
/// model for entry returned from HLTB API
pub struct RawEntry {
    pub game_image: String,
    pub game_name: String,
    /// Main + Extra
    pub comp_plus: i32,
    /// Main Story
    pub comp_main: i32,
    /// Completionist
    pub comp_100: i32,
    pub game_id: i64,
    pub profile_steam: i64
}

#[derive(Debug)]
/// model for mapped entry
pub struct Entry {
    pub name: String,
    pub link: String,
    pub img: String,
    pub descr: String,
    pub steam: Option<String>
}

impl Entry {
    pub fn new(name: String, link: String, img: String, descr: String, steam: Option<String>) -> Entry {
        Entry {
            name,
            img,
            link,
            descr,
            steam
        }
    }
}

#[derive(Debug)]
pub enum RunMode {
    Polling,
    WebHook
}
#[derive(Debug)]
pub struct EnumParseError(String);

impl FromStr for RunMode {
    type Err = EnumParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "polling" => Ok(RunMode::Polling),
            "webhook" => Ok(RunMode::WebHook),
            _ => Err(EnumParseError(format!("Unknown RunMode {}", s)))
        }
    }
}