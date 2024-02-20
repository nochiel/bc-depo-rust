pub mod db_depo;
mod depo;
pub mod depo_impl;
pub use db_depo::{can_connect_to_db, create_db_if_needed, reset_db};
pub use depo::reset_db_handler;

pub mod function;
pub mod mem_depo;
pub mod record;
