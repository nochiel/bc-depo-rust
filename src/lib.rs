pub mod db_depo;
pub mod depo_impl;
pub mod function;
pub mod mem_depo;
pub mod record;
pub mod recovery_continuation;
pub mod user;
pub mod server;
pub mod log;

pub use function::Depo;
pub use server::start_server;
pub use db_depo::{reset_db, can_connect_to_db};

const MAX_DATA_SIZE: u32 = 1000;
const CONTINUATION_EXPIRY_SECONDS: u32 = 60 * 60 * 24;
