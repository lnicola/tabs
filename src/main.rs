use std::cmp::Reverse;
use std::collections::HashMap;
use std::env;
use std::fs::File;

use reqwest::blocking::Client;
use reqwest::header::HeaderValue;
use reqwest::Url;
use serde::Deserialize;
use time::OffsetDateTime;

#[derive(Deserialize)]
struct Entry {
    url: String,
}

#[derive(Deserialize)]
struct Tab {
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct Window {
    tabs: Vec<Tab>,
}

#[derive(Deserialize)]
struct SessionStore {
    windows: Vec<Window>,
}

struct ApiConfig {
    api_url: String,
    access_token: String,
}

fn main() {
    let mut path = dirs::home_dir().expect("cannot find HOME directory");
    path.push(".mozilla/firefox");
    path.push("9pbspxtt.default");
    path.push("sessionstore-backups/recovery.jsonlz4");

    let file = File::open(&path).unwrap();
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file).unwrap() };
    let buf = lz4_flex::decompress_size_prepended(&mmap[8..]).unwrap();

    let val: SessionStore = serde_json::from_slice(&buf).unwrap();

    let mut count = 0i16;
    let mut domains = HashMap::<String, u32>::new();
    for tab in &val.windows[0].tabs {
        if !tab.entries.is_empty() {
            let _url = &tab.entries[tab.entries.len() - 1].url;
            let url = Url::parse(&_url).unwrap();
            if let Some(host) = url.host_str() {
                *domains.entry(host.to_string()).or_default() += 1;
            }
            count += 1;
        }
    }
    println!("{count} tabs");
    let mut domains = domains.into_iter().collect::<Vec<_>>();
    domains.sort_unstable_by_key(|p| Reverse(p.1));
    for (domain, count) in domains.into_iter().take(10) {
        println!("{} {}", domain, count);
    }
    let api_url = env::var("API_URL").unwrap();
    let access_token = env::var("ACCESS_TOKEN").unwrap();

    let api_config = ApiConfig {
        api_url,
        access_token,
    };

    let time = OffsetDateTime::now_utc();
    let client = Client::new();
    let body = format!(
        r#"{{ "time": {}, "tabs": {} }}"#,
        time.unix_timestamp(),
        count
    );
    client
        .post(format!("{}tabs", api_config.api_url))
        .bearer_auth(&api_config.access_token)
        .header(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )
        .body(body)
        .send()
        .unwrap()
        .error_for_status()
        .unwrap();
}
