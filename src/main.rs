use depo::{start_server, setup_log};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_log();

    let schema_name = "depo";

    start_server(schema_name, 5332).await
}
