use crate::contracts::ToolDescription;
use crate::tools::{
    AddTodoTool, DeleteTodoTool, GetTodoTool, ListTodosTool, MarkDoneTool, MarkPendingTool,
    McpTool, MoveTodoTool, UpdateTodoTool,
};
use machich::service::error::TodoError;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestMethod, CallToolRequestParam, CallToolResult, ErrorData, InitializeResult,
    JsonObject, ListToolsResult, ServerCapabilities,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, model::Implementation};
use serde_json::{Map, Value};
use std::future::Future;
use std::sync::Arc;

#[derive(Clone)]
pub struct McpOptions {
    pub add_tool: Arc<AddTodoTool>,
    pub list_tool: Arc<ListTodosTool>,
    pub get_tool: Arc<GetTodoTool>,
    pub update_tool: Arc<UpdateTodoTool>,
    pub delete_tool: Arc<DeleteTodoTool>,
    pub mark_done_tool: Arc<MarkDoneTool>,
    pub mark_pending_tool: Arc<MarkPendingTool>,
    pub move_tool: Arc<MoveTodoTool>,
    pub tool_descriptions: Arc<Vec<ToolDescription>>,
    pub instructions: Option<String>,
}

#[derive(Clone)]
pub struct MachMcpHandler {
    options: McpOptions,
    info: InitializeResult,
}

impl MachMcpHandler {
    pub fn new(options: McpOptions) -> Self {
        let server_info = Implementation {
            name: "mach-mcp".into(),
            title: Some("Mach Task Manager".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            icons: None,
            website_url: None,
        };

        let capabilities = ServerCapabilities::builder().enable_tools().build();

        Self {
            info: InitializeResult {
                protocol_version: Default::default(),
                capabilities,
                server_info,
                instructions: options.instructions.clone(),
            },
            options,
        }
    }
}

impl ServerHandler for MachMcpHandler {
    fn get_info(&self) -> InitializeResult {
        self.info.clone()
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        let tools = self
            .options
            .tool_descriptions
            .iter()
            .map(tool_description_to_rmcp)
            .collect();
        std::future::ready(Ok(ListToolsResult {
            tools,
            next_cursor: None,
        }))
    }

    #[allow(clippy::manual_async_fn)]
    fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = miette::Result<CallToolResult, ErrorData>> + Send + '_ {
        async move {
            let args = request.arguments.clone();
            match request.name.as_ref() {
                "add_todo" => dispatch_tool(&*self.options.add_tool, args).await,
                "list_todos" => dispatch_tool(&*self.options.list_tool, args).await,
                "get_todo" => dispatch_tool(&*self.options.get_tool, args).await,
                "update_todo" => dispatch_tool(&*self.options.update_tool, args).await,
                "delete_todo" => dispatch_tool(&*self.options.delete_tool, args).await,
                "mark_done" => dispatch_tool(&*self.options.mark_done_tool, args).await,
                "mark_pending" => dispatch_tool(&*self.options.mark_pending_tool, args).await,
                "move_todo" => dispatch_tool(&*self.options.move_tool, args).await,
                _ => Err(ErrorData::method_not_found::<CallToolRequestMethod>()),
            }
        }
    }
}

async fn dispatch_tool<T: McpTool>(
    tool: &T,
    args: Option<JsonObject>,
) -> Result<CallToolResult, ErrorData> {
    let value = Value::Object(args.unwrap_or_else(Map::new));
    let params: T::Params = serde_json::from_value(value).map_err(|err| {
        ErrorData::invalid_params(format!("invalid parameters for {}: {err}", T::name()), None)
    })?;
    let result = tool.run(params).await.map_err(map_miette)?;
    let value =
        serde_json::to_value(&result).map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
    Ok(CallToolResult::structured(value))
}

/// Maps miette errors to MCP error codes using typed error classification.
///
/// If the error is a `TodoError`, uses `is_client_error()` for robust classification.
/// Falls back to string heuristics for other error types.
fn map_miette(error: miette::Error) -> ErrorData {
    let msg = error.to_string();

    // Try typed classification first via TodoError
    if let Some(todo_error) = error.downcast_ref::<TodoError>() {
        return if todo_error.is_client_error() {
            ErrorData::invalid_params(msg, None)
        } else {
            ErrorData::internal_error(msg, None)
        };
    }

    // Fallback: string heuristics for non-TodoError types
    let lower = msg.to_lowercase();
    let is_invalid_params = lower.contains("not found")
        || lower.contains("invalid uuid")
        || lower.contains("does not match")
        || lower.contains("cannot");

    if is_invalid_params {
        ErrorData::invalid_params(msg, None)
    } else {
        ErrorData::internal_error(msg, None)
    }
}

fn tool_description_to_rmcp(desc: &ToolDescription) -> rmcp::model::Tool {
    let schema = match desc.input_schema.clone() {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    rmcp::model::Tool {
        name: desc.name.clone().into(),
        title: None,
        description: Some(desc.description.clone().into()),
        input_schema: Arc::new(schema),
        output_schema: None,
        annotations: None,
        icons: None,
        meta: None,
    }
}
