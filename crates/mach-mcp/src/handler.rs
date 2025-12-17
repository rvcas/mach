use crate::contracts::ToolDescription;
use crate::tools::{
    AddTodoTool, DeleteTodoTool, GetTodoTool, ListTodosTool, MarkDoneTool, MarkPendingTool,
    MoveTodoTool, UpdateTodoTool,
};
use miette::Result;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestMethod, CallToolRequestParam, CallToolResult, ErrorData, InitializeResult,
    JsonObject, ListToolsResult, ServerCapabilities,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, model::Implementation};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::future::Future;
use std::sync::Arc;

use crate::tools::add_todo::AddTodoParams;
use crate::tools::delete_todo::DeleteTodoParams;
use crate::tools::get_todo::GetTodoParams;
use crate::tools::list_todos::ListTodosParams;
use crate::tools::mark_done::MarkDoneParams;
use crate::tools::mark_pending::MarkPendingParams;
use crate::tools::move_todo::MoveTodoParams;
use crate::tools::update_todo::UpdateTodoParams;

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
    ) -> impl Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        async move {
            match request.name.as_ref() {
                "add_todo" => {
                    let params: AddTodoParams =
                        parse_arguments(request.arguments.clone(), "add_todo")?;
                    let result = self
                        .options
                        .add_tool
                        .execute(params)
                        .await
                        .map_err(map_miette)?;
                    let value = serde_json::to_value(&result)
                        .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
                    Ok(CallToolResult::structured(value))
                }
                "list_todos" => {
                    let params: ListTodosParams =
                        parse_arguments(request.arguments.clone(), "list_todos")?;
                    let result = self
                        .options
                        .list_tool
                        .execute(params)
                        .await
                        .map_err(map_miette)?;
                    let value = serde_json::to_value(&result)
                        .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
                    Ok(CallToolResult::structured(value))
                }
                "get_todo" => {
                    let params: GetTodoParams =
                        parse_arguments(request.arguments.clone(), "get_todo")?;
                    let result = self
                        .options
                        .get_tool
                        .execute(params)
                        .await
                        .map_err(map_miette)?;
                    let value = serde_json::to_value(&result)
                        .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
                    Ok(CallToolResult::structured(value))
                }
                "update_todo" => {
                    let params: UpdateTodoParams =
                        parse_arguments(request.arguments.clone(), "update_todo")?;
                    let result = self
                        .options
                        .update_tool
                        .execute(params)
                        .await
                        .map_err(map_miette)?;
                    let value = serde_json::to_value(&result)
                        .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
                    Ok(CallToolResult::structured(value))
                }
                "delete_todo" => {
                    let params: DeleteTodoParams =
                        parse_arguments(request.arguments.clone(), "delete_todo")?;
                    let result = self
                        .options
                        .delete_tool
                        .execute(params)
                        .await
                        .map_err(map_miette)?;
                    let value = serde_json::to_value(&result)
                        .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
                    Ok(CallToolResult::structured(value))
                }
                "mark_done" => {
                    let params: MarkDoneParams =
                        parse_arguments(request.arguments.clone(), "mark_done")?;
                    let result = self
                        .options
                        .mark_done_tool
                        .execute(params)
                        .await
                        .map_err(map_miette)?;
                    let value = serde_json::to_value(&result)
                        .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
                    Ok(CallToolResult::structured(value))
                }
                "mark_pending" => {
                    let params: MarkPendingParams =
                        parse_arguments(request.arguments.clone(), "mark_pending")?;
                    let result = self
                        .options
                        .mark_pending_tool
                        .execute(params)
                        .await
                        .map_err(map_miette)?;
                    let value = serde_json::to_value(&result)
                        .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
                    Ok(CallToolResult::structured(value))
                }
                "move_todo" => {
                    let params: MoveTodoParams =
                        parse_arguments(request.arguments.clone(), "move_todo")?;
                    let result = self
                        .options
                        .move_tool
                        .execute(params)
                        .await
                        .map_err(map_miette)?;
                    let value = serde_json::to_value(&result)
                        .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
                    Ok(CallToolResult::structured(value))
                }
                _ => Err(ErrorData::method_not_found::<CallToolRequestMethod>()),
            }
        }
    }
}

fn parse_arguments<T: DeserializeOwned>(
    args: Option<JsonObject>,
    name: &str,
) -> Result<T, ErrorData> {
    let value = Value::Object(args.unwrap_or_else(Map::new));
    serde_json::from_value(value).map_err(|err| {
        ErrorData::invalid_params(format!("invalid parameters for {name}: {err}"), None)
    })
}

fn map_miette(error: miette::Error) -> ErrorData {
    ErrorData::internal_error(error.to_string(), None)
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
