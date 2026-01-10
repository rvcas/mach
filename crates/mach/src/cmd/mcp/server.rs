use chrono::NaiveDate;
use miette::IntoDiagnostic;
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::service::{
    Services,
    todo::{ListOptions, ListScope, MovePlacement},
};

#[derive(Clone)]
pub struct MachMcpServer {
    services: Services,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddTodoParams {
    #[schemars(description = "The title/description of the todo item")]
    pub title: String,

    #[serde(rename = "scheduledFor")]
    #[schemars(
        description = "ISO date (YYYY-MM-DD) to schedule the todo. Omitted = today. Use 'backlog' for someday items."
    )]
    pub scheduled_for: Option<String>,

    #[schemars(description = "Optional notes/details for the todo")]
    pub notes: Option<String>,

    #[schemars(description = "Optional workspace name or UUID")]
    pub workspace: Option<String>,

    #[schemars(description = "Optional project name or UUID")]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListTodosParams {
    #[schemars(
        description = "Filter scope: 'today' (default), 'backlog', or ISO date (YYYY-MM-DD)"
    )]
    pub scope: Option<String>,

    #[serde(rename = "includeDone")]
    #[schemars(description = "Include completed todos (default: false)")]
    pub include_done: Option<bool>,

    #[schemars(description = "Optional workspace name or UUID to filter by")]
    pub workspace: Option<String>,

    #[schemars(description = "Optional project name or UUID to filter by")]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTodoParams {
    #[schemars(description = "UUID of the todo to retrieve")]
    pub id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateTodoParams {
    #[schemars(description = "UUID of the todo to update")]
    pub id: String,

    #[schemars(description = "New title (if provided)")]
    pub title: Option<String>,

    #[serde(rename = "scheduledFor")]
    #[schemars(
        description = "New scheduled date (YYYY-MM-DD) or null/'backlog' to move to backlog"
    )]
    pub scheduled_for: Option<String>,

    #[schemars(description = "New notes (if provided)")]
    pub notes: Option<String>,

    #[schemars(description = "Workspace name or UUID, or empty string to clear")]
    pub workspace: Option<String>,

    #[schemars(description = "Project name or UUID, or empty string to clear")]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteTodoParams {
    #[schemars(description = "UUID of the todo to delete")]
    pub id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MarkDoneParams {
    #[schemars(description = "UUID of the todo to mark as done")]
    pub id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MarkPendingParams {
    #[schemars(description = "UUID of the todo to mark as pending")]
    pub id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MoveTodoParams {
    #[schemars(description = "UUID of the todo to move")]
    pub id: String,

    #[schemars(description = "Target: 'today', 'backlog', or ISO date (YYYY-MM-DD)")]
    pub scope: String,

    #[schemars(description = "Where to place in the target column: 'top' (default) or 'bottom'")]
    pub placement: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateWorkspaceParams {
    #[schemars(description = "Name of the workspace")]
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListWorkspacesParams {}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateWorkspaceParams {
    #[schemars(description = "UUID of the workspace to update")]
    pub id: String,

    #[schemars(description = "New name for the workspace")]
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateProjectParams {
    #[schemars(description = "Name of the project")]
    pub name: String,

    #[schemars(description = "Workspace name or UUID (required)")]
    pub workspace: String,

    #[schemars(description = "Initial status: 'pending' (default), 'permanent'")]
    pub status: Option<String>,

    #[schemars(description = "Optional project description")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListProjectsParams {
    #[schemars(description = "Optional workspace name or UUID to filter by")]
    pub workspace: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateProjectParams {
    #[schemars(description = "UUID of the project to update")]
    pub id: String,

    #[schemars(description = "New name for the project")]
    pub name: Option<String>,

    #[schemars(description = "New status: 'pending', 'done', or 'permanent'")]
    pub status: Option<String>,

    #[schemars(description = "New description, or empty string to clear")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MarkProjectDoneParams {
    #[schemars(description = "UUID of the project to mark as done")]
    pub id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReopenProjectParams {
    #[schemars(description = "UUID of the project to reopen")]
    pub id: String,
}

#[derive(Debug, Serialize)]
struct TodoResponse {
    id: Uuid,
    title: String,
    status: String,
    #[serde(rename = "scheduledFor")]
    scheduled_for: Option<String>,
    notes: Option<String>,
    #[serde(rename = "workspaceId")]
    workspace_id: Option<Uuid>,
    #[serde(rename = "projectId")]
    project_id: Option<Uuid>,
}

impl From<crate::entity::todo::Model> for TodoResponse {
    fn from(model: crate::entity::todo::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            status: model.status,
            scheduled_for: model.scheduled_for.map(|d| d.to_string()),
            notes: model.notes,
            workspace_id: model.workspace_id,
            project_id: model.project_id,
        }
    }
}

#[derive(Debug, Serialize)]
struct WorkspaceResponse {
    id: Uuid,
    name: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

impl From<crate::entity::workspace::Model> for WorkspaceResponse {
    fn from(model: crate::entity::workspace::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
struct ProjectResponse {
    id: Uuid,
    name: String,
    #[serde(rename = "workspaceId")]
    workspace_id: Uuid,
    status: String,
    description: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

impl From<crate::entity::project::Model> for ProjectResponse {
    fn from(model: crate::entity::project::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            workspace_id: model.workspace_id,
            status: model.status,
            description: model.description,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}

#[tool_router]
impl MachMcpServer {
    pub fn new(services: Services) -> Self {
        Self {
            services,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Add a new todo item")]
    async fn mach_add_todo(
        &self,
        Parameters(params): Parameters<AddTodoParams>,
    ) -> Result<CallToolResult, McpError> {
        let scheduled_for = parse_scheduled_for(&params.scheduled_for, &self.services)?;

        let (workspace_id, project_id) = resolve_workspace_project(
            &self.services,
            params.workspace.as_deref(),
            params.project.as_deref(),
        )
        .await?;

        let todo = self
            .services
            .todos
            .add(
                &params.title,
                scheduled_for,
                params.notes,
                workspace_id,
                project_id,
            )
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let response = TodoResponse::from(todo);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "List todo items, optionally filtered by workspace and/or project")]
    async fn mach_list_todos(
        &self,
        Parameters(params): Parameters<ListTodosParams>,
    ) -> Result<CallToolResult, McpError> {
        let scope = match params.scope.as_deref() {
            None | Some("today") => ListScope::Day(self.services.today()),
            Some("backlog") => ListScope::Backlog,
            Some(date_str) => {
                let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
                    McpError::invalid_params("Invalid date format, use YYYY-MM-DD", None)
                })?;
                ListScope::Day(date)
            }
        };

        // Resolve workspace and project independently - no validation needed for listing.
        // Mismatched filters simply return 0 results.
        let workspace_id = if let Some(ref ws) = params.workspace {
            Some(
                self.services
                    .workspaces
                    .find_by_name_or_id(ws)
                    .await
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?
                    .ok_or_else(|| {
                        McpError::invalid_params(format!("Workspace '{}' not found", ws), None)
                    })?
                    .id,
            )
        } else {
            None
        };

        let project_id = if let Some(ref proj) = params.project {
            Some(
                self.services
                    .projects
                    .find_by_name_or_id(proj)
                    .await
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?
                    .ok_or_else(|| {
                        McpError::invalid_params(format!("Project '{}' not found", proj), None)
                    })?
                    .id,
            )
        } else {
            None
        };

        let opts = ListOptions {
            scope,
            include_done: params.include_done.unwrap_or(false),
            workspace_id,
            project_id,
        };

        let todos = self
            .services
            .todos
            .list(opts)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let response: Vec<TodoResponse> = todos.into_iter().map(TodoResponse::from).collect();
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Get a single todo by ID")]
    async fn mach_get_todo(
        &self,
        Parameters(params): Parameters<GetTodoParams>,
    ) -> Result<CallToolResult, McpError> {
        let id = Uuid::parse_str(&params.id)
            .map_err(|_| McpError::invalid_params("Invalid UUID format", None))?;

        let todo = self
            .services
            .todos
            .get(id)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let response = TodoResponse::from(todo);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Update a todo's title, date, notes, or workspace/project")]
    async fn mach_update_todo(
        &self,
        Parameters(params): Parameters<UpdateTodoParams>,
    ) -> Result<CallToolResult, McpError> {
        let id = Uuid::parse_str(&params.id)
            .map_err(|_| McpError::invalid_params("Invalid UUID format", None))?;

        let mut todo = self
            .services
            .todos
            .get(id)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        if let Some(title) = params.title {
            todo = self
                .services
                .todos
                .update_title(todo.id, title)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        }

        if let Some(ref scheduled_for) = params.scheduled_for {
            let date = parse_scheduled_for_update(scheduled_for)?;
            todo = self
                .services
                .todos
                .update_scheduled_for(todo.id, date)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        }

        if let Some(notes) = params.notes {
            let notes = if notes.is_empty() { None } else { Some(notes) };
            todo = self
                .services
                .todos
                .update_notes(todo.id, notes)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        }

        if params.workspace.is_some() || params.project.is_some() {
            let (workspace_id, project_id) = resolve_workspace_project_update(
                &self.services,
                params.workspace.as_deref(),
                params.project.as_deref(),
            )
            .await?;
            todo = self
                .services
                .todos
                .update_workspace_project(todo.id, workspace_id, project_id)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        }

        let response = TodoResponse::from(todo);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Delete a todo permanently")]
    async fn mach_delete_todo(
        &self,
        Parameters(params): Parameters<DeleteTodoParams>,
    ) -> Result<CallToolResult, McpError> {
        let id = Uuid::parse_str(&params.id)
            .map_err(|_| McpError::invalid_params("Invalid UUID format", None))?;

        self.services
            .todos
            .delete(id)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::json!({"deleted": true, "id": id}).to_string(),
        )]))
    }

    #[tool(description = "Mark a todo as completed")]
    async fn mach_mark_done(
        &self,
        Parameters(params): Parameters<MarkDoneParams>,
    ) -> Result<CallToolResult, McpError> {
        let id = Uuid::parse_str(&params.id)
            .map_err(|_| McpError::invalid_params("Invalid UUID format", None))?;

        let todo = self
            .services
            .todos
            .mark_done(id, self.services.today())
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let response = TodoResponse::from(todo);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Revert a completed todo back to pending")]
    async fn mach_mark_pending(
        &self,
        Parameters(params): Parameters<MarkPendingParams>,
    ) -> Result<CallToolResult, McpError> {
        let id = Uuid::parse_str(&params.id)
            .map_err(|_| McpError::invalid_params("Invalid UUID format", None))?;

        let todo = self
            .services
            .todos
            .mark_pending(id)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let response = TodoResponse::from(todo);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Move a todo to a different day or backlog")]
    async fn mach_move_todo(
        &self,
        Parameters(params): Parameters<MoveTodoParams>,
    ) -> Result<CallToolResult, McpError> {
        let id = Uuid::parse_str(&params.id)
            .map_err(|_| McpError::invalid_params("Invalid UUID format", None))?;

        let scope = match params.scope.as_str() {
            "today" => ListScope::Day(self.services.today()),
            "backlog" => ListScope::Backlog,
            date_str => {
                let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
                    McpError::invalid_params("Invalid date format, use YYYY-MM-DD", None)
                })?;
                ListScope::Day(date)
            }
        };

        let placement = match params.placement.as_deref() {
            None | Some("top") => MovePlacement::Top,
            Some("bottom") => MovePlacement::Bottom,
            Some(_) => {
                return Err(McpError::invalid_params(
                    "Placement must be 'top' or 'bottom'",
                    None,
                ));
            }
        };

        let todo = self
            .services
            .todos
            .move_to_scope(id, scope, placement)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let response = TodoResponse::from(todo);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Create a new workspace")]
    async fn mach_create_workspace(
        &self,
        Parameters(params): Parameters<CreateWorkspaceParams>,
    ) -> Result<CallToolResult, McpError> {
        let workspace = self
            .services
            .workspaces
            .create(&params.name)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let response = WorkspaceResponse::from(workspace);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "List all workspaces")]
    async fn mach_list_workspaces(
        &self,
        Parameters(_params): Parameters<ListWorkspacesParams>,
    ) -> Result<CallToolResult, McpError> {
        let workspaces = self
            .services
            .workspaces
            .list()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let response: Vec<WorkspaceResponse> = workspaces
            .into_iter()
            .map(WorkspaceResponse::from)
            .collect();
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Update a workspace's name")]
    async fn mach_update_workspace(
        &self,
        Parameters(params): Parameters<UpdateWorkspaceParams>,
    ) -> Result<CallToolResult, McpError> {
        let id = Uuid::parse_str(&params.id)
            .map_err(|_| McpError::invalid_params("Invalid UUID format", None))?;

        let workspace = self
            .services
            .workspaces
            .update_name(id, &params.name)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let response = WorkspaceResponse::from(workspace);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Create a new project in a workspace")]
    async fn mach_create_project(
        &self,
        Parameters(params): Parameters<CreateProjectParams>,
    ) -> Result<CallToolResult, McpError> {
        let workspace = self
            .services
            .workspaces
            .find_by_name_or_id(&params.workspace)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .ok_or_else(|| {
                McpError::invalid_params(
                    format!("Workspace '{}' not found", params.workspace),
                    None,
                )
            })?;

        let status = params.status.as_deref().unwrap_or("pending");
        if !["pending", "permanent"].contains(&status) {
            return Err(McpError::invalid_params(
                "Status must be 'pending' or 'permanent'",
                None,
            ));
        }

        let project = self
            .services
            .projects
            .create(&params.name, workspace.id, status, params.description)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let response = ProjectResponse::from(project);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "List projects, optionally filtered by workspace")]
    async fn mach_list_projects(
        &self,
        Parameters(params): Parameters<ListProjectsParams>,
    ) -> Result<CallToolResult, McpError> {
        let projects = if let Some(ref ws) = params.workspace {
            let workspace = self
                .services
                .workspaces
                .find_by_name_or_id(ws)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
                .ok_or_else(|| {
                    McpError::invalid_params(format!("Workspace '{}' not found", ws), None)
                })?;

            self.services
                .projects
                .list_by_workspace(workspace.id)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
        } else {
            self.services
                .projects
                .list()
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
        };

        let response: Vec<ProjectResponse> =
            projects.into_iter().map(ProjectResponse::from).collect();
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Update a project's name or status")]
    async fn mach_update_project(
        &self,
        Parameters(params): Parameters<UpdateProjectParams>,
    ) -> Result<CallToolResult, McpError> {
        let id = Uuid::parse_str(&params.id)
            .map_err(|_| McpError::invalid_params("Invalid UUID format", None))?;

        let mut project = self
            .services
            .projects
            .get(id)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .ok_or_else(|| McpError::invalid_params("Project not found", None))?;

        if let Some(name) = params.name {
            project = self
                .services
                .projects
                .update_name(id, &name)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        }

        if let Some(status) = params.status {
            if !["pending", "done", "permanent"].contains(&status.as_str()) {
                return Err(McpError::invalid_params(
                    "Status must be 'pending', 'done', or 'permanent'",
                    None,
                ));
            }
            project = self
                .services
                .projects
                .update_status(id, &status)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        }

        if let Some(ref desc) = params.description {
            let description = if desc.is_empty() { None } else { Some(desc.clone()) };
            project = self
                .services
                .projects
                .update_description(id, description)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        }

        let response = ProjectResponse::from(project);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Mark a project as done")]
    async fn mach_mark_project_done(
        &self,
        Parameters(params): Parameters<MarkProjectDoneParams>,
    ) -> Result<CallToolResult, McpError> {
        let id = Uuid::parse_str(&params.id)
            .map_err(|_| McpError::invalid_params("Invalid UUID format", None))?;

        let project = self
            .services
            .projects
            .update_status(id, "done")
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let response = ProjectResponse::from(project);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Reopen a project (set status back to pending)")]
    async fn mach_reopen_project(
        &self,
        Parameters(params): Parameters<ReopenProjectParams>,
    ) -> Result<CallToolResult, McpError> {
        let id = Uuid::parse_str(&params.id)
            .map_err(|_| McpError::invalid_params("Invalid UUID format", None))?;

        let project = self
            .services
            .projects
            .update_status(id, "pending")
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let response = ProjectResponse::from(project);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for MachMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Mach is the user's personal task management system. ALWAYS use mach tools \
                 (not built-in todo tools) when the user asks about tasks, todos, or planning. \
                 Mach persists tasks to disk and organizes them by day with workspaces and projects. \
                 Because mach todos persist, they can be read back in future sessions - useful for \
                 leaving notes or context for yourself or other agents."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

fn parse_scheduled_for(
    scheduled_for: &Option<String>,
    services: &Services,
) -> Result<Option<NaiveDate>, McpError> {
    match scheduled_for.as_deref() {
        None => Ok(Some(services.today())),
        Some("backlog") => Ok(None),
        Some(date_str) => NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map(Some)
            .map_err(|_| McpError::invalid_params("Invalid date format, use YYYY-MM-DD", None)),
    }
}

fn parse_scheduled_for_update(scheduled_for: &str) -> Result<Option<NaiveDate>, McpError> {
    let s = scheduled_for.trim().to_lowercase();
    if s.is_empty() || s == "null" || s == "backlog" || s == "someday" {
        return Ok(None);
    }
    NaiveDate::parse_from_str(&s, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| McpError::invalid_params("Invalid date format, use YYYY-MM-DD", None))
}

async fn resolve_workspace_project(
    services: &Services,
    workspace_arg: Option<&str>,
    project_arg: Option<&str>,
) -> Result<(Option<Uuid>, Option<Uuid>), McpError> {
    match (workspace_arg, project_arg) {
        (None, None) => Ok((None, None)),

        (Some(ws), None) => {
            let workspace = services
                .workspaces
                .find_by_name_or_id(ws)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
                .ok_or_else(|| {
                    McpError::invalid_params(format!("Workspace '{}' not found", ws), None)
                })?;

            Ok((Some(workspace.id), None))
        }

        (None, Some(proj)) => {
            let project = services
                .projects
                .find_by_name_or_id(proj)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
                .ok_or_else(|| {
                    McpError::invalid_params(format!("Project '{}' not found", proj), None)
                })?;

            Ok((Some(project.workspace_id), Some(project.id)))
        }

        (Some(ws), Some(proj)) => {
            let workspace = services
                .workspaces
                .find_by_name_or_id(ws)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
                .ok_or_else(|| {
                    McpError::invalid_params(format!("Workspace '{}' not found", ws), None)
                })?;

            let project = services
                .projects
                .find_by_name_or_id(proj)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
                .ok_or_else(|| {
                    McpError::invalid_params(format!("Project '{}' not found", proj), None)
                })?;

            if project.workspace_id != workspace.id {
                return Err(McpError::invalid_params(
                    format!("Project '{}' is not in workspace '{}'", proj, ws),
                    None,
                ));
            }

            Ok((Some(workspace.id), Some(project.id)))
        }
    }
}

async fn resolve_workspace_project_update(
    services: &Services,
    workspace_arg: Option<&str>,
    project_arg: Option<&str>,
) -> Result<(Option<Uuid>, Option<Uuid>), McpError> {
    let ws_clear = workspace_arg.is_some_and(|s| s.is_empty() || s == "null");
    let proj_clear = project_arg.is_some_and(|s| s.is_empty() || s == "null");

    if ws_clear && proj_clear {
        return Ok((None, None));
    }

    if ws_clear {
        return Ok((None, None));
    }

    if proj_clear {
        if let Some(ws) = workspace_arg {
            let workspace = services
                .workspaces
                .find_by_name_or_id(ws)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
                .ok_or_else(|| {
                    McpError::invalid_params(format!("Workspace '{}' not found", ws), None)
                })?;
            return Ok((Some(workspace.id), None));
        }
        return Ok((None, None));
    }

    resolve_workspace_project(services, workspace_arg, project_arg).await
}

pub async fn run(services: Services) -> miette::Result<()> {
    let server = MachMcpServer::new(services);
    let service = server.serve(stdio()).await.into_diagnostic()?;
    service.waiting().await.into_diagnostic()?;
    Ok(())
}
