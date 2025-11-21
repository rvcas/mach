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

        println!("{:<8} {:<12} Title", "Status", "Day");
        println!("{}", "-".repeat(48));

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

            println!("{:<8} {:<12} {}", status, day, todo.title);
        }

        Ok(())
    }
}
