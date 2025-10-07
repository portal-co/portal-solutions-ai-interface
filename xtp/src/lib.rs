mod pdk;

use extism_pdk::*;
use pdk::*;
use portal_solutions_ai_interface::Reactor;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;

thread_local! {static REACTOR: Reactor = Reactor::default()}
fn cast<A: Serialize, B: DeserializeOwned>(a: A) -> B {
    serde_json::from_value(serde_json::to_value(a).unwrap()).unwrap()
}
// Called when the tool is invoked.
// If you support multiple tools, you must switch on the input.params.name to detect which tool is being called.
pub(crate) fn call(input: types::CallToolRequest) -> Result<types::CallToolResult, Error> {
    return REACTOR.with(|a| a.call(cast(input)).map(cast));
}

// Called by mcpx to understand how and why to use this tool.
// Note: Your servlet configs will not be set when this function is called,
// so do not rely on config in this function
pub(crate) fn describe() -> Result<types::ListToolsResult, Error> {
    return REACTOR.with(|a| a.describe().map(cast));
}
