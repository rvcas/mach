use super::traits::McpTool;
use super::util::parse_date;
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
    pub scheduled_for: Option<String>,
    pub notes: Option<String>,
    pub project: Option<String>,
    pub epic_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTodoResult {
    pub id: String,
    pub title: String,
    pub status: String,
    pub scheduled_for: Option<String>,
    pub notes: Option<String>,
    pub project: Option<String>,
    pub epic_id: Option<String>,
    pub epic_title: Option<String>,
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
                    "type": "string",
                    "description": "New title (if provided)"
                },
                "scheduledFor": {
                    "type": "string",
                    "description": "New scheduled date (YYYY-MM-DD) or null to move to backlog"
                },
                "notes": {
                    "type": "string",
                    "description": "New notes (if provided)"
                },
                "project": {
                    "type": "string",
                    "description": "New project tag, or empty string/\"null\" to clear"
                },
                "epicId": {
                    "type": "string",
                    "description": "UUID of epic to link under, or empty string/\"null\" to clear"
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
- `project` (optional): New project tag, or empty string/"null" to clear
- `epicId` (optional): UUID of epic to link under, or empty string/"null" to clear

## Examples
```json
// Update title
{ "id": "uuid-here", "title": "New title" }

// Reschedule to tomorrow
{ "id": "uuid-here", "scheduledFor": "2024-01-16" }

// Move to backlog
{ "id": "uuid-here", "scheduledFor": null }

// Set project
{ "id": "uuid-here", "project": "my-app" }

// Clear epic link
{ "id": "uuid-here", "epicId": "null" }
```"#
            .to_string()
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

        if let Some(proj) = params.project {
            let p = if proj.is_empty() || proj == "null" { None } else { Some(proj) };
            model = self.service.update_project(id, p).await?;
            changes.push("project");
        }

        if let Some(eid_str) = params.epic_id {
            let eid = if eid_str.is_empty() || eid_str == "null" {
                None
            } else {
                Some(Uuid::parse_str(&eid_str).map_err(|_| miette::miette!("invalid epic_id UUID"))?)
            };
            model = self.service.update_epic_id(id, eid).await?;
            changes.push("epic_id");
        }

        let change_desc = if changes.is_empty() {
            "no changes".to_string()
        } else {
            changes.join(", ")
        };

        let epic_title = if let Some(eid) = model.epic_id {
            self.service.get_epic_title(eid).await.ok()
        } else {
            None
        };

        Ok(UpdateTodoResult {
            id: model.id.to_string(),
            title: model.title,
            status: model.status,
            scheduled_for: model
                .scheduled_for
                .map(|d| d.format("%Y-%m-%d").to_string()),
            notes: model.notes,
            project: model.project,
            epic_id: model.epic_id.map(|u| u.to_string()),
            epic_title,
            message: format!("Updated: {}", change_desc),
        })
    }
}

impl McpTool for UpdateTodoTool {
    type Params = UpdateTodoParams;
    type Result = UpdateTodoResult;

    fn name() -> &'static str {
        "update_todo"
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
