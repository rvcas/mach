use crate::contracts::ToolDescription;
use crate::handler::{MachMcpHandler, McpOptions};
use crate::tools::{
    AddTodoTool, DeleteTodoTool, GetTodoTool, ListTodosTool, MarkDoneTool, MarkPendingTool,
    MoveTodoTool, UpdateTodoTool,
};
use machich::service::todo::TodoService;
use miette::{IntoDiagnostic, Result, WrapErr};
use rmcp::service::serve_server;
use rmcp::transport::stdio;
use std::sync::Arc;

const INSTRUCTIONS: &str = r#"**Mach Task Manager MCP Server** - Manage todos with projects and epics.

## Available Tools

| Tool | Purpose |
|------|---------|
| `add_todo` | Create a new todo (with optional project/epicId) |
| `list_todos` | List todos by scope with project/epic filtering |
| `get_todo` | Get full details of a todo |
| `update_todo` | Update title, date, notes, project, or epicId |
| `delete_todo` | Permanently delete a todo |
| `mark_done` | Mark a todo as completed |
| `mark_pending` | Revert a completed todo to pending |
| `move_todo` | Move a todo to a different day/backlog |

## Projects & Epics
- `project`: A string tag for grouping todos (e.g., "my-app")
- `epicId`: UUID of a parent todo that serves as an epic
- Todos can have both a project and an epic
- Child todos inherit project from their epic if not specified

## Filtering
- `project`: Filter by exact project value
- `noProject`: Filter to todos with no project set
- `epicId`: Filter to sub-tasks of a specific epic

## Workflow
1. Create epics as regular todos (use descriptive titles/notes)
2. Create sub-tasks with `epicId` pointing to the epic
3. Use `list_todos` with `epicId` to see all sub-tasks
4. **Important**: Do NOT auto-close epics - let users decide when done
"#;

pub struct MachMcpServer {
    add_tool: Arc<AddTodoTool>,
    list_tool: Arc<ListTodosTool>,
    get_tool: Arc<GetTodoTool>,
    update_tool: Arc<UpdateTodoTool>,
    delete_tool: Arc<DeleteTodoTool>,
    mark_done_tool: Arc<MarkDoneTool>,
    mark_pending_tool: Arc<MarkPendingTool>,
    move_tool: Arc<MoveTodoTool>,
    tools: Arc<Vec<ToolDescription>>,
}

impl MachMcpServer {
    pub fn new(service: TodoService) -> Result<Self> {
        let add_tool = Arc::new(AddTodoTool::new(service.clone()));
        let list_tool = Arc::new(ListTodosTool::new(service.clone()));
        let get_tool = Arc::new(GetTodoTool::new(service.clone()));
        let update_tool = Arc::new(UpdateTodoTool::new(service.clone()));
        let delete_tool = Arc::new(DeleteTodoTool::new(service.clone()));
        let mark_done_tool = Arc::new(MarkDoneTool::new(service.clone()));
        let mark_pending_tool = Arc::new(MarkPendingTool::new(service.clone()));
        let move_tool = Arc::new(MoveTodoTool::new(service));

        let tools = Arc::new(vec![
            ToolDescription::new("add_todo", AddTodoTool::description(), AddTodoTool::schema())?,
            ToolDescription::new(
                "list_todos",
                ListTodosTool::description(),
                ListTodosTool::schema(),
            )?,
            ToolDescription::new("get_todo", GetTodoTool::description(), GetTodoTool::schema())?,
            ToolDescription::new(
                "update_todo",
                UpdateTodoTool::description(),
                UpdateTodoTool::schema(),
            )?,
            ToolDescription::new(
                "delete_todo",
                DeleteTodoTool::description(),
                DeleteTodoTool::schema(),
            )?,
            ToolDescription::new(
                "mark_done",
                MarkDoneTool::description(),
                MarkDoneTool::schema(),
            )?,
            ToolDescription::new(
                "mark_pending",
                MarkPendingTool::description(),
                MarkPendingTool::schema(),
            )?,
            ToolDescription::new(
                "move_todo",
                MoveTodoTool::description(),
                MoveTodoTool::schema(),
            )?,
        ]);

        Ok(Self {
            add_tool,
            list_tool,
            get_tool,
            update_tool,
            delete_tool,
            mark_done_tool,
            mark_pending_tool,
            move_tool,
            tools,
        })
    }

    pub async fn serve_stdio(self) -> Result<()> {
        let mcp_options = McpOptions {
            add_tool: self.add_tool,
            list_tool: self.list_tool,
            get_tool: self.get_tool,
            update_tool: self.update_tool,
            delete_tool: self.delete_tool,
            mark_done_tool: self.mark_done_tool,
            mark_pending_tool: self.mark_pending_tool,
            move_tool: self.move_tool,
            tool_descriptions: self.tools,
            instructions: Some(INSTRUCTIONS.to_string()),
        };

        let handler = MachMcpHandler::new(mcp_options);
        let (stdin, stdout) = stdio();

        let running = serve_server(handler, (stdin, stdout))
            .await
            .into_diagnostic()
            .wrap_err("failed to initialize stdio MCP server")?;

        running.waiting().await.into_diagnostic()?;
        Ok(())
    }
}
