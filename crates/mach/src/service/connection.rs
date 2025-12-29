use std::path::Path;

use miette::{Context, IntoDiagnostic};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use tokio::fs;
use tokio::fs::OpenOptions;

pub async fn init_database(path: impl AsRef<Path>) -> miette::Result<DatabaseConnection> {
    let path = path.as_ref();

    ensure_parent_dir(path).await?;

    let path_string = path_to_string(path);

    if !path.exists() {
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .await
            .into_diagnostic()
            .wrap_err("failed to create sqlite db file")?;
    }

    let url = format!("sqlite://{path_string}?mode=rwc");

    let conn = Database::connect(&url)
        .await
        .into_diagnostic()
        .wrap_err("failed to open SeaORM SQLite connection")?;

    conn.execute_unprepared("PRAGMA journal_mode=WAL")
        .await
        .into_diagnostic()
        .wrap_err("failed to set journal_mode")?;

    conn.execute_unprepared("PRAGMA busy_timeout=5000")
        .await
        .into_diagnostic()
        .wrap_err("failed to set busy_timeout")?;

    conn.get_schema_registry("machich::entity::*")
        .sync(&conn)
        .await
        .into_diagnostic()
        .wrap_err("failed to synchronize schema via SeaORM entity registry")?;

    Ok(conn)
}

fn path_to_string(path: &Path) -> String {
    path.to_path_buf()
        .into_os_string()
        .to_string_lossy()
        .into_owned()
}

async fn ensure_parent_dir(path: &Path) -> miette::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to create parent directory {}", parent.display()))?;
    }

    Ok(())
}
