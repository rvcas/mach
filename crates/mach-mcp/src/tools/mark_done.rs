use crate::contracts::{CallToolResponse, Content};
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

    pub async fn call(&self, params: MarkDoneParams) -> Result<CallToolResponse> {
        let result = self.execute(params).await?;
        let json = serde_json::to_string(&result).into_diagnostic()?;
        Ok(CallToolResponse {
            content: vec![Content::text(json)],
        })
    }

    pub async fn execute(&self, params: MarkDoneParams) -> Result<MarkDoneResult> {
        let id = Uuid::parse_str(&params.id)
            .into_diagnostic()
            .map_err(|_| miette::miette!("invalid UUID format"))?;

        let today = Local::now().date_naive();
        let model = self.service.mark_done(id, today).await?;

        Ok(MarkDoneResult {
            id: model.id.to_string(),
            title: model.title,
            status: model.status,
            scheduled_for: model
                .scheduled_for
                .map(|d| d.format("%Y-%m-%d").to_string()),
            message: "Todo marked as done".to_string(),
        })
    }
}
