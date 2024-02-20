use depo::{setup_log, start_server};
use log::error;
use nu_ansi_term::Color::Red;

use crate::modules::depo;

#[tokio::main]
async fn main() {
    setup_log();

    let schema_name = "depo";

    // @todo Each module should have a start function that the server calls.
    if let Err(e) = start_server(schema_name, 5332).await {
        error!(
            "{}",
            Red.paint("Could not start server. Is the database running?")
                .to_string()
        );
        error!("{}", Red.paint(format!("{}", e)).to_string());
    };
}
