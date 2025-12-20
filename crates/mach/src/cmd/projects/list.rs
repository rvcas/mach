use crate::service::Services;

/// List projects
#[derive(clap::Args)]
pub struct Args {
    /// Filter by workspace name or UUID
    #[clap(short, long)]
    workspace: Option<String>,

    /// Include the id column
    #[clap(short, long, default_value = "false")]
    id: bool,
}

impl Args {
    pub async fn exec(self, services: &Services) -> miette::Result<()> {
        let projects = match &self.workspace {
            Some(ws) => {
                let workspace = services
                    .workspaces
                    .find_by_name_or_id(ws)
                    .await?
                    .ok_or_else(|| miette::miette!("workspace '{}' not found", ws))?;

                services.projects.list_by_workspace(workspace.id).await?
            }
            None => services.projects.list().await?,
        };

        if projects.is_empty() {
            println!("No projects found");
            return Ok(());
        }

        if self.id {
            println!(
                "{:<38} {:<20} {:<10} {:>6} {:>9} {:>9} {:>12} {:>12}",
                "id", "name", "status", "todos", "completed", "remaining", "created", "updated"
            );
            println!("{}", "-".repeat(120));
        } else {
            println!(
                "{:<20} {:<10} {:>6} {:>9} {:>9} {:>12} {:>12}",
                "name", "status", "todos", "completed", "remaining", "created", "updated"
            );
            println!("{}", "-".repeat(82));
        }

        for project in projects {
            let stats = services.todos.stats_for_project(project.id).await?;
            let created = project.created_at.format("%Y-%m-%d").to_string();
            let updated = project.updated_at.format("%Y-%m-%d").to_string();

            if self.id {
                println!(
                    "{:<38} {:<20} {:<10} {:>6} {:>9} {:>9} {:>12} {:>12}",
                    project.id,
                    project.name,
                    project.status,
                    stats.total,
                    stats.completed,
                    stats.remaining,
                    created,
                    updated
                );
            } else {
                println!(
                    "{:<20} {:<10} {:>6} {:>9} {:>9} {:>12} {:>12}",
                    project.name,
                    project.status,
                    stats.total,
                    stats.completed,
                    stats.remaining,
                    created,
                    updated
                );
            }
        }

        Ok(())
    }
}
