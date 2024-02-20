pub mod db_depo;
mod depo;
pub mod depo_impl;
pub use db_depo::{can_connect_to_db, create_db_if_needed, reset_db};

mod mem_depo;
mod record;
