use crate::service::Services;

/// Update a workspace
#[derive(clap::Args)]
pub struct Args {
    /// Workspace id or name
    reference: String,

    /// New name
    #[clap(short, long)]
    name: Option<String>,
}

impl Args {
    pub async fn exec(self, services: &Services) -> miette::Result<()> {
        let workspace = services
            .workspaces
            .find_by_name_or_id(&self.reference)
            .await?
            .ok_or_else(|| miette::miette!("workspace '{}' not found", self.reference))?;

        if self.name.is_none() {
            println!("No updates specified");
            return Ok(());
        }

        let updated = if let Some(name) = self.name {
            services.workspaces.update_name(workspace.id, name).await?
        } else {
            workspace
        };

        println!("Updated workspace '{}'", updated.name);

        Ok(())
    }
}
