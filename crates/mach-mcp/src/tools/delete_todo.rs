use super::traits::McpTool;
use machich::service::todo::TodoService;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteTodoParams {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteTodoResult {
    pub id: String,
    pub deleted: bool,
    pub message: String,
}

pub struct DeleteTodoTool {
    service: TodoService,
}

impl DeleteTodoTool {
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
                    "description": "UUID of the todo to delete"
                }
            },
            "additionalProperties": false
        })
    }

    pub fn description() -> String {
        r#"**Delete Todo** - Permanently delete a todo.

## Parameters
- `id` (required): UUID of the todo to delete

## Warning
This action is permanent and cannot be undone.

## Returns
Confirmation with deleted status."#
            .to_string()
    }

    pub async fn execute(&self, params: DeleteTodoParams) -> Result<DeleteTodoResult> {
        let id = Uuid::parse_str(&params.id)
            .into_diagnostic()
            .map_err(|_| miette::miette!("invalid UUID format"))?;

        let deleted = self.service.delete(id).await?;

        let message = if deleted {
            "Todo deleted".to_string()
        } else {
            "Todo not found".to_string()
        };

        Ok(DeleteTodoResult {
            id: params.id,
            deleted,
            message,
        })
    }
}

impl McpTool for DeleteTodoTool {
    type Params = DeleteTodoParams;
    type Result = DeleteTodoResult;

    fn name() -> &'static str {
        "delete_todo"
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
