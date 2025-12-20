pub mod add;
pub mod delete;
pub mod done;
pub mod list;
pub mod projects;
pub mod reopen;
pub mod update;
pub mod workspaces;

#[derive(clap::Subcommand)]
pub enum Cmd {
    #[clap(visible_alias = "a")]
    Add(add::Args),
    #[clap(visible_alias = "l")]
    List(list::Args),
    #[clap(visible_alias = "d")]
    Done(done::Args),
    #[clap(visible_alias = "r")]
    Reopen(reopen::Args),
    #[clap(visible_alias = "u")]
    Update(update::Args),
    /// Delete a todo
    #[clap(visible_alias = "rm")]
    Delete(delete::Args),
    /// Manage workspaces
    #[clap(visible_alias = "w")]
    #[command(subcommand)]
    Workspaces(workspaces::Cmd),
    /// Manage projects
    #[clap(visible_alias = "p")]
    #[command(subcommand)]
    Projects(projects::Cmd),
}

impl Cmd {
    pub async fn exec(self, services: &crate::service::Services) -> miette::Result<()> {
        match self {
            Cmd::Add(args) => args.exec(services).await,
            Cmd::List(args) => args.exec(services).await,
            Cmd::Done(args) => args.exec(services).await,
            Cmd::Reopen(args) => args.exec(services).await,
            Cmd::Update(args) => args.exec(services).await,
            Cmd::Delete(args) => args.exec(services).await,
            Cmd::Workspaces(cmd) => cmd.exec(services).await,
            Cmd::Projects(cmd) => cmd.exec(services).await,
        }
    }
}
