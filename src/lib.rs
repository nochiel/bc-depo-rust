pub mod db_depo;
pub mod db;
pub mod depo_impl;
pub mod depo_struct;
pub mod mem_depo;
pub mod record;
pub mod recovery_continuation;
pub mod user;
pub mod server;

pub use depo_struct::Depo;
pub use server::start_server;

const MAX_PAYLOAD_SIZE: u32 = 1000;
const CONTINUATION_EXPIRY_SECONDS: u32 = 60 * 60 * 24;
