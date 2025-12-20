pub mod config;
pub mod connection;
pub mod project;
pub mod todo;
pub mod workspace;

use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use directories::ProjectDirs;
use miette::{Context, IntoDiagnostic};

use self::{
    config::{ConfigService, WeekStart},
    connection::init_database,
    project::ProjectService,
    todo::TodoService,
    workspace::WorkspaceService,
};

#[derive(Clone)]
pub struct Services {
    pub todos: TodoService,
    pub config: ConfigService,
    pub workspaces: WorkspaceService,
    pub projects: ProjectService,
    today: NaiveDate,
    week_start_pref: WeekStart,
}

impl Services {
    pub async fn bootstrap() -> miette::Result<Self> {
        let db_path = default_db_path()?;

        let conn = init_database(&db_path).await?;

        let todos = TodoService::new(conn.clone());
        let config = ConfigService::new(conn.clone());
        let workspaces = WorkspaceService::new(conn.clone());
        let projects = ProjectService::new(conn.clone());

        let today = Local::now().date_naive();

        todos.rollover_to(today).await?;
        let week_start = config.load_week_start().await?;

        Ok(Self {
            todos,
            config,
            workspaces,
            projects,
            today,
            week_start_pref: week_start,
        })
    }

    pub fn today(&self) -> NaiveDate {
        self.today
    }

    pub fn week_start(&self) -> WeekStart {
        self.week_start_pref
    }
}

fn default_db_path() -> miette::Result<PathBuf> {
    let dirs = ProjectDirs::from("co.machich", "Orbistry", "mach")
        .ok_or_else(|| miette::miette!("unable to determine data directory"))?;

    let dir = dirs.data_dir();

    std::fs::create_dir_all(dir)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to create data directory {}", dir.display()))?;

    Ok(dir.join("mach.db"))
}
