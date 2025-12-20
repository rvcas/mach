pub mod create;
pub mod list;
pub mod update;

/// Manage workspaces
#[derive(clap::Subcommand)]
pub enum Cmd {
    /// Create a new workspace
    #[clap(visible_alias = "c")]
    Create(create::Args),
    /// List all workspaces
    #[clap(visible_alias = "l")]
    List(list::Args),
    /// Update a workspace
    #[clap(visible_alias = "u")]
    Update(update::Args),
}

impl Cmd {
    pub async fn exec(self, services: &crate::service::Services) -> miette::Result<()> {
        match self {
            Cmd::Create(args) => args.exec(services).await,
            Cmd::List(args) => args.exec(services).await,
            Cmd::Update(args) => args.exec(services).await,
        }
    }
}
