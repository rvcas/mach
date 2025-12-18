use miette::Diagnostic;
use thiserror::Error;
use uuid::Uuid;

/// Domain errors for todo operations.
///
/// These typed errors enable robust error classification at the MCP layer:
/// - `NotFound`, `InvalidUuid`, `ProjectMismatch`, `SelfReference` → invalid_params
/// - `HasChildren`, `Database` → internal_error
#[derive(Debug, Error, Diagnostic)]
pub enum TodoError {
    #[error("todo {0} not found")]
    #[diagnostic(code(todo::not_found))]
    NotFound(Uuid),

    #[error("epic {0} not found")]
    #[diagnostic(code(todo::epic_not_found))]
    EpicNotFound(Uuid),

    #[error("invalid UUID format")]
    #[diagnostic(code(todo::invalid_uuid))]
    InvalidUuid,

    #[error("project '{0}' does not match epic's project '{1}'")]
    #[diagnostic(code(todo::project_mismatch))]
    ProjectMismatch(String, String),

    #[error("cannot delete todo: it is an epic with {0} sub-todo(s)")]
    #[diagnostic(code(todo::has_children))]
    HasChildren(u64),

    #[error("a todo cannot be its own epic")]
    #[diagnostic(code(todo::self_reference))]
    SelfReference,

    #[error("database error: {0}")]
    #[diagnostic(code(todo::database))]
    Database(#[from] sea_orm::DbErr),
}

impl TodoError {
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            TodoError::NotFound(_)
                | TodoError::EpicNotFound(_)
                | TodoError::InvalidUuid
                | TodoError::ProjectMismatch(_, _)
                | TodoError::SelfReference
        )
    }
}
