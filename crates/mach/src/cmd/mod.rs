pub mod add;
pub mod list;
pub mod projects;
pub mod workspaces;

#[derive(clap::Subcommand)]
pub enum Cmd {
    #[clap(alias = "a")]
    Add(add::Args),
    #[clap(alias = "l")]
    List(list::Args),
    /// Manage workspaces
    #[clap(alias = "w")]
    #[command(subcommand)]
    Workspaces(workspaces::Cmd),
    /// Manage projects
    #[clap(alias = "p")]
    #[command(subcommand)]
    Projects(projects::Cmd),
}

impl Cmd {
    pub async fn exec(self, services: &crate::service::Services) -> miette::Result<()> {
        match self {
            Cmd::Add(args) => args.exec(services).await,
            Cmd::List(args) => args.exec(services).await,
            Cmd::Workspaces(cmd) => cmd.exec(services).await,
            Cmd::Projects(cmd) => cmd.exec(services).await,
        }
    }
}
