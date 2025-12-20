use crate::service::{
    Services,
    todo::{ListOptions, ListScope},
};

/// List all todos in a table
#[derive(clap::Args)]
pub struct Args {
    /// List todos in the backlog
    #[clap(short, long, default_value = "false")]
    some_day: bool,

    /// Include completed todos
    #[clap(short, long, default_value = "false")]
    done: bool,

    /// Include the id column
    #[clap(short, long, default_value = "false")]
    id: bool,
}

impl Args {
    pub async fn exec(self, services: &Services) -> miette::Result<()> {
        let scope = if self.some_day {
            ListScope::Backlog
        } else {
            ListScope::Day(services.today())
        };

        let opts = ListOptions {
            scope,
            include_done: self.done,
        };

        let todos = services.todos.list(opts).await?;

        if todos.is_empty() {
            println!("No todos found.");

            return Ok(());
        }

        if self.id {
            println!(
                "{:<38} {:<30} {:<8} {:<15} {:<15} {:<12}",
                "Id", "Title", "Status", "Workspace", "Project", "Day"
            );
            println!("{}", "-".repeat(125));
        } else {
            println!(
                "{:<30} {:<8} {:<15} {:<15} {:<12}",
                "Title", "Status", "Workspace", "Project", "Day"
            );
            println!("{}", "-".repeat(85));
        }

        for todo in todos {
            let day = todo
                .scheduled_for
                .map(|d| d.to_string())
                .unwrap_or_else(|| "Someday".to_string());

            let status = if todo.status == "done" {
                "done"
            } else {
                "pending"
            };

            let workspace_name = match todo.workspace_id {
                Some(id) => services
                    .workspaces
                    .get(id)
                    .await?
                    .map(|w| w.name)
                    .unwrap_or_else(|| "-".to_string()),
                None => "-".to_string(),
            };

            let project_name = match todo.project_id {
                Some(id) => services
                    .projects
                    .get(id)
                    .await?
                    .map(|p| p.name)
                    .unwrap_or_else(|| "-".to_string()),
                None => "-".to_string(),
            };

            if self.id {
                println!(
                    "{:<38} {:<30} {:<8} {:<15} {:<15} {:<12}",
                    todo.id, todo.title, status, workspace_name, project_name, day
                );
            } else {
                println!(
                    "{:<30} {:<8} {:<15} {:<15} {:<12}",
                    todo.title, status, workspace_name, project_name, day
                );
            }
        }

        Ok(())
    }
}
