use crate::service::Services;

/// Update a todo
#[derive(clap::Args)]
pub struct Args {
    /// Todo id or title
    reference: String,

    /// New title
    #[clap(short, long)]
    title: Option<String>,

    /// New scheduled date (YYYY-MM-DD or "none"/"someday" to clear)
    #[clap(short, long)]
    day: Option<String>,

    /// New notes
    #[clap(short, long)]
    notes: Option<String>,

    /// Workspace name or UUID
    #[clap(short, long)]
    workspace: Option<String>,

    /// Project name or UUID
    #[clap(short, long)]
    project: Option<String>,
}

impl Args {
    pub async fn exec(self, services: &Services) -> miette::Result<()> {
        let todo = services
            .todos
            .find_by_title_or_id(&self.reference)
            .await?
            .ok_or_else(|| miette::miette!("todo '{}' not found", self.reference))?;

        let mut updated = todo.clone();

        if let Some(title) = self.title {
            updated = services.todos.update_title(updated.id, title).await?;
        }

        if let Some(day) = self.day {
            let date = parse_scheduled_for(&day)?;
            updated = services
                .todos
                .update_scheduled_for(updated.id, date)
                .await?;
        }

        if let Some(notes) = self.notes {
            let notes = if notes.is_empty() { None } else { Some(notes) };
            updated = services.todos.update_notes(updated.id, notes).await?;
        }

        if self.workspace.is_some() || self.project.is_some() {
            let (workspace_id, project_id) = resolve_workspace_project(
                services,
                self.workspace.as_deref(),
                self.project.as_deref(),
            )
            .await?;
            updated = services
                .todos
                .update_workspace_project(updated.id, workspace_id, project_id)
                .await?;
        }

        println!("Updated '{}'", updated.title);

        Ok(())
    }
}

fn parse_scheduled_for(s: &str) -> miette::Result<Option<chrono::NaiveDate>> {
    let s = s.trim().to_lowercase();
    if s == "none" || s == "someday" {
        return Ok(None);
    }
    chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| miette::miette!("invalid date format, use YYYY-MM-DD"))
}

async fn resolve_workspace_project(
    services: &Services,
    workspace_arg: Option<&str>,
    project_arg: Option<&str>,
) -> miette::Result<(Option<uuid::Uuid>, Option<uuid::Uuid>)> {
    match (workspace_arg, project_arg) {
        (None, None) => Ok((None, None)),

        (Some(ws), None) => {
            let workspace = services
                .workspaces
                .find_by_name_or_id(ws)
                .await?
                .ok_or_else(|| miette::miette!("workspace '{}' not found", ws))?;

            Ok((Some(workspace.id), None))
        }

        (None, Some(proj)) => {
            let project = services
                .projects
                .find_by_name_or_id(proj)
                .await?
                .ok_or_else(|| miette::miette!("project '{}' not found", proj))?;

            Ok((Some(project.workspace_id), Some(project.id)))
        }

        (Some(ws), Some(proj)) => {
            let workspace = services
                .workspaces
                .find_by_name_or_id(ws)
                .await?
                .ok_or_else(|| miette::miette!("workspace '{}' not found", ws))?;

            let project = services
                .projects
                .find_by_name_or_id(proj)
                .await?
                .ok_or_else(|| miette::miette!("project '{}' not found", proj))?;

            if project.workspace_id != workspace.id {
                miette::bail!("project '{}' is not in workspace '{}'", proj, ws);
            }

            Ok((Some(workspace.id), Some(project.id)))
        }
    }
}
