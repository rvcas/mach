use crate::contracts::{CallToolResponse, Content};
use chrono::NaiveDate;
use machich::service::todo::TodoService;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTodoParams {
    pub id: String,
    pub title: Option<String>,
    /// ISO date or null to move to backlog
    pub scheduled_for: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTodoResult {
    pub id: String,
    pub title: String,
    pub status: String,
    pub scheduled_for: Option<String>,
    pub notes: Option<String>,
    pub message: String,
}

pub struct UpdateTodoTool {
    service: TodoService,
}

impl UpdateTodoTool {
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
                    "description": "UUID of the todo to update"
                },
                "title": {
                    "type": ["string", "null"],
                    "description": "New title (if provided)"
                },
                "scheduledFor": {
                    "type": ["string", "null"],
                    "description": "New scheduled date (YYYY-MM-DD) or null to move to backlog"
                },
                "notes": {
                    "type": ["string", "null"],
                    "description": "New notes (if provided)"
                }
            },
            "additionalProperties": false
        })
    }

    pub fn description() -> String {
        r#"**Update Todo** - Update a todo's title, date, or notes.

## Parameters
- `id` (required): UUID of the todo
- `title` (optional): New title
- `scheduledFor` (optional): New date (YYYY-MM-DD) or null for backlog
- `notes` (optional): New notes

## Examples
```json
// Update title
{ "id": "uuid-here", "title": "New title" }

// Reschedule to tomorrow
{ "id": "uuid-here", "scheduledFor": "2024-01-16" }

// Move to backlog
{ "id": "uuid-here", "scheduledFor": null }
```"#
            .to_string()
    }

    pub async fn call(&self, params: UpdateTodoParams) -> Result<CallToolResponse> {
        let result = self.execute(params).await?;
        let json = serde_json::to_string(&result).into_diagnostic()?;
        Ok(CallToolResponse {
            content: vec![Content::text(json)],
        })
    }

    pub async fn execute(&self, params: UpdateTodoParams) -> Result<UpdateTodoResult> {
        let id = Uuid::parse_str(&params.id)
            .into_diagnostic()
            .map_err(|_| miette::miette!("invalid UUID format"))?;

        let mut model = self.service.get(id).await?;
        let mut changes = Vec::new();

        if let Some(title) = params.title {
            model = self.service.update_title(id, title).await?;
            changes.push("title");
        }

        if let Some(date_str) = params.scheduled_for.as_ref() {
            let scheduled = if date_str == "null" || date_str.is_empty() {
                None
            } else {
                Some(parse_date(date_str)?)
            };
            model = self.service.update_scheduled_for(id, scheduled).await?;
            changes.push("scheduled_for");
        }

        if let Some(notes) = params.notes {
            let notes_opt = if notes.is_empty() { None } else { Some(notes) };
            model = self.service.update_notes(id, notes_opt).await?;
            changes.push("notes");
        }

        let change_desc = if changes.is_empty() {
            "no changes".to_string()
        } else {
            changes.join(", ")
        };

        Ok(UpdateTodoResult {
            id: model.id.to_string(),
            title: model.title,
            status: model.status,
            scheduled_for: model
                .scheduled_for
                .map(|d| d.format("%Y-%m-%d").to_string()),
            notes: model.notes,
            message: format!("Updated: {}", change_desc),
        })
    }
}

fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
        .into_diagnostic()
        .map_err(|_| miette::miette!("invalid date format, expected YYYY-MM-DD"))
}
