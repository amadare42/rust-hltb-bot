use crate::model::*;

use std::error::Error;
use std::str::FromStr;
use std::usize;
use reqwest::{Client, header};
use reqwest::header::{CONTENT_TYPE};
use serde_json::{json, Value};

/// fetches full entries for specific query
pub async fn fetch_entries(query: &str) -> Result<Vec<Entry>, Box<dyn Error>> {
    let rsp = query_games(&query).await?;
    parse_entries_from_rsp(rsp)
}

pub fn parse_entries_from_rsp(mut rsp: Value) -> Result<Vec<Entry>, Box<dyn Error>> {
    let entries_limit = usize::from_str(&std::env::var("ENTRIES_LIMIT")
        .unwrap_or("5".to_string())
    ).unwrap();

    let data = rsp["data"].take();
    let entries = serde_json::from_value::<Vec<RawEntry>>(data)?;

    Ok(entries
        .into_iter()
        .map(map_entry)
        .take(entries_limit)
        .collect())
}

fn map_entry(raw: RawEntry) -> Entry {
    let name = raw.game_name.to_string();
    let link = std::format!("https://howlongtobeat.com/game/{}", raw.game_id);
    let img = std::format!("https://howlongtobeat.com/games/{}", raw.game_image);
    let steam = if raw.profile_steam == 0 {
        None
    } else {
        Some(format!("https://store.steampowered.com/app/{}/", raw.profile_steam))
    };
    let descr = map_descr(raw);
    Entry::new(
        name,
        link,
        img,
        descr,
        steam,
    )
}

fn map_descr(raw: RawEntry) -> String {
    let main = map_hours(raw.comp_main);
    let plus = map_hours(raw.comp_plus);
    let comp = map_hours(raw.comp_100);
    format!("\
Main Story: {}
Main + Extra: {}
Completionist: {}\n", main, plus, comp)
}

fn map_hours(sec: i32) -> String {
    let hours = sec as f32 / 60.0 / 60.0;
    let integral = f32::trunc(hours);
    let fraction = hours - integral;

    if integral < 1.0 {
        format!("--")
    } else if fraction > 0.5 {
        format!("{}½ Hours", integral)
    } else {
        format!("{} Hours", integral)
    }
}

/// queries HLTB API
pub async fn query_games(query: &str) -> Result<Value, Box<dyn Error>> {
    let parts = query.split_whitespace().collect::<Vec<&str>>();
    let body = json!({
      "searchType": "games",
      "searchTerms": parts,
      "searchPage": 1,
      "size": 20,
      "searchOptions": {
        "filter": "",
        "games": {
          "userId": 0,
          "platform": "",
          "sortCategory": "popular",
          "rangeCategory": "main",
          "rangeTime": { "min": null, "max": null }
        },
        "filter": "",
      },
      "useCache": true
    }).to_string();

    let rq_future = build_client()
        .post("https://howlongtobeat.com/api/search")
        .header(CONTENT_TYPE, "application/json")
        .body(body)
        .send();

    let rsp = rq_future
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;

    Ok(rsp)
}

fn build_client() -> Client {
    let mut headers = header::HeaderMap::new();
    headers.insert("Referer", header::HeaderValue::from_static("https://howlongtobeat.com/"));
    headers.insert("Accept", header::HeaderValue::from_static("*/*"));
    headers.insert("Origin", header::HeaderValue::from_static("https://howlongtobeat.com"));
    headers.insert("User-Agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.5005.63 Safari/537.36"));

    Client::builder()
        .default_headers(headers)
        .http1_title_case_headers()
        .build().unwrap()
}

