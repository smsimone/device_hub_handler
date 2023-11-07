use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    response::Response,
    Router,
    routing::post,
};
use tracing::info;

use crate::{services, utils::multipart_helper};

pub fn initialize_router() -> Router {
    Router::new()
        .route("/upload", post(upload_test_file))
        .route("/run/:test_name", post(run_test))
}

async fn upload_test_file(mut multipart: Multipart) -> Result<Response, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file = match multipart_helper::extract_file(field).await {
            Some(file) => file,
            None => continue,
        };

        return match services::maestro_service::store_file(file) {
            Ok(_) => Ok(Response::default()),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        };
    }

    Ok(Response::default())
}

async fn run_test(Path(test_name): Path<String>) -> Result<StatusCode, StatusCode> {
    info!("Running test {}", test_name);
    services::maestro_service::execute_test(&test_name);
    Ok(StatusCode::OK)
}
