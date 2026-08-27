use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamWebApiListResponse {
    #[serde(rename = "apilist")]
    pub api_list: ApiList,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiList {
    pub interfaces: Vec<SteamWebApiInterface>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamWebApiInterface {
    pub name: String,
    pub methods: Vec<SteamWebApiMethod>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamWebApiMethod {
    pub name: String,
    pub version: u8,
    #[serde(rename = "httpmethod")]
    pub http_method: String,
    pub parameters: Vec<SteamWebApiMethodParameter>,
    pub description: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamWebApiMethodParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub optional: bool,
    pub description: Option<String>,
}
