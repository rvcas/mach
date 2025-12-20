use crate::service::Services;

/// Create a new workspace
#[derive(clap::Args)]
pub struct Args {
    /// Name of the workspace (quoted or space separated)
    #[clap(required = true)]
    name: Vec<String>,
}

impl Args {
    pub async fn exec(self, services: &Services) -> miette::Result<()> {
        let name = self.name.join(" ");

        let workspace = services.workspaces.create(&name).await?;

        println!("Created workspace '{}'", workspace.name);

        Ok(())
    }
}
