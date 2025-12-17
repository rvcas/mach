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

const INSTRUCTIONS: &str = r#"**Mach Task Manager MCP Server** - Manage todos across projects.

## Available Tools

| Tool | Purpose |
|------|---------|
| `add_todo` | Create a new todo (with optional project tag) |
| `list_todos` | List todos by scope (today/backlog/date) with optional project filter |
| `get_todo` | Get full details of a todo |
| `update_todo` | Update title, date, or notes |
| `delete_todo` | Permanently delete a todo |
| `mark_done` | Mark a todo as completed |
| `mark_pending` | Revert a completed todo to pending |
| `move_todo` | Move a todo to a different day/backlog |

## Project Filtering
- Use the `project` parameter in `add_todo` to tag todos with `[project]` prefix
- Use the `project` parameter in `list_todos` to filter by project
- Or use the `prefix` parameter for custom filtering

## Workflow
1. Use `list_todos` with optional project filter
2. Use `add_todo` with project param to create tagged todos
3. Use `mark_done` when completing tasks
4. Use `move_todo` to reschedule tasks
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
