pub mod db_depo;
pub mod depo_impl;
pub use db_depo::{can_connect_to_db, create_db_if_needed, reset_db};
pub use depo::{make_routes, reset_db_handler, start_server, API_NAME};

pub mod function;
pub mod mem_depo;
pub mod record;

mod depo;
