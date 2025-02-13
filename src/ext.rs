use crate::perms::ZombiePermissions;
use deno_core::error::CoreError;
use deno_core::op2;

#[op2(async)]
async fn op_set_timeout(#[bigint] delay: u64) -> Result<(), CoreError> {
    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    Ok(())
}

deno_core::extension!(
    pjs_extension,
    deps = [deno_url, deno_web, deno_websocket, deno_crypto],
    esm_entry_point = "ext:pjs_extension/src/runtime.js",
    esm = ["src/js/06_util.js", "src/js/07.js", "src/runtime.js",],
    state = |state| {
        state.put(ZombiePermissions {});
    }
);
