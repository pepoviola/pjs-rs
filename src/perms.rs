use std::path::Path;

use deno_core::error::AnyError;
use deno_core::url::Url;
use deno_fetch::FetchPermissions;
use deno_web::TimersPermission;
use deno_websocket::WebSocketPermissions;

pub struct ZombiePermissions;

impl TimersPermission for ZombiePermissions {
    fn allow_hrtime(&mut self) -> bool {
        // Disable high-resolution time management.
        //
        // Quoting from https://v8.dev/docs/untrusted-code-mitigations
        // > A high-precision timer makes it easier to observe side channels in the SSCA
        // > vulnerability. If your product offers high-precision timers that can be accessed by
        // > untrusted JavaScript or WebAssembly code, consider making these timers more coarse or
        // > adding jitter to them.
        false
    }
}

impl WebSocketPermissions for ZombiePermissions {
    fn check_net_url(
        &mut self,
        _url: &deno_core::url::Url,
        _api_name: &str,
      ) -> Result<(), AnyError> {
        Ok(())
    }
}

impl FetchPermissions for ZombiePermissions {
    fn check_net_url(&mut self, _url: &Url, _api_name: &str) -> Result<(), AnyError> {
        Ok(())
    }
    fn check_read(&mut self, _p: &Path, _api_name: &str) -> Result<(), AnyError> {
        Ok(())
    }
}