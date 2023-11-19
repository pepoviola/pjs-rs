use crate::perms::ZombiePermissions;
use deno_core::error::AnyError;
use deno_core::op2;

#[op2(async)]
async fn op_set_timeout(#[bigint] delay: u64) -> Result<(), AnyError> {
    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    Ok(())
}

deno_core::extension!(
    pjs_extension,
    deps = [deno_url, deno_web, deno_websocket, deno_crypto],
    ops = [op_set_timeout],
    esm_entry_point = "ext:pjs_extension/src/runtime.js",
    esm = [
        "src/js/06_util.js",
        "src/js/pjs_bundle.js",
        "src/runtime.js",
    ],
    state = |state| {
        state.put(ZombiePermissions {});
    }
);
