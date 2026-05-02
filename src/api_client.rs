use crate::model::*;

use futures::future::join_all;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, REFERER};
use reqwest::{header, Client};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::usize;

pub struct HltbApiClientConfig {
    init_url: String,
    find_url: String,
    domain: String,
}

pub struct HltbApiClient {
    auth_headers: header::HeaderMap,
    client: Client,
    cfg: HltbApiClientConfig,
}

impl HltbApiClient {
    pub fn new(cfg: HltbApiClientConfig) -> Self {
        let headers = header::HeaderMap::new();
        Self {
            auth_headers: headers,
            client: build_client(cfg.domain.as_str()),
            cfg,
        }
    }

    pub fn new_from_env() -> Self {
        let init_url = std::env::var("HLTB_INIT_URL").unwrap();
        let find_url = std::env::var("HLTB_FIND_URL").unwrap();
        let domain = std::env::var("HLTB_DOMAIN_URL").unwrap();

        Self::new(HltbApiClientConfig {
            init_url,
            find_url,
            domain,
        })
    }

    pub async fn fetch_steam_url(
        &mut self,
        hltb_id: i64,
    ) -> Result<Option<String>, Box<dyn Error>> {
        self.ensure_auth().await?;

        self.fetch_steam_url_for_id(hltb_id).await
    }

    pub async fn fetch_steam_urls_for_entries(
        &mut self,
        entries: &[Entry],
    ) -> Result<HashMap<i64, Option<String>>, Box<dyn Error>> {
        self.ensure_auth().await?;

        use std::sync::{Arc, Mutex};
        let self_arc = Arc::new(Mutex::new(self));

        let requests = entries.iter().map(|entry| {
            let hltb_id = entry.hltb_id;
            let self_clone = Arc::clone(&self_arc);
            async move {
                let url = self_clone
                    .lock()
                    .unwrap()
                    .fetch_steam_url_for_id(hltb_id)
                    .await;
                (hltb_id, url)
            }
        });

        let mut urls = HashMap::with_capacity(entries.len());
        for (hltb_id, url) in join_all(requests).await {
            match url {
                Ok(url) => {
                    urls.insert(hltb_id, url);
                }
                Err(err) => {
                    eprintln!("Failed to fetch steam url for entry {}: {}", hltb_id, err);
                    urls.insert(hltb_id, None);
                }
            }
        }

        Ok(urls)
    }

    async fn fetch_steam_url_for_id(
        &mut self,
        hltb_id: i64,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let url = format!("{}/game/{}", self.cfg.domain, hltb_id);
        let headers = self.auth_headers.clone();
        let rsp = self
            .client
            .clone()
            .get(&url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        match rsp.find("https://store.steampowered.com/app") {
            Some(start_at) => {
                let end_at = rsp[start_at..].find('"').unwrap() + start_at;
                let steam_url = &rsp[start_at..end_at];
                Ok(Some(steam_url.to_string()))
            }
            None => Ok(None),
        }
    }
}

impl HltbApiClient {
    pub async fn fetch_entries(&mut self, query: &str) -> Result<Vec<Entry>, Box<dyn Error>> {
        self.ensure_auth().await?;
        match self.query_games(query).await {
            Ok(rsp) => parse_entries_from_rsp(rsp, &self.cfg.domain),
            Err(err) => {
                // HLTB sometimes returns a 404 page for stale/invalid auth tokens.
                if err.to_string().contains("status 404") {
                    self.perform_auth().await?;
                    let rsp = self.query_games(query).await?;
                    return parse_entries_from_rsp(rsp, &self.cfg.domain);
                }

                Err(err)
            }
        }
    }

    async fn query_games(&self, query: &str) -> Result<Value, Box<dyn Error>> {
        let parts = query.split_whitespace().collect::<Vec<&str>>();
        let mut body = json!({
            "searchType": "games",
            "searchTerms": parts,
            "searchPage": 1,
            "size": 20,
            "searchOptions": {
                "games": {
                    "userId": 0,
                    "platform": "",
                    "sortCategory": "popular",
                    "rangeCategory": "main",
                    "rangeTime": { "min": null, "max": null },
                    "gameplay": {
                        "perspective": "",
                        "flow": "",
                        "genre": "",
                        "difficulty": ""
                    },
                    "rangeYear": {
                        "min": "",
                        "max": ""
                    },
                    "modifier": ""
                },
                "users": {
                    "sortCategory": "postcount"
                },
                "lists": {
                    "sortCategory": "follows"
                },
                "filter": "",
                "sort": 0,
                "randomizer": 0
            },
            "useCache": true
        });

        if let Some((hp_key, hp_val)) = self.hp_payload_field() {
            if let Some(map) = body.as_object_mut() {
                map.insert(hp_key, Value::String(hp_val));
            }
        }

        let referer = format!("{}/?q={}", self.cfg.domain, query);

        let rsp = self
            .client
            .post(&self.cfg.find_url)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "*/*")
            .header(ACCEPT_LANGUAGE, "en-US,en;q=0.9")
            .header(REFERER, referer)
            .headers(self.auth_headers.clone())
            .json(&body)
            .send()
            .await?;

