use cli::run_cli;

mod cli;
mod resolve_command;
mod publish_command;
mod pkarr_zone;


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    run_cli().await;
    Ok(())
}
