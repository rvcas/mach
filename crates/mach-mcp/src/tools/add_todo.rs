use crate::contracts::{CallToolResponse, Content};
use chrono::{Local, NaiveDate};
use machich::service::todo::TodoService;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTodoParams {
    pub title: String,
    /// ISO date string (YYYY-MM-DD) or null for backlog
    pub scheduled_for: Option<String>,
    pub notes: Option<String>,
    /// Optional project tag for the todo (e.g., "my-app")
    pub project: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTodoResult {
    pub id: String,
    pub title: String,
    pub status: String,
    pub scheduled_for: Option<String>,
    pub message: String,
}

pub struct AddTodoTool {
    service: TodoService,
}

impl AddTodoTool {
    pub fn new(service: TodoService) -> Self {
        Self { service }
    }

    pub fn schema() -> Value {
        json!({
            "type": "object",
            "required": ["title"],
            "properties": {
                "title": {
                    "type": "string",
                    "minLength": 1,
                    "description": "The title/description of the todo item"
                },
                "scheduledFor": {
                    "type": ["string", "null"],
                    "description": "ISO date (YYYY-MM-DD) to schedule the todo. Omitted = today. Use 'backlog' for someday items."
                },
                "notes": {
                    "type": ["string", "null"],
                    "description": "Optional notes/details for the todo"
                },
                "project": {
                    "type": ["string", "null"],
                    "description": "Optional project tag (e.g., 'my-app'). Will be prefixed as [project] in title if provided."
                }
            },
            "additionalProperties": false
        })
    }

    pub fn description() -> String {
        r#"**Add Todo** - Create a new todo item.

## Parameters
- `title` (required): The todo title/description
- `scheduledFor` (optional): ISO date (YYYY-MM-DD) to schedule. Omit for today's date
- `notes` (optional): Additional notes
- `project` (optional): Project tag to prefix the title with [project]

## Examples
```json
// Schedule for today (default)
{ "title": "Fix login bug" }

// With project tag - title becomes "[my-app] Write tests"
{ "title": "Write tests", "scheduledFor": "2024-01-15", "project": "my-app" }

// Add to backlog
{ "title": "Refactor auth", "scheduledFor": "backlog" }
```

## Returns
Created todo with id, title, status, and scheduled date."#
            .to_string()
    }

    pub async fn call(&self, params: AddTodoParams) -> Result<CallToolResponse> {
        let result = self.execute(params).await?;
        let json = serde_json::to_string(&result).into_diagnostic()?;
        Ok(CallToolResponse {
            content: vec![Content::text(json)],
        })
    }

    pub async fn execute(&self, params: AddTodoParams) -> Result<AddTodoResult> {
        let scheduled_for = match params.scheduled_for.as_deref() {
            Some("backlog") | Some("") => None, // Explicit backlog
            Some(date_str) => Some(parse_date(date_str)?),
            None => Some(Local::now().date_naive()), // Default to today
        };

        // Optionally prefix title with [project] if project is provided
        let title = match &params.project {
            Some(proj) if !params.title.starts_with('[') => format!("[{}] {}", proj, params.title),
            _ => params.title.clone(),
        };

        let model = self
            .service
            .add(&title, scheduled_for, params.notes, None, None)
            .await?;

        let location = match model.scheduled_for {
            Some(date) => {
                let today = Local::now().date_naive();
                if date == today {
                    "today".to_string()
                } else {
                    date.format("%Y-%m-%d").to_string()
                }
            }
            None => "backlog".to_string(),
        };

        Ok(AddTodoResult {
            id: model.id.to_string(),
            title: model.title,
            status: model.status,
            scheduled_for: model
                .scheduled_for
                .map(|d| d.format("%Y-%m-%d").to_string()),
            message: format!("Todo added to {}", location),
        })
    }
}

fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
        .into_diagnostic()
        .map_err(|_| miette::miette!("invalid date format, expected YYYY-MM-DD"))
}
