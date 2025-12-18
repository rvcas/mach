use super::traits::McpTool;
use super::util::parse_scope;
use chrono::Local;
use machich::service::todo::{MovePlacement, TodoService};
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveTodoParams {
    pub id: String,
    /// "today", "backlog", or ISO date (YYYY-MM-DD)
    pub scope: String,
    /// "top" or "bottom" (default: "top")
    pub placement: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveTodoResult {
    pub id: String,
    pub title: String,
    pub status: String,
    pub scheduled_for: Option<String>,
    pub project: Option<String>,
    pub epic_id: Option<String>,
    pub epic_title: Option<String>,
    pub message: String,
}

pub struct MoveTodoTool {
    service: TodoService,
}

impl MoveTodoTool {
    pub fn new(service: TodoService) -> Self {
        Self { service }
    }

    pub fn schema() -> Value {
        json!({
            "type": "object",
            "required": ["id", "scope"],
            "properties": {
                "id": {
                    "type": "string",
                    "description": "UUID of the todo to move"
                },
                "scope": {
                    "type": "string",
                    "description": "Target: 'today', 'backlog', or ISO date (YYYY-MM-DD)"
                },
                "placement": {
                    "type": ["string", "null"],
                    "enum": ["top", "bottom", null],
                    "description": "Where to place in the target column (default: 'top')"
                }
            },
            "additionalProperties": false
        })
    }

    pub fn description() -> String {
        r#"**Move Todo** - Move a todo to a different day or backlog.

## Parameters
- `id` (required): UUID of the todo
- `scope` (required): "today", "backlog", or ISO date (YYYY-MM-DD)
- `placement` (optional): "top" (default) or "bottom"

## Examples
```json
// Move to today
{ "id": "uuid-here", "scope": "today" }

// Move to backlog at bottom
{ "id": "uuid-here", "scope": "backlog", "placement": "bottom" }

// Move to specific date
{ "id": "uuid-here", "scope": "2024-01-20" }
```"#
            .to_string()
    }

    pub async fn execute(&self, params: MoveTodoParams) -> Result<MoveTodoResult> {
        let id = Uuid::parse_str(&params.id)
            .into_diagnostic()
            .map_err(|_| miette::miette!("invalid UUID format"))?;

        let today = Local::now().date_naive();
        let scope = parse_scope(&params.scope, today)?;
        let placement = parse_placement(params.placement.as_deref());

        let model = self.service.move_to_scope(id, scope, placement).await?;

        let destination = match model.scheduled_for {
            Some(date) if date == today => "today".to_string(),
            Some(date) => date.format("%Y-%m-%d").to_string(),
            None => "backlog".to_string(),
        };

        let epic_title = if let Some(eid) = model.epic_id {
            self.service.get_epic_title(eid).await.ok()
        } else {
            None
        };

        Ok(MoveTodoResult {
            id: model.id.to_string(),
            title: model.title,
            status: model.status,
            scheduled_for: model
                .scheduled_for
                .map(|d| d.format("%Y-%m-%d").to_string()),
            project: model.project,
            epic_id: model.epic_id.map(|u| u.to_string()),
            epic_title,
            message: format!("Todo moved to {}", destination),
        })
    }
}

fn parse_placement(s: Option<&str>) -> MovePlacement {
    match s.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("bottom") => MovePlacement::Bottom,
        _ => MovePlacement::Top,
    }
}

impl McpTool for MoveTodoTool {
    type Params = MoveTodoParams;
    type Result = MoveTodoResult;

    fn name() -> &'static str {
        "move_todo"
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
