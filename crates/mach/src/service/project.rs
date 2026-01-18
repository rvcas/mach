use crate::entity::{project, todo};
use miette::{IntoDiagnostic, Result, bail};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, Set,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct ProjectService {
    db: DatabaseConnection,
}

impl ProjectService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_name_or_id(&self, name_or_id: &str) -> Result<Option<project::Model>> {
        let mut condition = Condition::any();
        if let Ok(uuid) = Uuid::parse_str(name_or_id) {
            condition = condition.add(project::Column::Id.eq(uuid));
        }
        condition = condition.add(project::Column::Name.eq(name_or_id));

        project::Entity::find()
            .filter(condition)
            .one(&self.db)
            .await
            .into_diagnostic()
    }

    pub async fn create(
        &self,
        name: impl Into<String>,
        workspace_id: Uuid,
        status: impl Into<String>,
        description: Option<String>,
    ) -> Result<project::Model> {
        let model = project::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name.into()),
            workspace_id: Set(workspace_id),
            status: Set(status.into()),
            description: Set(description),
            ..Default::default()
        };

        model.insert(&self.db).await.into_diagnostic()
    }

    pub async fn list(&self) -> Result<Vec<project::Model>> {
        project::Entity::find()
            .all(&self.db)
            .await
            .into_diagnostic()
    }

    pub async fn list_by_workspace(&self, workspace_id: Uuid) -> Result<Vec<project::Model>> {
        project::Entity::find()
            .filter(project::Column::WorkspaceId.eq(workspace_id))
            .all(&self.db)
            .await
            .into_diagnostic()
    }

    pub async fn count_by_workspace(&self, workspace_id: Uuid) -> Result<u64> {
        project::Entity::find()
            .filter(project::Column::WorkspaceId.eq(workspace_id))
            .count(&self.db)
            .await
            .into_diagnostic()
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<project::Model>> {
        project::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .into_diagnostic()
    }

    pub async fn update_name(&self, id: Uuid, name: impl Into<String>) -> Result<project::Model> {
        let model = project::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .into_diagnostic()?
            .ok_or_else(|| miette::miette!("project not found"))?;

        let mut active: project::ActiveModel = model.into();
        active.name = Set(name.into());
        active.update(&self.db).await.into_diagnostic()
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: impl Into<String>,
    ) -> Result<project::Model> {
        let model = project::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .into_diagnostic()?
            .ok_or_else(|| miette::miette!("project not found"))?;

        let mut active: project::ActiveModel = model.into();
        active.status = Set(status.into());
        active.update(&self.db).await.into_diagnostic()
    }

    pub async fn update_description(
        &self,
        id: Uuid,
        description: Option<String>,
    ) -> Result<project::Model> {
        let model = project::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .into_diagnostic()?
            .ok_or_else(|| miette::miette!("project not found"))?;

        let mut active: project::ActiveModel = model.into();
        active.description = Set(description);
        active.update(&self.db).await.into_diagnostic()
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let project = project::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .into_diagnostic()?
            .ok_or_else(|| miette::miette!("project not found"))?;

        let todo_count = todo::Entity::find()
            .filter(todo::Column::ProjectId.eq(id))
            .count(&self.db)
            .await
            .into_diagnostic()?;

        if todo_count > 0 {
            bail!(
                "cannot delete project '{}': has {} todo(s). Delete or move them first.",
                project.name,
                todo_count
            );
        }

        let res = project::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .into_diagnostic()?;

        Ok(res.rows_affected > 0)
    }
}
