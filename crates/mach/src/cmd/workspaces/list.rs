use crate::service::Services;

/// List all workspaces
#[derive(clap::Args)]
pub struct Args {
    /// Include the id column
    #[clap(short, long, default_value = "false")]
    id: bool,
}

impl Args {
    pub async fn exec(self, services: &Services) -> miette::Result<()> {
        let workspaces = services.workspaces.list().await?;

        if workspaces.is_empty() {
            println!("No workspaces found");
            return Ok(());
        }

        if self.id {
            println!(
                "{:<38} {:<20} {:>8} {:>6} {:>9} {:>9} {:>12} {:>12}",
                "id", "name", "projects", "todos", "completed", "remaining", "created", "updated"
            );
            println!("{}", "-".repeat(130));
        } else {
            println!(
                "{:<20} {:>8} {:>6} {:>9} {:>9} {:>12} {:>12}",
                "name", "projects", "todos", "completed", "remaining", "created", "updated"
            );
            println!("{}", "-".repeat(90));
        }

        for workspace in workspaces {
            let project_count = services.projects.count_by_workspace(workspace.id).await?;
            let stats = services.todos.stats_for_workspace(workspace.id).await?;
            let created = workspace.created_at.format("%Y-%m-%d").to_string();
            let updated = workspace.updated_at.format("%Y-%m-%d").to_string();

            if self.id {
                println!(
                    "{:<38} {:<20} {:>8} {:>6} {:>9} {:>9} {:>12} {:>12}",
                    workspace.id,
                    workspace.name,
                    project_count,
                    stats.total,
                    stats.completed,
                    stats.remaining,
                    created,
                    updated
                );
            } else {
                println!(
                    "{:<20} {:>8} {:>6} {:>9} {:>9} {:>12} {:>12}",
                    workspace.name,
                    project_count,
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
