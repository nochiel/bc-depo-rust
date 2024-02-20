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

use crate::api::InvalidBody;
use crate::modules::depo::function::Depo;
use crate::modules::depo::{
    db_depo::{create_db, server_pool},
    reset_db,
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

// @todo Move this to Server. Server will receive routes from
// the plugin then loop through the array of routes and add an then(operation_handler) to each.
async fn operation_handler(depo: Depo, body: bytes::Bytes) -> Result<Box<dyn Reply>, Rejection> {
    let body_string = std::str::from_utf8(&body)
        .map_err(|_| warp::reject::custom(InvalidBody))?
        .to_string();
    let a = depo.handle_request_string(body_string).await;
    let result: Box<dyn Reply> = Box::new(reply::with_status(a, StatusCode::OK));
    Ok(result)
}

pub async fn make_routes(
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
// impl warp::generic::Either<Filter<Extract = (Depo,), Error = std::convert::Infallible>> + Clone
{
    // @fixme What's the return type?
    // @todo Each module will be have it's own path.
    // e.g. /api/depo/ for depo
    // e.g. /api/timestamp/ for timestamp

    let depo = Depo::new_db(SCHEMA_NAME).await.unwrap();

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
        Green.paint(format!("Starting Blockchain Commons Depository"))
    );
    info!(
        "{}",
        Green.paint(format!("Public key: {}", depo.public_key_string()))
    );

    Ok(())
}

pub async fn reset_db_handler(schema_name: String) -> Result<Box<dyn Reply>, Rejection> {
    match reset_db(&schema_name).await {
        Ok(_) => Ok(Box::new(reply::with_status("Database reset successfully. A new private key has been assigned. Server must be restarted.", StatusCode::OK))),
        Err(e) => {
            let error_message = format!("Failed to reset database: {}", e);
            let reply = reply::html(error_message);
            Ok(Box::new(reply::with_status(reply, StatusCode::INTERNAL_SERVER_ERROR)))
        },
    }
}
