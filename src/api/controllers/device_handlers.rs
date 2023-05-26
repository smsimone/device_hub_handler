use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use log::info;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::services::{device_service::find_devices, maestro_service};

#[derive(Deserialize)]
pub struct BundleName {
    pub ios_bundle: String,
    pub android_bundle: String,
    pub device_id: Option<String>,
}

pub fn initialize_router() -> Router {
    Router::new()
        .route("/", get(get_devices))
        .route("/:test_name", post(run_test))
}

async fn get_devices() -> Result<Json<Value>, StatusCode> {
    info!("Getting connected devices");
    let devices = find_devices(None)
        .iter()
        .map(|dev| {
            info!("Found device {}", dev.get_device_name());
            json!({
                "name": dev.get_device_name(),
                "os": dev.get_os_type(),
                "id": dev.get_device_id()
            })
        })
        .collect::<Vec<Value>>();

    return Ok(Json(json!({ "devices": [devices] })));
}

async fn run_test(
    Path(test_name): Path<String>,
    Query(query_params): Query<BundleName>,
) -> Result<Response, StatusCode> {
    match &query_params.device_id {
        Some(device_id) => {
            info!("Running '{}' on device {}", &test_name, &device_id);
            return maestro_service::run_test(&test_name, &device_id, &query_params).await;
        }
        None => {
            info!("Running tests on all connected devices");
            return Ok(Response::default());
        }
    }
}