use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_core::error::AnyError;
use deno_core::serde_json;
use deno_core::serde_json::json;
use deno_core::serde_v8;
use deno_core::url;
use deno_core::v8;
use deno_core::JsRuntime;

mod perms;
use perms::ZombiePermissions;

// Not used anymore
// mod cert;
// use cert::ValueRootCertStoreProvider;

mod ext;
use ext::pjs_extension;

type DynLoader = Rc<dyn deno_core::ModuleLoader + 'static>;

#[derive(Debug, PartialEq)]
/// Js script return value, mapping types that can be deserialized as [serde_json::Value] or
/// not, for the latest an string is returned witht the error message.
pub enum ReturnValue {
    Deserialized(serde_json::Value),
    CantDeserialize(String),
}

/// Create a new runtime with the pjs extension built-in.
/// Allow to pass a [ModuleLoader](deno_core::ModuleLoader) to use, by default
/// [NoopModuleLoader](deno_core::NoopModuleLoader) is used.
pub fn create_runtime_with_loader(loader: Option<DynLoader>) -> JsRuntime {
    deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: if let Some(loader) = loader {
            Some(loader)
        } else {
            Some(Rc::new(deno_core::NoopModuleLoader))
        },
        startup_snapshot: None,
        extensions: vec![
            deno_console::deno_console::init_ops_and_esm(),
            deno_webidl::deno_webidl::init_ops_and_esm(),
            deno_url::deno_url::init_ops_and_esm(),
            deno_web::deno_web::init_ops_and_esm::<ZombiePermissions>(
                Arc::new(deno_web::BlobStore::default()),
                None,
            ),
            deno_crypto::deno_crypto::init_ops_and_esm(None),
            deno_fetch::deno_fetch::init_ops_and_esm::<ZombiePermissions>(deno_fetch::Options {
                user_agent: "zombienet-agent".to_string(),
                ..Default::default()
            }),
            deno_net::deno_net::init_ops_and_esm::<ZombiePermissions>(None, None),
            deno_websocket::deno_websocket::init_ops_and_esm::<ZombiePermissions>(
                "zombienet-agent".to_string(),
                None,
                None,
            ),
            pjs_extension::init_ops_and_esm(),
        ],
        ..Default::default()
    })
}

/// Run a js/ts code from file in an isolated runtime with polkadotjs bundles embedded
///
/// The code runs without any wrapping clousure, and the arguments are binded to the `pjs` Object,
/// available as `pjs.arguments`. If you want to to use top level `await` you will need to wrap your
/// code like we do in [run_file_with_wrap].
///
/// # Example
/// ## Javascript file example
/// ```javascript
/// return pjs.api.consts.babe.epochDuration;
/// ```
/// NOTE: To return a value you need to **explicit** call `return <value>`
///
///
///
/// ## Executing:
/// ```rust
/// # use pjs_rs::run_file;
/// # use deno_core::error::AnyError;
/// # async fn example() -> Result<(), AnyError> {
/// let resp = run_file("./testing/epoch_duration_rococo.js", None).await?;
/// # Ok(())
/// # }
/// ```
///
///
pub async fn run_file(
    file_path: impl AsRef<Path>,
    json_args: Option<Vec<serde_json::Value>>,
) -> Result<ReturnValue, AnyError> {
    let code_content = get_code(file_path).await?;
    run_code(code_content, json_args, false).await
}

/// Run a js/ts code from file in an isolated runtime with polkadotjs bundles embedded
///
/// The code runs in a closure where the `json_args` are passed as `arguments` array
/// and the polkadotjs modules (util, utilCrypto, keyring, types) are exposed from the global `pjs` Object.
/// `ApiPromise` and `WsProvider` are also availables to easy access.
///
/// All code is wrapped within an async closure,
/// allowing access to api methods, keyring, types, util, utilCrypto.
/// ```javascript
/// (async (arguments, ApiPromise, WsProvider, util, utilCrypto, keyring, types) => {
///   ... any user code is executed here ...
/// })();
/// ```
///
/// # Example
/// ## Javascript file example
/// ```javascript
/// const api = await ApiPromise.create({ provider: new pjs.api.WsProvider('wss://rpc.polkadot.io') });
/// const parachains = (await api.query.paras.parachains()) || [];
/// console.log("parachain ids in polkadot:", parachains);
/// return parachains.toJSON();
/// ```
/// NOTE: To return a value you need to **explicit** call `return <value>`
///
///
///
/// ## Executing:
/// ```rust
/// # use pjs_rs::run_file_with_wrap;
/// # use deno_core::error::AnyError;
/// # async fn example() -> Result<(), AnyError> {
/// let resp = run_file_with_wrap("./testing/query_parachains.js", None).await?;
/// # Ok(())
/// # }
/// ```
///
///
pub async fn run_file_with_wrap(
    file_path: impl AsRef<Path>,
    json_args: Option<Vec<serde_json::Value>>,
) -> Result<ReturnValue, AnyError> {
    let code_content = get_code(file_path).await?;
    run_code(&code_content, json_args, true).await
}

