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
        let mut condition = Condition::any();
        if let Ok(uuid) = Uuid::parse_str(name_or_id) {
            condition = condition.add(workspace::Column::Id.eq(uuid));
        }
        condition = condition.add(workspace::Column::Name.eq(name_or_id));

        workspace::Entity::find()
            .filter(condition)
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

    pub async fn update_name(&self, id: Uuid, name: impl Into<String>) -> Result<workspace::Model> {
        let model = workspace::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .into_diagnostic()?
            .ok_or_else(|| miette::miette!("workspace not found"))?;

        let mut active: workspace::ActiveModel = model.into();
        active.name = Set(name.into());
        active.update(&self.db).await.into_diagnostic()
    }
}
