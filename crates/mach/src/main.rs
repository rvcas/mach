#[tokio::main]
async fn main() -> miette::Result<()> {
    let cli = machich::Cli::default();

    cli.exec().await
}
