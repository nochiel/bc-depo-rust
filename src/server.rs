use log::info;
use nu_ansi_term::Color::Green;
use warp::{
    filters::{path::Exact, BoxedFilter},
    http::StatusCode,
    reject::Rejection,
    reply::{self, Reply},
    Filter,
};

use crate::api::InvalidBody;
use crate::modules::torgap;
use crate::modules::{
    depo,
    depo::{
        db_depo::{create_db, server_pool},
        function::Depo,
        reset_db, reset_db_handler,
    },
};

pub async fn start_server(schema_name: &str, port: u16) -> anyhow::Result<()> {
    // @todo Loop through all modules and get routes.
    let routes = make_routes().await;

    let host = "0.0.0.0"; // 127.0.0.1 won't work inside docker.
    let addr = format!("{}:{}", host, port);
    let socket_addr = addr.parse::<std::net::SocketAddr>()?;

    {
        depo::start_server().await;
        torgap::start_server().await;
    }

    warp::serve(routes).run(socket_addr).await;

    Ok(())
}

async fn make_routes() -> BoxedFilter<(impl Reply,)> {
    let api = warp::path("api");
    let status = api.and(warp::path!("status")).and_then(status_handler);
    let depo_route = api.and(warp::path(depo::API_NAME));
    let depo_route = depo_route.and(depo::make_routes().await);
    let torgap_route = api.and(warp::path(torgap::API_NAME));
    let torgap_route = torgap_route.and(torgap::make_routes().await);

    /*
    let mut routes = vec![];
    routes.push((depo::API_NAME, depo::make_routes().await));
    let api /* : BoxedFilter<(dyn Reply, dyn Extract + Clone)>*/ = warp::path("api");

    let mut modules = vec![];
    for r in routes {
        let module = warp::path(String::from(r.0)).and(r.1);
        modules.push(api.and(module));
    }
    let mut result = modules[0].clone().or(modules[1].clone());
    // @fixme Generic types in Rust are so stupid and I'm stupid so
    // I can't figure out what fucking type result should be and this will
    // never compile.
    // So for now, each module's routes will be added manually isntead of
    // looping through a collection.
    let result = warp::path::end().or(modules[0]);
    for m in &modules[1..] {
        result = result.or(m.clone());
    }
    */

    // Add all routes here.
    let result = status.or(depo_route).or(torgap_route);
    result.boxed()
}

fn status() -> BoxedFilter<(impl Reply,)> {
    let api = warp::path("api");

    api.and(warp::path!("status"))
        .and_then(status_handler)
        .boxed()
}

// Ref. https://github.com/seanmonstar/warp/blob/master/examples/dyn_reply.rs
async fn status_handler() -> Result<Box<dyn Reply>, Rejection> {
    Ok(Box::new(reply::json(&"Server is running")))
}

#[tokio::test]
async fn test_status() {
    let filter = status();
    let result = warp::test::request()
        .path("/api/status")
        .reply(&filter)
        .await;
    assert_eq!(result.status(), 200, "{}", result.status());
}
