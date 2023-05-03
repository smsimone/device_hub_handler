use api::handlers::initialize_router;
use axum::{extract::DefaultBodyLimit, Server};
use std::{
    fs::{create_dir, create_dir_all, read_dir, remove_dir_all, DirEntry},
    io::Error,
    net::SocketAddr,
    path::Path,
    process::exit,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use dialoguer::Confirm;

use log::{error, info, warn};
use utils::{command_executor::command_exists, env_helper::ENV_DATA};

mod api;
mod device_adapter;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    validate_depdencies();
    match check_extraction_path() {
        Ok(_) => {}
        Err(err) => {
            error!("Could not check extraction path: {}", err.to_string());
            exit(1);
        }
    }

    let router = initialize_router()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(DefaultBodyLimit::disable());

    let address = SocketAddr::from(([0, 0, 0, 0], 42069));
    tracing::info!("Listening on {}", &address);
    Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}

fn check_extraction_path() -> Result<(), Error> {
    let extract_path = String::from(&ENV_DATA.lock().unwrap().extract_output_dir);

    let dir_path = Path::new(&extract_path);
    if !dir_path.exists() {
        match create_dir_all(dir_path) {
            Ok(_) => {
                info!("Created directory {}", &extract_path);
                Ok(())
            }
            Err(err) => {
                error!("Failed to create directory: {}", err);
                Err(err)
            }
        }
    } else {
        if !(Path::new(&extract_path).is_dir()) {
            error!("The path {} is not a directory", &extract_path);
            exit(1);
        }

        let content = match read_dir(&extract_path) {
            Ok(val) => val,
            Err(_) => panic!("Could not read the directory"),
        };
        let content = content.collect::<Vec<Result<DirEntry, Error>>>();
        if !content.is_empty() {
            let should_erase = Confirm::new()
                .with_prompt(format!(
                    "The directory {} is not empty, do you want to delete its content?",
                    &extract_path
                ))
                .interact()?;

            if should_erase {
                match remove_dir_all(&extract_path) {
                    Ok(_) => {
                        info!("Directory {} erased", &extract_path);
                        return Ok(());
                    }
                    Err(err) => {
                        error!(
                            "Failed to erase directory {}: {}",
                            &extract_path,
                            err.to_string()
                        );
                        exit(1);
                    }
                }
            } else {
                warn!("Cannot continue if {} is not empty", &extract_path);
                exit(1);
            }
        }
        return Ok(());
    }
}

/// Checks whether all the required binaries are installed and present in PATH
fn validate_depdencies() {
    // FIXME: should remove this dependency to use just adb and idb
    // Used to find all connected devices
    if command_exists(&"flutter".to_string()).is_err() {
        exit(1);
    }

    // Used to interact with android devices
    if command_exists(&"adb".to_string()).is_err() {
        exit(1);
    }

    // Used to manage android appbundles
    if command_exists(&"bundletool".to_string()).is_err() {
        exit(1);
    }

    // Used to interact with ios devices just like adb
    if command_exists(&"idb".to_string()).is_err() {
        exit(1);
    }

    // Used to extract packagename from an apk
    if command_exists(&"aapt2".to_string()).is_err() {
        exit(1);
    }

    if command_exists(&"tar".to_string()).is_err() {
        exit(1);
    }
}
