use miette::{Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Describes an MCP tool (name/description/schema).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
    pub input_schema: JsonValue,
}

impl ToolDescription {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: JsonValue,
    ) -> Result<Self> {
        let name = name.into();
        ensure!(!name.trim().is_empty(), "tool name must not be blank");
        let description = description.into();
        ensure!(
            !description.trim().is_empty(),
            "tool description must not be blank"
        );
        Ok(Self {
            name,
            description,
            input_schema,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListToolsResponse {
    pub tools: Vec<ToolDescription>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Content {
    Text {
        text: String,
    },
    #[serde(other)]
    Unknown,
}

impl Content {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallToolResponse {
    pub content: Vec<Content>,
}
