use std::env;
use std::fs::File;

use reqwest::blocking::Client;
use reqwest::header::HeaderValue;
use serde::de::{self, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::{self, Formatter};
use std::marker::PhantomData;
use time::OffsetDateTime;

struct Entry {
    _url: String,
}

impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntryVisitor<'de> {
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de> Visitor<'de> for EntryVisitor<'de> {
            type Value = Entry;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("struct Entry")
            }
            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut url = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "url" => {
                            url = Some(map.next_value()?);
                        }
                        _ => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }
                let url = url.ok_or_else(|| de::Error::missing_field("url"))?;
                Ok(Entry { _url: url })
            }
        }
        Deserializer::deserialize_struct(
            deserializer,
            "Entry",
            &["url"],
            EntryVisitor {
                lifetime: PhantomData,
            },
        )
    }
}

struct Tab {
    entries: Vec<Entry>,
}

impl<'de> Deserialize<'de> for Tab {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TabVisitor<'de> {
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de> Visitor<'de> for TabVisitor<'de> {
            type Value = Tab;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("struct Tab")
            }
            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "entries" => {
                            entries = Some(map.next_value()?);
                        }
                        _ => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }
                let entries = entries.ok_or_else(|| de::Error::missing_field("entries"))?;
                Ok(Tab { entries })
            }
        }
        Deserializer::deserialize_struct(
            deserializer,
            "Tab",
            &["entries"],
            TabVisitor {
                lifetime: PhantomData,
            },
        )
    }
}

struct Window {
    tabs: Vec<Tab>,
}

impl<'de> Deserialize<'de> for Window {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WindowVisitor<'de> {
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de> Visitor<'de> for WindowVisitor<'de> {
            type Value = Window;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("struct Window")
            }
            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut tabs = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "tabs" => {
                            tabs = Some(map.next_value()?);
                        }
                        _ => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }
                let tabs = tabs.ok_or_else(|| de::Error::missing_field("tabs"))?;
                Ok(Window { tabs })
            }
        }
        Deserializer::deserialize_struct(
            deserializer,
            "Window",
            &["tabs"],
            WindowVisitor {
                lifetime: PhantomData,
            },
        )
    }
}

struct SessionStore {
    windows: Vec<Window>,
}
impl<'de> Deserialize<'de> for SessionStore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SessionStoreVisitor<'de> {
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de> Visitor<'de> for SessionStoreVisitor<'de> {
            type Value = SessionStore;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("struct SessionStore")
            }
            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut windows = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "windows" => {
                            windows = Some(map.next_value()?);
                        }
                        _ => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }
                let windows = windows.ok_or_else(|| de::Error::missing_field("windows"))?;
                Ok(SessionStore { windows })
            }
        }
        Deserializer::deserialize_struct(
            deserializer,
            "SessionStore",
            &["windows"],
            SessionStoreVisitor {
                lifetime: PhantomData,
            },
        )
    }
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
    for tab in &val.windows[0].tabs {
        if !tab.entries.is_empty() {
            // println!("{}", tab.entries[tab.entries.len() - 1].url);
            count += 1;
        }
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
