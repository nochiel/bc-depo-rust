// @todo Pass a logger to the module.

use log::info;
use nu_ansi_term::Color::Green;
use warp::{
    http::StatusCode,
    reject::Rejection,
    reply::{self, Reply},
    Filter,
};

const SCHEMA_NAME: &str = "depo";

use crate::{
    db_depo::{create_db, server_pool},
    reset_db, Depo,
};

async fn key_handler(depo: Depo) -> Result<Box<dyn Reply>, Rejection> {
    Ok(Box::new(reply::with_status(
        depo.public_key_string().to_string(),
        StatusCode::OK,
    )))
}

fn with_depo(
    depo: Depo,
) -> impl Filter<Extract = (Depo,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || depo.clone())
}

pub async fn make_routes() {
    // @fixme What's the return type?
    // @todo Each module will be have it's own path.
    // e.g. /api/depo/ for depo
    // e.g. /api/timestamp/ for timestamp

    let depo = Depo::new_db(SCHEMA_NAME).await;

    let key_route = warp::path::end()
        .and(warp::get())
        .and(with_depo(depo.clone()))
        .and_then(key_handler);

    let operation_route = warp::path::end()
        .and(warp::post())
        .and(with_depo(depo.clone()))
        .and(warp::body::bytes())
        .and_then(operation_handler);

    let cloned_schema_name = SCHEMA_NAME.to_owned();
    let reset_db_route = warp::path("reset-db")
        .and(warp::post())
        .and(warp::any().map(move || cloned_schema_name.clone()))
        .and_then(reset_db_handler);

    let routes = key_route.or(operation_route).or(reset_db_route);

    routes
}

pub async fn start_server() -> anyhow::Result<()> {
    create_db(&server_pool(), SCHEMA_NAME).await?;
    let depo = Depo::new_db(SCHEMA_NAME).await?;

    info!(
        "{}",
        Green.paint(format!(
            "Starting Blockchain Commons Depository on {}:{}",
            host, port
        ))
    );
    info!(
        "{}",
        Green.paint(format!("Public key: {}", depo.public_key_string()))
    );

    Ok(())
}
