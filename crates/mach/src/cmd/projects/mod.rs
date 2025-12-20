pub mod create;
pub mod done;
pub mod list;
pub mod reopen;
pub mod update;

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum ProjectStatus {
    Pending,
    Done,
    Permanent,
}

impl ProjectStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectStatus::Pending => "pending",
            ProjectStatus::Done => "done",
            ProjectStatus::Permanent => "permanent",
        }
    }
}

/// Manage projects
#[derive(clap::Subcommand)]
pub enum Cmd {
    /// Create a new project
    #[clap(visible_alias = "c")]
    Create(create::Args),
    /// List projects
    #[clap(visible_alias = "l")]
    List(list::Args),
    /// Update a project
    #[clap(visible_alias = "u")]
    Update(update::Args),
    /// Mark a project as done
    #[clap(visible_alias = "d")]
    Done(done::Args),
    /// Reopen a project (set status to pending)
    #[clap(visible_alias = "r")]
    Reopen(reopen::Args),
}

impl Cmd {
    pub async fn exec(self, services: &crate::service::Services) -> miette::Result<()> {
        match self {
            Cmd::Create(args) => args.exec(services).await,
            Cmd::List(args) => args.exec(services).await,
            Cmd::Update(args) => args.exec(services).await,
            Cmd::Done(args) => args.exec(services).await,
            Cmd::Reopen(args) => args.exec(services).await,
        }
    }
}
