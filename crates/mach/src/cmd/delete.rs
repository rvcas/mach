use crate::service::Services;

/// Delete a todo
#[derive(clap::Args)]
pub struct Args {
    /// Todo id or title
    #[clap(required = true)]
    reference: Vec<String>,
}

impl Args {
    pub async fn exec(self, services: &Services) -> miette::Result<()> {
        let reference = self.reference.join(" ");

        let todo = services
            .todos
            .find_by_title_or_id(&reference)
            .await?
            .ok_or_else(|| miette::miette!("todo '{}' not found", reference))?;

        let title = todo.title.clone();

        services.todos.delete(todo.id).await?;

        println!("Deleted '{}'", title);

        Ok(())
    }
}
