use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveValue::Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Weekly planner task backing record.
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "todos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    #[sea_orm(default_value = "pending")]
    pub status: String,
    pub scheduled_for: Option<Date>,
    #[sea_orm(default_value = 0)]
    pub order_index: i64,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub notes: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: JsonValue,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, _insert: bool) -> Result<Self, sea_orm::DbErr>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now();

        if self.created_at.is_not_set() {
            self.created_at = Set(now);
        }

        self.updated_at = Set(now);

        if self.metadata.is_not_set() {
            self.metadata = Set(JsonValue::Null);
        }

        if self.status.is_not_set() {
            self.status = Set("pending".to_string());
        }

        Ok(self)
    }
}
