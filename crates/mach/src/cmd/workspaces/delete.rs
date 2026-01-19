use crate::service::Services;

/// Delete a workspace
#[derive(clap::Args)]
pub struct Args {
    /// Workspace name or id
    #[clap(required = true)]
    reference: Vec<String>,
}

impl Args {
    pub async fn exec(self, services: &Services) -> miette::Result<()> {
        let reference = self.reference.join(" ");

        let workspace = services
            .workspaces
            .find_by_name_or_id(&reference)
            .await?
            .ok_or_else(|| miette::miette!("workspace '{}' not found", reference))?;

        let name = workspace.name.clone();

        let deleted = services.workspaces.delete(workspace.id).await?;

        if deleted {
            println!("Deleted workspace '{}'", name);
        } else {
            println!("Workspace '{}' was already deleted", name);
        }

        Ok(())
    }
}
