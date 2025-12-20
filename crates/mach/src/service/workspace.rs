use crate::entity::workspace;
use miette::{IntoDiagnostic, Result};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct WorkspaceService {
    db: DatabaseConnection,
}

impl WorkspaceService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_name_or_id(&self, name_or_id: &str) -> Result<Option<workspace::Model>> {
        workspace::Entity::find()
            .filter(
                Condition::any()
                    .add(workspace::Column::Id.eq(name_or_id))
                    .add(workspace::Column::Name.eq(name_or_id)),
            )
            .one(&self.db)
            .await
            .into_diagnostic()
    }

    pub async fn create(&self, name: impl Into<String>) -> Result<workspace::Model> {
        let model = workspace::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name.into()),
            ..Default::default()
        };

        model.insert(&self.db).await.into_diagnostic()
    }

    pub async fn list(&self) -> Result<Vec<workspace::Model>> {
        workspace::Entity::find()
            .all(&self.db)
            .await
            .into_diagnostic()
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<workspace::Model>> {
        workspace::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .into_diagnostic()
    }
}
