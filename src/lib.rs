mod function;
mod log;
mod modules;
mod recovery_continuation;
mod server;
mod user;

pub use function::Depo;
pub use log::setup_log;
pub use server::start_server;

const MAX_DATA_SIZE: u32 = 1000;
const CONTINUATION_EXPIRY_SECONDS: u32 = 60 * 60 * 24;

// API

#[derive(Debug)]
pub struct InvalidBody;
impl warp::reject::Reject for InvalidBody {}

#[derive(Debug)]
pub struct AnyhowError(anyhow::Error);
impl warp::reject::Reject for AnyhowError {}

use warp::{
    http::StatusCode,
    reject::Rejection,
    reply::{self, Reply},
};
