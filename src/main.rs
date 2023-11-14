use depo::{db::reset_db, Depo};

use warp::{Filter, http::StatusCode, reply::{self, Reply}, reject::Rejection};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let depo = Depo::new_db().await?;

    let key_route = warp::path::end()
        .and(warp::get())
        .and(with_depo(depo.clone()))
        .and_then(key_handler);

    let operation_route = warp::path::end()
        .and(warp::post())
        .and(with_depo(depo.clone()))
        .and(warp::body::bytes())
        .and_then(operation_handler);

    let reset_db_route = warp::path("reset-db")
        .and(warp::post())
        .and_then(reset_db_handler);

    let routes =
        key_route
        .or(operation_route)
        .or(reset_db_route);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 5332))
        .await;

    Ok(())
}

fn with_depo(depo: Depo) -> impl Filter<Extract = (Depo,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || depo.clone())
}

async fn key_handler(depo: Depo) -> Result<Box<dyn Reply>, Rejection> {
    Ok(Box::new(reply::with_status(depo.public_key_string().to_string(), StatusCode::OK)))
}

async fn operation_handler(depo: Depo, body: bytes::Bytes) -> Result<Box<dyn Reply>, Rejection> {
    let body_string = std::str::from_utf8(&body).map_err(|_| warp::reject::custom(InvalidBody))?.to_string();
    let a: Result<String, anyhow::Error> = depo.handle_request_string(body_string).await;
    let result: Result<Box<dyn Reply>, Rejection> = match a {
        Ok(response) => {
            Ok(Box::new(reply::with_status(response, StatusCode::OK)))
        },
        Err(e) => {
            Err(warp::reject::custom(AnyhowError(e)))
        }
    };
    result
}

async fn reset_db_handler() -> Result<Box<dyn Reply>, Rejection> {
    match reset_db().await {
        Ok(_) => Ok(Box::new(reply::with_status("Database reset successfully. A new private key has been assigned. Server must be restarted.", StatusCode::OK))),
        Err(e) => {
            let error_message = format!("Failed to reset database: {}", e);
            let reply = reply::html(error_message);
            Ok(Box::new(reply::with_status(reply, StatusCode::INTERNAL_SERVER_ERROR)))
        },
    }
}

#[derive(Debug)]
struct InvalidBody;
impl warp::reject::Reject for InvalidBody {}

#[derive(Debug)]
struct AnyhowError(anyhow::Error);
impl warp::reject::Reject for AnyhowError {}
