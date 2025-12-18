use super::traits::McpTool;
use chrono::Local;
use machich::service::todo::TodoService;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkDoneParams {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkDoneResult {
    pub id: String,
    pub title: String,
    pub status: String,
    pub scheduled_for: Option<String>,
    pub project: Option<String>,
    pub epic_id: Option<String>,
    pub epic_title: Option<String>,
    pub message: String,
}

pub struct MarkDoneTool {
    service: TodoService,
}

impl MarkDoneTool {
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
                    "description": "UUID of the todo to mark as done"
                }
            },
            "additionalProperties": false
        })
    }

    pub fn description() -> String {
        r#"**Mark Done** - Mark a todo as completed.

## Parameters
- `id` (required): UUID of the todo

## Behavior
- Sets status to "done"
- If the todo was in backlog, it moves to today's date
- The todo moves to the bottom of its column

## Returns
Updated todo with new status."#
            .to_string()
    }

    pub async fn execute(&self, params: MarkDoneParams) -> Result<MarkDoneResult> {
        let id = Uuid::parse_str(&params.id)
            .into_diagnostic()
            .map_err(|_| miette::miette!("invalid UUID format"))?;

        let today = Local::now().date_naive();
        let model = self.service.mark_done(id, today).await?;

        let epic_title = if let Some(eid) = model.epic_id {
            self.service.get_epic_title(eid).await.ok()
        } else {
            None
        };

        Ok(MarkDoneResult {
            id: model.id.to_string(),
            title: model.title,
            status: model.status,
            scheduled_for: model
                .scheduled_for
                .map(|d| d.format("%Y-%m-%d").to_string()),
            project: model.project,
            epic_id: model.epic_id.map(|u| u.to_string()),
            epic_title,
            message: "Todo marked as done".to_string(),
        })
    }
}

impl McpTool for MarkDoneTool {
    type Params = MarkDoneParams;
    type Result = MarkDoneResult;

    fn name() -> &'static str {
        "mark_done"
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
