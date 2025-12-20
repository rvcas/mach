pub mod create;
pub mod list;

/// Manage workspaces
#[derive(clap::Subcommand)]
pub enum Cmd {
    /// Create a new workspace
    #[clap(alias = "c")]
    Create(create::Args),
    /// List all workspaces
    #[clap(alias = "l")]
    List(list::Args),
}

impl Cmd {
    pub async fn exec(self, services: &crate::service::Services) -> miette::Result<()> {
        match self {
            Cmd::Create(args) => args.exec(services).await,
            Cmd::List(args) => args.exec(services).await,
        }
    }
}
