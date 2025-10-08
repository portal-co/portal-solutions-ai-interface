pub mod types;
#[cfg(not(feature = "wasm-bidngen"))]
use std::convert::Infallible;
use std::{cell::OnceCell, sync::Mutex};

use anyhow::{Context as AnyhowContext, Error};
use boa_engine::{Context, Source};
use either::Either;
use serde_json::json;
#[cfg(feature = "wasm-bindgen")]
type ToolCallClosure = wasm_bindgen::closure::Closure<
    dyn Fn(wasm_bindgen::JsValue) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
>;
#[derive(Default)]
#[non_exhaustive]
pub struct Reactor {
    ctx: OnceCell<Mutex<Context>>,
    #[cfg(feature = "wasm-bindgen")]
    wbg_ctx: OnceCell<Option<wasm_bindgen::JsValue>>,
    #[cfg(feature = "wasm-bindgen")]
    wbg_tool_call: OnceCell<ToolCallClosure>,
}
impl Reactor {
    fn ctx(&self) -> &Mutex<Context> {
        return self.ctx.get_or_init(|| {
            Mutex::new({
                let mut ctx = Context::builder().build().unwrap();

                ctx
            })
        });
    }
    #[cfg(feature = "wasm-bindgen")]
    fn wbg_ctx(&self) -> &Option<wasm_bindgen::JsValue> {
        return self.wbg_ctx.get_or_init(|| {
            #[wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"
            export function f(s){
                try{
                    return new Function(s)();
                }catch{
                    return;
                }
            }
            "#)]
            extern "C" {
                fn f(s: &str) -> wasm_bindgen::JsValue;
            }
            let f = f("return Function");
            if f.is_falsy() {
                return None;
            }
            return Some(f);
        });
    }
    // Called when the tool is invoked.
    // If you support multiple tools, you must switch on the input.params.name to detect which tool is being called.
    pub fn call(&self, input: types::CallToolRequest) -> Result<types::CallToolResult, Error> {
        let name = &*input.params.name;
        match name {
            "execute_code" => {
                let code = input
                    .params
                    .arguments
                    .as_ref()
                    .and_then(|a| a.get("code"))
                    .and_then(|a| a.as_str())
                    .context("in geting the code")?;
                let gen_code =
                    |code: &str| format!("(async ()=>{{{code}}})().then(a=>JSON.stringify(a))");
                let code = gen_code(code);

                #[cfg(feature = "wasm-bindgen")]
                let w = self.wbg_ctx().as_ref();
                #[cfg(not(feature = "wasm-bindgen"))]
                let w: Option<&Infallible> = None;

                let x = match w {
                    #[cfg(not(feature = "wasm-bindgen"))]
                    Some(a) => Either::Left(match a {
                        _ => async move { loop {} },
                    }),
                    #[cfg(feature = "wasm-bindgen")]
                    Some(a) => {
                        use wasm_bindgen::prelude::Closure;

                        #[wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"
                        async fn go(Function,code,call_tool){
                            return new Function(`call_tool`,`return ${code};`)(call_tool);
                        }
                        "#)]
                        extern "C" {
                            async fn go(
                                a: wasm_bindgen::JsValue,
                                code: &str,
                                call_tool: &ToolCallClosure,
                            ) -> wasm_bindgen::JsValue;
                        }
                        Either::Left(async move {
                            match go(
                                a.clone(),
                                &code,
                                self.wbg_tool_call.get_or_init(|| {
                                    Closure::new(|v| {
                                        Err(wasm_bindgen::JsValue::from_str("invalid tool call"))
                                    })
                                }),
                            )
                            .await
                            {
                                v => v.as_string().unwrap(),
                            }
                        })
                    }
                    _ => match self.ctx().lock().unwrap() {
                        mut l => {
                            let v = l
                                .eval(Source::from_bytes(&code))
                                .map_err(|e| e.into_erased(&mut l))
                                .context("in js evaluation")?;
                            let v = v.as_promise().context("in awaiting the thing")?;
                            let v = v.into_js_future(&mut l);
                            Either::Right(async move {
                                let v = v.await;
                                v.unwrap().as_string().unwrap().display_lossy().to_string()
                            })
                        }
                    },
                };
                let x = async move { x.await };

                todo!()
            }
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
                input_schema: [
                    (format!("type"),json!("object")),
                    (format!("required"),json!(["code"])),
                    (format!("properties"),json!({"code":{"type":"string"}}))
                    ].into_iter().collect()
                }
            ].into_iter().collect(),
        })
    }
}
