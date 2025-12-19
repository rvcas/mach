use miette::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::future::Future;

pub trait McpTool: Send + Sync {
    type Params: DeserializeOwned + Send;
    type Result: Serialize + Send;

    fn name() -> &'static str;
    fn schema() -> Value;
    fn description() -> String;
    fn run(&self, params: Self::Params) -> impl Future<Output = Result<Self::Result>> + Send;
}
