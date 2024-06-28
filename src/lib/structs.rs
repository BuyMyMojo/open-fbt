use merge::Merge;
use serde::{Deserialize, Serialize};
use serde_with::{As, FromInto};

// User data, which is stored and accessible in all command invocations
#[derive(Debug)]
pub struct Data {}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, PartialEq, Hash, Eq)]
pub struct CsvEntry {
    pub AuthorID: String,
    pub Author: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Merge, Clone, Hash, Eq)]
pub struct UserInfo {
    #[serde(with = "As::<FromInto<OptionalString2>>")]
    pub vrc_id: Option<String>,
    #[serde(with = "As::<FromInto<OptionalString>>")]
    pub username: Option<String>,
    #[serde(with = "As::<FromInto<OptionalString2>>")]
    pub discord_id: Option<String>,
    #[merge(strategy = merge::vec::append)]
    pub offences: Vec<Offense>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Hash, Eq)]
pub struct GuildSettings {
    pub channel_id: String,
    pub kick: bool,
    pub server_name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Merge, Hash, Eq)]
pub struct Offense {
    #[merge(skip)]
    pub guild_id: String,
    #[merge(skip)]
    pub reason: String,
    #[serde(with = "As::<FromInto<OptionalString2>>")]
    pub image: Option<String>,
    #[serde(with = "As::<FromInto<OptionalString2>>")]
    pub extra: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Hash, Eq)]
pub struct GuildAuthList {
    pub users: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Hash, Eq)]
pub struct ClearedUser {
    pub user_id: String,
    pub username: String,
    pub where_found: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Hash, Eq)]
pub struct MonitoredGuildInfo {
    pub guild_name: String,
    pub guild_id: String,
    pub invite_link: String,
    pub updated: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Hash, Eq)]
pub struct BlacklistHit {
    pub user_id: String,
    pub username: String,
    pub guild_id: String,
    pub reason: String,
    pub image: String,
    pub extra: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaybackResponse {
    pub url: Option<String>,
    #[serde(rename = "job_id")]
    pub job_id: Option<String>,
    pub message: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "status_ext")]
    pub status_ext: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaybackStatus {
    #[serde(rename = "http_status")]
    pub http_status: Option<i64>,
    #[serde(default)]
    outlinks: Vec<String>,
    pub timestamp: Option<String>,
    #[serde(rename = "original_url")]
    pub original_url: Option<String>,
    resources: Vec<String>,
    #[serde(rename = "duration_sec")]
    pub duration_sec: Option<f64>,
    pub status: String,
    #[serde(rename = "job_id")]
    pub job_id: String,
    pub counters: Option<Counters>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Counters {
    pub outlinks: i64,
    pub embeds: i64,
}

#[derive(Deserialize, Serialize)]
pub struct OptionalString(pub Option<String>);

impl From<OptionalString> for Option<String> {
    fn from(val: OptionalString) -> Self {
        val.0.map_or_else(|| Some("N/A".to_string()), Some)
    }
}

impl From<Option<String>> for OptionalString {
    fn from(val: Option<String>) -> Self {
        val.map_or_else(|| Self(Some("N/A".to_string())), |s| Self(Some(s)))
    }
}

#[derive(Deserialize, Serialize)]
pub struct OptionalString2(pub Option<String>);

impl From<OptionalString2> for Option<String> {
    fn from(val: OptionalString2) -> Self {
        val.0.map_or_else(
            || Some("N/A".to_string()),
            |s| match s.as_str() {
                "0" => Some("N/A".to_string()),
                x => Some(x.to_string()),
            },
        )
    }
}

impl From<Option<String>> for OptionalString2 {
    fn from(val: Option<String>) -> Self {
        val.map_or_else(|| Self(Some("N/A".to_string())), |s| Self(Some(s)))
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasteResponse {
    pub key: String,
}
