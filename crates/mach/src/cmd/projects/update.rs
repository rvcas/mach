use crate::service::Services;

use super::ProjectStatus;

/// Update a project
#[derive(clap::Args)]
pub struct Args {
    /// Project id or name
    reference: String,

    /// New name
    #[clap(short, long)]
    name: Option<String>,

    /// New status
    #[clap(short, long)]
    status: Option<ProjectStatus>,
}

impl Args {
    pub async fn exec(self, services: &Services) -> miette::Result<()> {
        let project = services
            .projects
            .find_by_name_or_id(&self.reference)
            .await?
            .ok_or_else(|| miette::miette!("project '{}' not found", self.reference))?;

        if self.name.is_none() && self.status.is_none() {
            println!("No updates specified");
            return Ok(());
        }

        let mut updated = project;

        if let Some(name) = self.name {
            updated = services.projects.update_name(updated.id, name).await?;
        }

        if let Some(status) = self.status {
            updated = services
                .projects
                .update_status(updated.id, status.as_str())
                .await?;
        }

        println!("Updated project '{}'", updated.name);

        Ok(())
    }
}