        let text = rsp.text().await?;
        let json = serde_json::from_str(&text)?;
        Ok(json)
    }

    pub async fn ensure_auth(&mut self) -> Result<&mut HltbApiClient, Box<dyn Error>> {
        if !self.has_required_auth_headers() {
            self.perform_auth().await?;
        }

        Ok(self)
    }

    pub async fn perform_auth(&mut self) -> Result<&mut HltbApiClient, Box<dyn Error>> {
        let url = format!(
            "{}?t={}",
            &self.cfg.init_url,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis()
        );

        let rsp_future = self.client.get(url).send();

        let rsp = rsp_future
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        if let Some(map) = rsp.as_object() {
            for (key, value) in map {
                if let Some(value) = value.as_str() {
                    let header_key = match key.as_str() {
                        "token" => "x-auth-token".to_string(),
                        "hpKey" => "x-hp-key".to_string(),
                        "hpVal" => "x-hp-val".to_string(),
                        _ => format!("x-{}", key),
                    };

                    self.upsert_auth_header(header_key.as_str(), value)?;
                }
            }
        }

        if !self.has_required_auth_headers() {
            return Err("HLTB auth init response did not include token/hpKey/hpVal".into());
        }

        Ok(self)
    }

    fn has_required_auth_headers(&self) -> bool {
        self.auth_headers.contains_key("x-auth-token")
            && self.auth_headers.contains_key("x-hp-key")
            && self.auth_headers.contains_key("x-hp-val")
    }

    fn upsert_auth_header(&mut self, header_key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let header_name = header::HeaderName::from_str(header_key)?;
        self.auth_headers.remove(&header_name);
        self.auth_headers
            .insert(header_name, header::HeaderValue::from_str(value)?);

        Ok(())
    }

    fn hp_payload_field(&self) -> Option<(String, String)> {
        let hp_key = self
            .auth_headers
            .get("x-hp-key")?
            .to_str()
            .ok()?
            .to_string();
        let hp_val = self
            .auth_headers
            .get("x-hp-val")?
            .to_str()
            .ok()?
            .to_string();
        Some((hp_key, hp_val))
    }
}

pub fn parse_entries_from_rsp(mut rsp: Value, domain: &str) -> Result<Vec<Entry>, Box<dyn Error>> {
    let entries_limit =
        usize::from_str(&std::env::var("ENTRIES_LIMIT").unwrap_or("5".to_string())).unwrap();

    let data = rsp["data"].take();
    let entries = serde_json::from_value::<Vec<RawEntry>>(data)?;

    Ok(entries
        .into_iter()
        .map(|raw| map_entry(raw, domain))
        .take(entries_limit)
        .collect())
}

fn map_entry(raw: RawEntry, domain: &str) -> Entry {
    let name = raw.game_name.to_string();
    let link = format!("{}/game/{}", domain, raw.game_id);
    let img = format!("{}/games/{}", domain, raw.game_image);
    let descr = map_descr(&raw);
    Entry::new(name, link, img, descr, raw.game_id)
}

fn map_descr(raw: &RawEntry) -> String {
    let main = map_hours(raw.comp_main);
    let plus = map_hours(raw.comp_plus);
    let comp = map_hours(raw.comp_100);
    format!(
        "\
Main Story: {}
Main + Extra: {}
Completionist: {}\n",
        main, plus, comp
    )
}

fn map_hours(sec: i32) -> String {
    let hours = sec as f32 / 60.0 / 60.0;
    let integral = f32::trunc(hours);
    let fraction = hours - integral;

    if integral < 1.0 {
        "--".to_string()
    } else if fraction > 0.5 {
        format!("{}½ Hours", integral)
    } else {
        format!("{} Hours", integral)
    }
}

fn build_client(domain: &str) -> Client {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Referer",
        header::HeaderValue::from_str(&format!("{}/", domain)).unwrap(),
    );
    headers.insert("Accept", header::HeaderValue::from_static("*/*"));
    headers.insert(
        "Origin",
        header::HeaderValue::from_str(domain).unwrap(),
    );
    headers.insert("User-Agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.5005.63 Safari/537.36"));

    Client::builder()
        .cookie_store(true)
        .default_headers(headers)
        .build()
        .unwrap()
}
