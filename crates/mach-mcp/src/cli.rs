use crate::server::MachMcpServer;
use clap::Parser;
use machich::service::Services;
use miette::Result;

#[derive(Parser, Debug)]
#[command(
    name = "mach-mcp",
    about = "MCP server for mach task management",
    long_about = "Starts the mach MCP server that exposes todo management tools.\n\
Your AI agents can manage tasks instantly via the MCP protocol.\n\n\
Uses stdio transport for Claude Code integration."
)]
pub struct Cli {}

pub async fn run() -> Result<()> {
    let _cli = Cli::parse();

    let services = Services::bootstrap().await?;
    let server = MachMcpServer::new(services.todos)?;
    server.serve_stdio().await?;

    Ok(())
}
