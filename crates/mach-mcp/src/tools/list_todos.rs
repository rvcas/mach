use crate::contracts::{CallToolResponse, Content};
use chrono::{Local, NaiveDate};
use machich::service::todo::{ListOptions, ListScope, TodoService};
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTodosParams {
    /// "today", "backlog", or ISO date (YYYY-MM-DD)
    pub scope: Option<String>,
    /// Include completed todos
    pub include_done: Option<bool>,
    /// Include notes in response (default: false)
    pub include_notes: Option<bool>,
    /// Filter by title prefix (e.g., "[my-project]")
    pub prefix: Option<String>,
    /// Filter by project tag. Equivalent to prefix "[project]"
    pub project: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub status: String,
    pub scheduled_for: Option<String>,
    pub notes: Option<String>,
    pub order_index: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTodosResult {
    pub todos: Vec<TodoItem>,
    pub scope: String,
    pub count: usize,
}

pub struct ListTodosTool {
    service: TodoService,
}

impl ListTodosTool {
    pub fn new(service: TodoService) -> Self {
        Self { service }
    }

    pub fn schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "scope": {
                    "type": ["string", "null"],
                    "description": "Filter scope: 'today' (default), 'backlog', or ISO date (YYYY-MM-DD)"
                },
                "includeDone": {
                    "type": ["boolean", "null"],
                    "description": "Include completed todos (default: false)"
                },
                "includeNotes": {
                    "type": ["boolean", "null"],
                    "description": "Include notes in response (default: false)"
                },
                "prefix": {
                    "type": ["string", "null"],
                    "description": "Filter by title prefix (e.g., '[my-project]'). No default filter."
                },
                "project": {
                    "type": ["string", "null"],
                    "description": "Filter by project tag. Equivalent to prefix '[project]'."
                }
            },
            "additionalProperties": false
        })
    }

    pub fn description() -> String {
        r#"**List Todos** - List todo items.

## Parameters
- `scope` (optional): "today" (default), "backlog", or ISO date (YYYY-MM-DD)
- `includeDone` (optional): Include completed todos (default: false)
- `includeNotes` (optional): Include notes in response (default: false)
- `prefix` (optional): Filter by title prefix (e.g., "[my-project]")
- `project` (optional): Filter by project tag. Equivalent to prefix "[project]"

## Examples
```json
// List all today's todos
{ }

// Filter by project
{ "project": "my-app" }

// Filter by custom prefix
{ "prefix": "[backend]" }

// List backlog items
{ "scope": "backlog" }
```

## Returns
Array of todos with id, title, status, scheduledFor, notes, orderIndex."#
            .to_string()
    }

    pub async fn call(&self, params: ListTodosParams) -> Result<CallToolResponse> {
        let result = self.execute(params).await?;
        let json = serde_json::to_string(&result).into_diagnostic()?;
        Ok(CallToolResponse {
            content: vec![Content::text(json)],
        })
    }

    pub async fn execute(&self, params: ListTodosParams) -> Result<ListTodosResult> {
        let today = Local::now().date_naive();
        let scope_str = params.scope.clone().unwrap_or_else(|| "today".to_string());

        let scope = parse_scope(&scope_str, today)?;
        let include_done = params.include_done.unwrap_or(false);
        let include_notes = params.include_notes.unwrap_or(false);

        // Determine prefix filter: project param takes precedence, then prefix param
        let prefix_filter = match (&params.project, &params.prefix) {
            (Some(proj), _) => Some(format!("[{}]", proj)),
            (None, Some(p)) if !p.is_empty() => Some(p.clone()),
            _ => None,
        };

        let opts = ListOptions {
            scope,
            include_done,
        };

        let models = self.service.list(opts).await?;

        let todos: Vec<TodoItem> = models
            .into_iter()
            .filter(|m| match &prefix_filter {
                Some(prefix) => m.title.starts_with(prefix),
                None => true,
            })
            .map(|m| TodoItem {
                id: m.id.to_string(),
                title: m.title,
                status: m.status,
                scheduled_for: m.scheduled_for.map(|d| d.format("%Y-%m-%d").to_string()),
                notes: if include_notes { m.notes } else { None },
                order_index: m.order_index,
            })
            .collect();

        let count = todos.len();

        Ok(ListTodosResult {
            todos,
            scope: scope_str,
            count,
        })
    }
}

fn parse_scope(s: &str, today: NaiveDate) -> Result<ListScope> {
    match s.trim().to_lowercase().as_str() {
        "today" => Ok(ListScope::Day(today)),
        "backlog" | "someday" => Ok(ListScope::Backlog),
        date_str => {
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .into_diagnostic()
                .map_err(|_| {
                    miette::miette!("invalid scope, expected 'today', 'backlog', or YYYY-MM-DD")
                })?;
            Ok(ListScope::Day(date))
        }
    }
}
