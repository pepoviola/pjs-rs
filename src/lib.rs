use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::fs;


use deno_core::JsRuntime;
use deno_core::error::AnyError;
use deno_core::serde_json;
use deno_core::serde_json::json;
use deno_core::serde_v8;
use deno_core::v8;

use deno_tls::rustls::RootCertStore;

mod perms;
use perms::ZombiePermissions;

mod cert;
use cert::ValueRootCertStoreProvider;

mod ext;
use ext::pjs_extension;


pub type DynLoader = Rc<dyn deno_core::ModuleLoader + 'static>;
pub fn create_runtime_with_loader(loader: Option<DynLoader>) -> JsRuntime {
    let js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: if let Some(loader) = loader { Some(loader) } else {Some(Rc::new(deno_core::NoopModuleLoader))},
        startup_snapshot: None,
        extensions: vec![
            deno_console::deno_console::init_ops_and_esm(),
            deno_webidl::deno_webidl::init_ops_and_esm(),
            deno_url::deno_url::init_ops_and_esm(),
            deno_web::deno_web::init_ops_and_esm::<ZombiePermissions>(Arc::new(deno_web::BlobStore::default()),None),
            deno_crypto::deno_crypto::init_ops_and_esm(None),
            deno_fetch::deno_fetch::init_ops_and_esm::<ZombiePermissions>(
                deno_fetch::Options {
                  user_agent: "zombienet-agent".to_string(),
                  root_cert_store_provider: Some(Arc::new(ValueRootCertStoreProvider(RootCertStore::empty()))),
                  unsafely_ignore_certificate_errors: Some(vec![]),
                  file_fetch_handler: Rc::new(deno_fetch::FsFetchHandler),
                  ..Default::default()
                },
              ),
            deno_websocket::deno_websocket::init_ops_and_esm::<ZombiePermissions>(
                "zombienet-agent".to_string(),
                Some(Arc::new(ValueRootCertStoreProvider(RootCertStore::empty()))),
                Some(vec![])
              ),
              pjs_extension::init_ops_and_esm(),
        ],
        ..Default::default()
    });

    js_runtime
}


#[derive(Debug, PartialEq)]
pub enum ReturnValue {
    Deserialized(serde_json::Value),
    CantDeserialize(String),
}


/// No loader
pub async fn run_file(file_path: impl AsRef<Path>, json_args: Option<Vec<serde_json::Value>>) -> Result<ReturnValue, AnyError> {
    let mut js_runtime = create_runtime_with_loader(None);

    let code_content = fs::read_to_string(file_path)?;
    let arg_to_pass = if let Some(json_value) = json_args {
        json_value
    } else {
        // just pass null
        vec![json!(serde_json::Value::Null)]
    };
    let code = format!(
    r#"(async (arguments) => {{
        {}
    }})({})"#,
    code_content, json!(arg_to_pass));
    log::trace!("code: \n{}", code);
    let executed = js_runtime.execute_script("name", deno_core::FastString::from(code))?;
    let resolved = js_runtime.resolve_value(executed).await;
    match resolved {
        Ok(global) => {
          let scope = &mut js_runtime.handle_scope();
          let local = v8::Local::new(scope, global);
          // Deserialize a `v8` object into a Rust type using `serde_v8`,
          // in this case deserialize to a JSON `Value`.
          let deserialized_value =
            serde_v8::from_v8::<serde_json::Value>(scope, local);

            let resp = match deserialized_value {
                Ok(value) => {
                    log::debug!("{:#?}", value);
                    ReturnValue::Deserialized(value)
                },
                Err(err) => {
                    log::warn!("{}", format!("Cannot deserialize value: {:?}", err));
                    ReturnValue::CantDeserialize(err.to_string())
                }
            };

            Ok(resp)
        },
        Err(err) => {
            log::error!("{}",format!("Evaling error: {:?}", err));
            Err(err)
        },
      }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn query_parachains_works() {
        let resp = run_file("./testing/query_parachains.js", None).await.unwrap();
        if let ReturnValue::Deserialized(value) = resp {
            let first_para_id = value.as_array().unwrap().first().unwrap().as_u64().unwrap();
            assert_eq!(first_para_id, 1000_u64);
        }
    }

    #[tokio::test]
    async fn query_historic_data_rococo_works() {
        run_file("./testing/query_historic_data.js", None).await.unwrap();
    }

    #[tokio::test]
    async fn query_chain_state_info_rococo_works() {
        run_file("./testing/get_chain_state_info.js", Some(vec![json!("wss://rococo-rpc.polkadot.io")])).await.unwrap();
    }

    #[tokio::test]
    async fn listen_new_head_works() {
        let resp = run_file("./testing/rpc_listen_new_head.js", Some(vec![json!("wss://rococo-rpc.polkadot.io")])).await.unwrap();
        assert_eq!(resp, ReturnValue::Deserialized(json!(5)));
    }

    #[tokio::test]
    async fn transfer_works() {
        let args = vec![
            json!("wss://rococo-rpc.polkadot.io"),
            json!("//Alice"),
        ];
        run_file("./testing/transfer.js", Some(args)).await.unwrap();
    }
}
