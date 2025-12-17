use crate::contracts::{CallToolResponse, Content};
use machich::service::todo::TodoService;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkPendingParams {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkPendingResult {
    pub id: String,
    pub title: String,
    pub status: String,
    pub scheduled_for: Option<String>,
    pub message: String,
}

pub struct MarkPendingTool {
    service: TodoService,
}

impl MarkPendingTool {
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
                    "description": "UUID of the todo to mark as pending"
                }
            },
            "additionalProperties": false
        })
    }

    pub fn description() -> String {
        r#"**Mark Pending** - Revert a completed todo back to pending.

## Parameters
- `id` (required): UUID of the todo

## Behavior
- Sets status back to "pending"
- The todo moves to the top of its column

## Returns
Updated todo with new status."#
            .to_string()
    }

    pub async fn call(&self, params: MarkPendingParams) -> Result<CallToolResponse> {
        let result = self.execute(params).await?;
        let json = serde_json::to_string(&result).into_diagnostic()?;
        Ok(CallToolResponse {
            content: vec![Content::text(json)],
        })
    }

    pub async fn execute(&self, params: MarkPendingParams) -> Result<MarkPendingResult> {
        let id = Uuid::parse_str(&params.id)
            .into_diagnostic()
            .map_err(|_| miette::miette!("invalid UUID format"))?;

        let model = self.service.mark_pending(id).await?;

        Ok(MarkPendingResult {
            id: model.id.to_string(),
            title: model.title,
            status: model.status,
            scheduled_for: model
                .scheduled_for
                .map(|d| d.format("%Y-%m-%d").to_string()),
            message: "Todo marked as pending".to_string(),
        })
    }
}
