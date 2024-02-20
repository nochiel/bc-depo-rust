use log::info;
use nu_ansi_term::Color::Green;
use warp::{
    http::StatusCode,
    reject::Rejection,
    reply::{self, Reply},
    Filter,
};

use crate::{
    db_depo::{create_db, server_pool},
    reset_db, reset_db_handler, Depo, InvalidBody,
};

pub async fn start_server(schema_name: &str, port: u16) -> anyhow::Result<()> {
    create_db(&server_pool(), schema_name).await?;

    let depo = Depo::new_db(schema_name).await?;

    let key_route = warp::path::end()
        .and(warp::get())
        .and(with_depo(depo.clone()))
        .and_then(key_handler);

    let operation_route = warp::path::end()
        .and(warp::post())
        .and(with_depo(depo.clone()))
        .and(warp::body::bytes())
        .and_then(operation_handler);

    let cloned_schema_name = schema_name.to_owned();

    let reset_db_route = warp::path("reset-db")
        .and(warp::post())
        .and(warp::any().map(move || cloned_schema_name.clone()))
        .and_then(reset_db_handler);

    let routes = key_route.or(operation_route).or(reset_db_route);

    let host = "0.0.0.0";
    let addr = format!("{}:{}", host, port);
    let socket_addr = addr.parse::<std::net::SocketAddr>()?;

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

    warp::serve(routes).run(socket_addr).await;

    Ok(())
}

fn with_depo(
    depo: Depo,
) -> impl Filter<Extract = (Depo,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || depo.clone())
}

async fn key_handler(depo: Depo) -> Result<Box<dyn Reply>, Rejection> {
    Ok(Box::new(reply::with_status(
        depo.public_key_string().to_string(),
        StatusCode::OK,
    )))
}

async fn operation_handler(depo: Depo, body: bytes::Bytes) -> Result<Box<dyn Reply>, Rejection> {
    let body_string = std::str::from_utf8(&body)
        .map_err(|_| warp::reject::custom(InvalidBody))?
        .to_string();
    let a = depo.handle_request_string(body_string).await;
    let result: Box<dyn Reply> = Box::new(reply::with_status(a, StatusCode::OK));
    Ok(result)
}
