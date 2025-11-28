use crate::entity::config;
use chrono::Utc;
use miette::IntoDiagnostic;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    sea_query::OnConflict,
};
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeekStart {
    Sunday,
    Monday,
}

impl WeekStart {
    pub fn toggle(self) -> Self {
        match self {
            WeekStart::Sunday => WeekStart::Monday,
            WeekStart::Monday => WeekStart::Sunday,
        }
    }
}

impl From<&str> for WeekStart {
    fn from(value: &str) -> Self {
        match value {
            "monday" => WeekStart::Monday,
            _ => WeekStart::Sunday,
        }
    }
}

impl WeekStart {
    pub fn as_str(&self) -> &'static str {
        match self {
            WeekStart::Sunday => "sunday",
            WeekStart::Monday => "monday",
        }
    }
}

#[derive(Clone)]
pub struct ConfigService {
    db: DatabaseConnection,
}

impl ConfigService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn load_week_start(&self) -> miette::Result<WeekStart> {
        let result = config::Entity::find()
            .filter(config::Column::Key.eq("week_start"))
            .one(&self.db)
            .await
            .into_diagnostic()?;

        if let Some(model) = result {
            if let Some(value) = model.value.as_str() {
                return Ok(WeekStart::from(value));
            }
        }

        Ok(WeekStart::Sunday)
    }

    pub async fn save_week_start(&self, week_start: WeekStart) -> miette::Result<()> {
        let now = Utc::now();
        let model = config::ActiveModel {
            key: Set("week_start".to_string()),
            value: Set(json!(week_start.as_str())),
            created_at: Set(now),
            updated_at: Set(now),
        };

        config::Entity::insert(model)
            .on_conflict(
                OnConflict::column(config::Column::Key)
                    .update_columns([config::Column::Value, config::Column::UpdatedAt])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .into_diagnostic()?;

        Ok(())
    }
}
