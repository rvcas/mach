use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
    mach_mcp::cli::run().await
}