pub async fn run_js_code(
    code_content: impl Into<String>,
    json_args: Option<Vec<serde_json::Value>>,
) -> Result<ReturnValue, AnyError> {
    run_code(code_content, json_args, false).await
}

pub async fn run_ts_code(
    code_content: impl Into<String>,
    json_args: Option<Vec<serde_json::Value>>,
) -> Result<ReturnValue, AnyError> {
    let transpiled = transpile(code_content)?;
    run_code(transpiled, json_args, false).await
}

fn transpile(code: impl Into<String>) -> Result<String, AnyError> {
    let parsed = deno_ast::parse_module(ParseParams {
        specifier: url::Url::parse("file:///inner")?,
        text: Arc::from(code.into()),
        media_type: MediaType::TypeScript,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })?;

    let transpiled = parsed.transpile(&Default::default(), &Default::default(), &Default::default())?;
    Ok(transpiled.into_source().text)
}
async fn get_code(file_path: impl AsRef<Path>) -> Result<String, AnyError> {
    let content = fs::read_to_string(file_path.as_ref())?;

    // Check if we need to transpile (e.g .ts file)
    let code_content = if let MediaType::TypeScript = MediaType::from_path(file_path.as_ref()) {
        transpile(&content)?
    } else {
        content
    };

    Ok(code_content)
}

