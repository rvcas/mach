use super::traits::McpTool;
use super::util::parse_date;
use chrono::Local;
use machich::service::todo::TodoService;
use miette::Result;
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
    /// Optional epic UUID to link this todo under
    pub epic_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTodoResult {
    pub id: String,
    pub title: String,
    pub status: String,
    pub scheduled_for: Option<String>,
    pub project: Option<String>,
    pub epic_id: Option<String>,
    pub epic_title: Option<String>,
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
                    "description": "Optional project tag (e.g., 'my-app')."
                },
                "epicId": {
                    "type": ["string", "null"],
                    "description": "UUID of an existing todo to use as this todo's epic (parent). The todo inherits the epic's project if not specified."
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
- `project` (optional): Project tag (e.g., 'my-app')
- `epicId` (optional): UUID of parent todo (epic) to link under

## Examples
```json
// Schedule for today (default)
{ "title": "Fix login bug" }

// With project tag
{ "title": "Write tests", "scheduledFor": "2024-01-15", "project": "my-app" }

// Add to backlog
{ "title": "Refactor auth", "scheduledFor": "backlog" }

// Link to an epic
{ "title": "Implement login form", "epicId": "uuid-of-epic" }
```

## Creating Epics
Epics are regular todos that other todos link to via `epicId`.
To create an epic, just create a normal todo (usually with detailed notes).

**Important**: Do NOT automatically close epics. Let users manually close them
when they decide the epic is complete.

## Returns
Created todo with id, title, status, scheduledFor, project, epicId."#
            .to_string()
    }

    pub async fn execute(&self, params: AddTodoParams) -> Result<AddTodoResult> {
        let scheduled_for = match params.scheduled_for.as_deref() {
            Some("backlog") | Some("") => None,
            Some(date_str) => Some(parse_date(date_str)?),
            None => Some(Local::now().date_naive()),
        };

        let epic_uuid = params
            .epic_id
            .as_ref()
            .map(|s| uuid::Uuid::parse_str(s))
            .transpose()
            .map_err(|_| miette::miette!("invalid epic_id UUID format"))?;

        let model = self
            .service
            .add(&params.title, scheduled_for, params.notes, params.project.clone(), epic_uuid)
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

        let epic_title = if let Some(eid) = model.epic_id {
            self.service.get_epic_title(eid).await.ok()
        } else {
            None
        };

        Ok(AddTodoResult {
            id: model.id.to_string(),
            title: model.title,
            status: model.status,
            scheduled_for: model
                .scheduled_for
                .map(|d| d.format("%Y-%m-%d").to_string()),
            project: model.project,
            epic_id: model.epic_id.map(|u| u.to_string()),
            epic_title,
            message: format!("Todo added to {}", location),
        })
    }
}

impl McpTool for AddTodoTool {
    type Params = AddTodoParams;
    type Result = AddTodoResult;

    fn name() -> &'static str {
        "add_todo"
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
