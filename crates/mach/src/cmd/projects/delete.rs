use crate::service::Services;

/// Delete a project
#[derive(clap::Args)]
pub struct Args {
    /// Project name or id
    #[clap(required = true)]
    reference: Vec<String>,
}

impl Args {
    pub async fn exec(self, services: &Services) -> miette::Result<()> {
        let reference = self.reference.join(" ");

        let project = services
            .projects
            .find_by_name_or_id(&reference)
            .await?
            .ok_or_else(|| miette::miette!("project '{}' not found", reference))?;

        let name = project.name.clone();

        let deleted = services.projects.delete(project.id).await?;

        if deleted {
            println!("Deleted project '{}'", name);
        } else {
            println!("Project '{}' was already deleted", name);
        }

        Ok(())
    }
}