async fn run_code(
    code_content: impl Into<String>,
    json_args: Option<Vec<serde_json::Value>>,
    use_wrapper: bool,
) -> Result<ReturnValue, AnyError> {
    let code_content = code_content.into();
    let bundle_util = include_str!("js/bundle-polkadot-util.js");
    let bundle_util_crypto = include_str!("js/bundle-polkadot-util-crypto.js");
    let bundle_keyring = include_str!("js/bundle-polkadot-keyring.js");
    let bundle_types = include_str!("js/bundle-polkadot-types.js");
    let bundle_api = include_str!("js/bundle-polkadot-api.js");
    // Code templates
    let code = if use_wrapper {
        format!(
            r#"
        const {{ ApiPromise, WsProvider }} = pjs.api;
        const {{ util, utilCrypto, keyring, types }} = pjs;
        (async (arguments, ApiPromise, WsProvider, util, utilCrypto, keyring, types) => {{
            {}
        }})({}, ApiPromise, WsProvider, util, utilCrypto, keyring, types)"#,
            &code_content,
            json!(json_args.unwrap_or_default())
        )
    } else {
        format!(
            r#"
            pjs.arguments = {};
            {}
            "#,
            json!(json_args.unwrap_or_default()),
            &code_content
        )
    };

    log::trace!("code: \n{}", code);

    let mut js_runtime = create_runtime_with_loader(None);
    let with_bundle = format!(
        "
    {}
    {}
    {}
    {}
    {}

    let pjs = {{
        util: polkadotUtil,
        utilCrypto: polkadotUtilCrypto,
        keyring: polkadotKeyring,
        types: polkadotTypes,
        api: polkadotApi,
    }};

    {}
    ",
        bundle_util, bundle_util_crypto, bundle_keyring, bundle_types, bundle_api, code
    );
    log::trace!("full code: \n{}", with_bundle);
    execute_script(&mut js_runtime, &with_bundle).await
}
async fn execute_script(
    js_runtime: &mut JsRuntime,
    code: impl Into<String>,
) -> Result<ReturnValue, AnyError> {
    // Execution
    let executed = js_runtime.execute_script("name", deno_core::FastString::from(code.into()))?;
    let resolve = js_runtime.resolve(executed);
    let resolved = js_runtime
        .with_event_loop_promise(resolve, deno_core::PollEventLoopOptions::default())
        .await;
    match resolved {
        Ok(global) => {
            let scope = &mut js_runtime.handle_scope();
            let local = v8::Local::new(scope, global);
            // Deserialize a `v8` object into a Rust type using `serde_v8`,
            // in this case deserialize to a JSON `Value`.
            let deserialized_value = serde_v8::from_v8::<serde_json::Value>(scope, local);

            let resp = match deserialized_value {
                Ok(value) => {
                    log::debug!("{:#?}", value);
                    ReturnValue::Deserialized(value)
                }
                Err(err) => {
                    log::warn!("{}", format!("Cannot deserialize value: {:?}", err));
                    ReturnValue::CantDeserialize(err.to_string())
                }
            };

            Ok(resp)
        }
        Err(err) => {
            log::error!("{}", format!("Evaling error: {:?}", err));
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn query_parachains_works() {
        let resp = run_file_with_wrap("./testing/query_parachains.ts", None)
            .await
            .unwrap();
        if let ReturnValue::Deserialized(value) = resp {
            let first_para_id = value.as_array().unwrap().first().unwrap().as_u64().unwrap();
            assert_eq!(first_para_id, 1000_u64);
        }
    }

    #[tokio::test]
    async fn consts_works() {
        let resp = run_file("./testing/epoch_duration_rococo.js", None)
            .await
            .unwrap();

        println!("{:#?}", resp);
        assert!(matches!(resp, ReturnValue::Deserialized { .. }));
        if let ReturnValue::Deserialized(value) = resp {
            assert_eq!(value, json!(600));
        }
    }

    #[tokio::test]
    async fn query_parachains_without_wrap_works() {
        let resp = run_file("./testing/query_parachains_no_wrap.ts", None)
            .await
            .unwrap();
        assert!(matches!(resp, ReturnValue::Deserialized { .. }));
        if let ReturnValue::Deserialized(value) = resp {
            println!("{:?}", value);
            let first_para_id = value.as_array().unwrap().first().unwrap().as_u64().unwrap();
            assert_eq!(first_para_id, 1000_u64);
        }
    }

    #[tokio::test]
    async fn query_parachains_from_ts_code_works() {
        let ts_code = r#"
        (async () => {
            const api = await pjs.api.ApiPromise.create({ provider: new pjs.api.WsProvider('wss://rpc.polkadot.io') });
            const parachains: number[] = (await api.query.paras.parachains()) || [];

            return parachains.toJSON();
        })();
        "#;
        let resp = run_ts_code(ts_code, None).await.unwrap();
        assert!(matches!(resp, ReturnValue::Deserialized { .. }));
        if let ReturnValue::Deserialized(value) = resp {
            let first_para_id = value.as_array().unwrap().first().unwrap().as_u64().unwrap();
            assert_eq!(first_para_id, 1000_u64);
        }
    }

    #[tokio::test]
    async fn query_parachains_from_js_code_works() {
        let ts_code = r#"
        (async () => {
            const api = await pjs.api.ApiPromise.create({ provider: new pjs.api.WsProvider('wss://rpc.polkadot.io') });
            const parachains = (await api.query.paras.parachains()) || [];

            return parachains.toJSON();
        })();
        "#;
        let resp = run_ts_code(ts_code, None).await.unwrap();
        assert!(matches!(resp, ReturnValue::Deserialized { .. }));
        if let ReturnValue::Deserialized(value) = resp {
            let first_para_id = value.as_array().unwrap().first().unwrap().as_u64().unwrap();
            assert_eq!(first_para_id, 1000_u64);
        }
    }

    #[tokio::test]
    async fn query_historic_data_rococo_works() {
        run_file_with_wrap("./testing/query_historic_data.js", None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn query_chain_state_info_rococo_works() {
        run_file_with_wrap(
            "./testing/get_chain_state_info.js",
            Some(vec![json!("wss://paseo-rpc.dwellir.com")]),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn listen_new_head_works() {
        let resp = run_file_with_wrap(
            "./testing/rpc_listen_new_head.js",
            Some(vec![json!("wss://paseo-rpc.dwellir.com")]),
        )
        .await
        .unwrap();
        assert_eq!(resp, ReturnValue::Deserialized(json!(5)));
    }

    #[tokio::test]
    async fn transfer_works() {
        let args = vec![json!("wss://paseo-rpc.dwellir.com"), json!("//Alice")];
        let resp = run_file_with_wrap("./testing/transfer.js", Some(args.clone()))
            .await
            .unwrap();

        assert!(matches!(resp, ReturnValue::Deserialized { .. }));

        if let ReturnValue::Deserialized(value) = resp {
            let amount = value.as_u64().unwrap();
            println!("Returning {amount:?} to Bob");
            let args = [args,vec![json!(amount)]].concat();
            run_file_with_wrap("./testing/transfer.js", Some(args))
            .await
            .unwrap();
        }
    }
}
