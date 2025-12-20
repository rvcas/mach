use crate::entity::project;
use miette::{IntoDiagnostic, Result};
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
        project::Entity::find()
            .filter(
                Condition::any()
                    .add(project::Column::Id.eq(name_or_id))
                    .add(project::Column::Name.eq(name_or_id)),
            )
            .one(&self.db)
            .await
            .into_diagnostic()
    }

    pub async fn create(
        &self,
        name: impl Into<String>,
        workspace_id: Uuid,
        status: impl Into<String>,
    ) -> Result<project::Model> {
        let model = project::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name.into()),
            workspace_id: Set(workspace_id),
            status: Set(status.into()),
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
}
