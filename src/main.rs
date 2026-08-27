#![forbid(unsafe_code)]

use std::{collections::HashMap, env, fs};

use env_logger::{DEFAULT_FILTER_ENV, Env};
use log::{info, warn};
use utoipa::openapi::{
    ContentBuilder, InfoBuilder, License, OpenApiBuilder, PathsBuilder, RefOr, Required,
    ResponseBuilder, Schema, ServerBuilder, Tag,
    path::{OperationBuilder, ParameterBuilder, ParameterIn, PathItemBuilder},
    tag::TagBuilder,
};

use crate::models::SteamWebApiListResponse;

pub mod macros;
pub mod models;

const BASE_URL: &str = "https://api.steampowered.com";

fn main() {
    let env = Env::default().filter_or(DEFAULT_FILTER_ENV, "info");

    env_logger::init_from_env(env);

    let api_key = env::var("STEAM_API_KEY").ok();

    if api_key.is_some() {
        info!("A Steam API Key was provided")
    } else {
        warn!("No Steam API Key was provided")
    }

    let api_interfaces = fetch_endpoints(api_key.as_deref())
        .unwrap()
        .api_list
        .interfaces;

    info!("Fetched {} interfaces", api_interfaces.len());

    let openapi_builder = OpenApiBuilder::new().info(
        InfoBuilder::new()
            .title("Steam Web API")
            .version(cargo_crate_version!())
            .license(Some(License::new("MIT")))
            .build(),
    );

    let tags = api_interfaces
        .iter()
        .map(|interface| {
            (
                interface.name.to_owned(),
                TagBuilder::new().name(&interface.name).build(),
            )
        })
        .collect::<HashMap<String, Tag>>();

    let openapi_tags = tags.values().cloned().collect::<Vec<_>>();

    let mut path_builder = PathsBuilder::new();

    let default_response_schema = ResponseBuilder::new()
        .description("Successful response")
        .content(
            "application/json",
            ContentBuilder::new()
                .schema(Some(ObjectBuilder::new().build()))
                .build(),
        )
        .build();

    for api_interface in api_interfaces.iter() {
        for method in api_interface.methods.iter() {
            let normalized_name = if api_interface.name.starts_with('/') {
                &api_interface.name
            } else {
                &format!("/{}", api_interface.name)
            };

            let path_str = format!("{}/{}/v{}", normalized_name, method.name, method.version);

            let parameters = method
                .parameters
                .iter()
                .map(|param| {
                    ParameterBuilder::new()
                        .name(&param.name)
                        .description(param.description.as_deref())
                        .parameter_in(ParameterIn::Query)
                        .schema(schema_for_type(&param.type_field))
                        // this has to stay here
                        .required(bool_to_openapi_required(param.optional))
                        .build()
                })
                .collect::<Vec<_>>();

            let parameters = match parameters.is_empty() {
                true => None,
                false => Some(parameters),
            };

            let mut operation_builder = OperationBuilder::new()
                .parameters(parameters)
                .response("200", default_response_schema.clone());

            if tags.contains_key(&api_interface.name) {
                operation_builder = operation_builder.tag(&api_interface.name);
            }

            let path_item = PathItemBuilder::new()
                .description(method.description.as_deref())
                .operation(
                    steam_http_method_to_openapi(&method.http_method),
                    operation_builder.build(),
                )
                .build();

            path_builder = path_builder.path(path_str, path_item);
        }
    }

    let api = openapi_builder
        .tags(Some(openapi_tags))
        .paths(path_builder.build())
        .servers(Some([ServerBuilder::new()
            .url(BASE_URL)
            .description(Some("Steam Web API Base Url"))
            .build()]))
        .build();

    fs::write("schema.json", api.to_pretty_json().unwrap()).unwrap();

    info!("Done.")
}

fn bool_to_openapi_required(optional: bool) -> Required {
    if optional {
        Required::False
    } else {
        Required::True
    }
}

use utoipa::PartialSchema;
use utoipa::openapi::schema::ObjectBuilder;

fn any_schema() -> RefOr<Schema> {
    RefOr::T(Schema::Object(Default::default()))
}

fn schema_for_type(type_name: &str) -> Option<RefOr<Schema>> {
    match type_name {
        "bool" => Some(bool::schema()),
        "string" => Some(String::schema()),
        "int32" => Some(i32::schema()),
        "uint32" => Some(u32::schema()),
        "int64" => Some(i64::schema()),
        "uint64" => Some(u64::schema()),
        name if name.starts_with('{') => Some(any_schema()),
        _ => None,
    }
}

fn steam_http_method_to_openapi(method: &str) -> utoipa::openapi::HttpMethod {
    use utoipa::openapi::HttpMethod;

    let normalized_method = method.to_uppercase();

    match normalized_method.trim() {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        "DELETE" => HttpMethod::Delete,
        "PATCH" => HttpMethod::Patch,
        "OPTIONS" => HttpMethod::Options,
        "PUT" => HttpMethod::Put,
        _ => panic!("Method {method} is not mapped yet"),
    }
}

fn fetch_endpoints(api_key: Option<&str>) -> Result<SteamWebApiListResponse, serde_json::Error> {
    let request_client = reqwest::blocking::Client::new();

    let mut request = request_client
        .get(format!(
            "{BASE_URL}/ISteamWebAPIUtil/GetSupportedAPIList/v1"
        ))
        .header("User-Agent", user_agent_header!());

    if let Some(api_key) = api_key {
        request = request.query(&[("key", api_key)])
    }

    let response = request.send().expect("Couldnt fetch").bytes();

    match response {
        Ok(res) => serde_json::from_slice::<SteamWebApiListResponse>(&res),
        Err(err) => panic!("Couldnt parse to text: {err}"),
    }
}
