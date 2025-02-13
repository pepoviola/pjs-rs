use std::borrow::Cow;
use std::path::{Path, PathBuf};

use deno_core::url::Url;
use deno_fetch::FetchPermissions;
use deno_net::NetPermissions;
use deno_permissions::PermissionCheckError;
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
    ) -> Result<(), PermissionCheckError> {
        Ok(())
    }
}

impl FetchPermissions for ZombiePermissions {
    fn check_net_url(&mut self, _url: &Url, _api_name: &str) -> Result<(), PermissionCheckError> {
        Ok(())
    }
    fn check_read<'a>(
        &mut self,
        p: &'a Path,
        _api_name: &str,
    ) -> Result<Cow<'a, Path>, PermissionCheckError> {
        Ok(p.into())
    }
}

impl NetPermissions for ZombiePermissions {
    fn check_net<T: AsRef<str>>(
        &mut self,
        _host: &(T, Option<u16>),
        _api_name: &str,
    ) -> Result<(), PermissionCheckError> {
        Ok(())
    }

    fn check_read(&mut self, p: &str, _api_name: &str) -> Result<PathBuf, PermissionCheckError> {
        Ok(p.into())
    }

    fn check_write(&mut self, p: &str, _api_name: &str) -> Result<PathBuf, PermissionCheckError> {
        Ok(p.into())
    }

    fn check_write_path<'a>(
        &mut self,
        p: &'a Path,
        _api_name: &str,
    ) -> Result<Cow<'a, Path>, PermissionCheckError> {
        Ok(p.into())
    }
}
