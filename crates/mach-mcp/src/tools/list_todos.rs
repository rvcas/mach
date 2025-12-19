use super::traits::McpTool;
use super::util::parse_scope;
use chrono::Local;
use machich::service::todo::{ListOptions, ProjectFilter, TodoService};
use miette::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTodosParams {
    pub scope: Option<String>,
    pub include_done: Option<bool>,
    pub include_notes: Option<bool>,
    pub project: Option<String>,
    pub no_project: Option<bool>,
    pub epic_id: Option<String>,
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
    pub project: Option<String>,
    pub epic_id: Option<String>,
    pub epic_title: Option<String>,
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
                    "type": "string",
                    "description": "Filter scope: 'today' (default), 'backlog', or ISO date (YYYY-MM-DD)"
                },
                "includeDone": {
                    "type": "boolean",
                    "description": "Include completed todos (default: false)"
                },
                "includeNotes": {
                    "type": "boolean",
                    "description": "Include notes in response (default: false)"
                },
                "project": {
                    "type": "string",
                    "description": "Filter by project column value"
                },
                "noProject": {
                    "type": "boolean",
                    "description": "Filter to todos with no project (project IS NULL)"
                },
                "epicId": {
                    "type": "string",
                    "description": "Filter to todos linked to this epic UUID (sub-tasks of an epic)"
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
- `project` (optional): Filter by project column value
- `noProject` (optional): Filter to todos with no project set
- `epicId` (optional): Filter to sub-tasks of a specific epic

## Examples
```json
// List all today's todos
{ }

// Filter by project
{ "project": "my-app" }

// List todos with no project
{ "noProject": true }

// List backlog items
{ "scope": "backlog" }

// List sub-tasks of an epic
{ "epicId": "uuid-of-epic" }
```

## Returns
Array of todos with id, title, status, scheduledFor, notes, orderIndex, project, epicId, epicTitle."#
            .to_string()
    }

    pub async fn execute(&self, params: ListTodosParams) -> Result<ListTodosResult> {
        let today = Local::now().date_naive();
        let scope_str = params.scope.clone().unwrap_or_else(|| "today".to_string());

        let scope = parse_scope(&scope_str, today)?;
        let include_done = params.include_done.unwrap_or(false);
        let include_notes = params.include_notes.unwrap_or(false);

        let project_filter = if params.no_project.unwrap_or(false) {
            ProjectFilter::IsNull
        } else {
            match &params.project {
                Some(p) => ProjectFilter::Equals(p.clone()),
                None => ProjectFilter::Any,
            }
        };

        let epic_id = params
            .epic_id
            .as_ref()
            .map(|s| uuid::Uuid::parse_str(s))
            .transpose()
            .map_err(|_| miette::miette!("invalid epicId UUID format"))?;

        let opts = ListOptions {
            scope,
            include_done,
            project: project_filter,
            epic_id,
        };

        let models = self.service.list(opts).await?;

        let epic_ids: Vec<uuid::Uuid> = models.iter().filter_map(|m| m.epic_id).collect();
        let epic_titles = self.service.get_epic_titles(&epic_ids).await?;

        let todos: Vec<TodoItem> = models
            .into_iter()
            .map(|m| {
                let epic_title = m.epic_id.and_then(|eid| epic_titles.get(&eid).cloned());
                TodoItem {
                    id: m.id.to_string(),
                    title: m.title,
                    status: m.status,
                    scheduled_for: m.scheduled_for.map(|d| d.format("%Y-%m-%d").to_string()),
                    notes: if include_notes { m.notes } else { None },
                    order_index: m.order_index,
                    project: m.project,
                    epic_id: m.epic_id.map(|u| u.to_string()),
                    epic_title,
                }
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

impl McpTool for ListTodosTool {
    type Params = ListTodosParams;
    type Result = ListTodosResult;

    fn name() -> &'static str {
        "list_todos"
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
