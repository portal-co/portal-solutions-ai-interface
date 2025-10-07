pub mod types;
use anyhow::Error;
use serde_json::json;
#[derive(Default)]
#[non_exhaustive]
pub struct Reactor {
    
}
impl Reactor {
    // Called when the tool is invoked.
    // If you support multiple tools, you must switch on the input.params.name to detect which tool is being called.
    pub fn call(&self, input: types::CallToolRequest) -> Result<types::CallToolResult, Error> {
        let name = &*input.params.name;
        match name {
            _ => return Err(Error::msg("invalid tool name")),
        }
    }

    // Called by mcpx to understand how and why to use this tool.
    // Note: Your servlet configs will not be set when this function is called,
    // so do not rely on config in this function
    pub fn describe(&self) -> Result<types::ListToolsResult, Error> {
        Ok(types::ListToolsResult {
        tools: [
            types::ToolDescription{
                name: format!("execute_code"),
                description: format!("Executes code in a secure sandbox. Use this if you don't already have a sandbox to run code in, and use it to encapsulate the rest of the tools here with `call_tool`"),
                input_schema: [(format!("type"),json!("string"))].into_iter().collect()}].into_iter().collect(),
        })
    }
}
