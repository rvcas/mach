pub mod connection;
pub mod todo;

use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use directories::ProjectDirs;
use miette::{Context, IntoDiagnostic};

use self::{connection::init_database, todo::TodoService};

#[derive(Clone)]
pub struct Services {
    pub todos: TodoService,
    today: NaiveDate,
}

impl Services {
    pub async fn bootstrap() -> miette::Result<Self> {
        let db_path = default_db_path()?;

        let conn = init_database(&db_path).await?;

        let todos = TodoService::new(conn.clone());

        let today = Local::now().date_naive();

        todos.rollover_to(today).await?;

        Ok(Self { todos, today })
    }

    pub fn today(&self) -> NaiveDate {
        self.today
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
