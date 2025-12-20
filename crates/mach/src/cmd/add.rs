use crate::service::Services;
use miette::bail;
use uuid::Uuid;

/// Add a new todo
#[derive(clap::Args)]
pub struct Args {
    /// Insert the todo into the backlog
    #[clap(short, long, default_value = "false")]
    some_day: bool,

    /// Workspace name or UUID
    #[clap(short, long)]
    workspace: Option<String>,

    /// Project name or UUID
    #[clap(short, long)]
    project: Option<String>,

    /// Title of the todo (quoted or space separated)
    #[clap(required = true)]
    title: Vec<String>,
}

impl Args {
    pub async fn exec(self, services: &Services) -> miette::Result<()> {
        let scheduled_for = if self.some_day {
            None
        } else {
            Some(services.today())
        };

        let (workspace_id, project_id) =
            resolve_workspace_project(services, self.workspace.as_deref(), self.project.as_deref())
                .await?;

        let todo = services
            .todos
            .add(self.title(), scheduled_for, None, workspace_id, project_id)
            .await?;

        let date_label = scheduled_for
            .map(|d| d.to_string())
            .unwrap_or_else(|| "Someday".into());

        println!("Added todo '{}' -> {}", todo.title, date_label);

        Ok(())
    }

    fn title(&self) -> String {
        self.title.join(" ")
    }
}

async fn resolve_workspace_project(
    services: &Services,
    workspace_arg: Option<&str>,
    project_arg: Option<&str>,
) -> miette::Result<(Option<Uuid>, Option<Uuid>)> {
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
                bail!("project '{}' is not in workspace '{}'", proj, ws);
            }

            Ok((Some(workspace.id), Some(project.id)))
        }
    }
}
