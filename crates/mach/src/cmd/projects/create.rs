use crate::service::Services;

/// Create a new project
#[derive(clap::Args)]
pub struct Args {
    /// Workspace name or UUID (required)
    #[clap(short, long)]
    workspace: String,

    /// Set project status to permanent
    #[clap(short, long, default_value = "false")]
    permanent: bool,

    /// Name of the project (quoted or space separated)
    #[clap(required = true)]
    name: Vec<String>,
}

impl Args {
    pub async fn exec(self, services: &Services) -> miette::Result<()> {
        let name = self.name.join(" ");

        let workspace = services
            .workspaces
            .find_by_name_or_id(&self.workspace)
            .await?
            .ok_or_else(|| miette::miette!("workspace '{}' not found", self.workspace))?;

        let status = if self.permanent { "permanent" } else { "pending" };

        let project = services
            .projects
            .create(&name, workspace.id, status)
            .await?;

        println!(
            "Created project '{}' in workspace '{}'",
            project.name, workspace.name
        );

        Ok(())
    }
}
