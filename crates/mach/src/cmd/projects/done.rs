use crate::service::Services;

/// Mark a project as done
#[derive(clap::Args)]
pub struct Args {
    /// Project id or name
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

        let updated = services.projects.update_status(project.id, "done").await?;

        println!("Marked project '{}' as done", updated.name);

        Ok(())
    }
}
