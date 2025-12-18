use super::traits::McpTool;
use machich::service::todo::TodoService;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTodoParams {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTodoResult {
    pub id: String,
    pub title: String,
    pub status: String,
    pub scheduled_for: Option<String>,
    pub notes: Option<String>,
    pub order_index: i64,
    pub backlog_column: i64,
    pub project: Option<String>,
    pub epic_id: Option<String>,
    pub epic_title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct GetTodoTool {
    service: TodoService,
}

impl GetTodoTool {
    pub fn new(service: TodoService) -> Self {
        Self { service }
    }

    pub fn schema() -> Value {
        json!({
            "type": "object",
            "required": ["id"],
            "properties": {
                "id": {
                    "type": "string",
                    "description": "UUID of the todo to retrieve"
                }
            },
            "additionalProperties": false
        })
    }

    pub fn description() -> String {
        r#"**Get Todo** - Retrieve a single todo by ID.

## Parameters
- `id` (required): UUID of the todo

## Returns
Full todo details including id, title, status, scheduledFor, notes, orderIndex, backlogColumn, project, epicId, epicTitle, createdAt, updatedAt."#
            .to_string()
    }

    pub async fn execute(&self, params: GetTodoParams) -> Result<GetTodoResult> {
        let id = Uuid::parse_str(&params.id)
            .into_diagnostic()
            .map_err(|_| miette::miette!("invalid UUID format"))?;

        let model = self.service.get(id).await?;

        let epic_title = if let Some(eid) = model.epic_id {
            self.service.get_epic_title(eid).await.ok()
        } else {
            None
        };

        Ok(GetTodoResult {
            id: model.id.to_string(),
            title: model.title,
            status: model.status,
            scheduled_for: model
                .scheduled_for
                .map(|d| d.format("%Y-%m-%d").to_string()),
            notes: model.notes,
            order_index: model.order_index,
            backlog_column: model.backlog_column,
            project: model.project,
            epic_id: model.epic_id.map(|u| u.to_string()),
            epic_title,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        })
    }
}

impl McpTool for GetTodoTool {
    type Params = GetTodoParams;
    type Result = GetTodoResult;

    fn name() -> &'static str {
        "get_todo"
    }

    fn schema() -> Value {
        Self::schema()
    }

    fn description() -> String {
        Self::description()
    }

    async fn run(&self, params: Self::Params) -> Result<Self::Result> {
        self.execute(params).await
    }
}
