use deno_core::error::AnyError;
use deno_tls::rustls::RootCertStore;
use deno_tls::RootCertStoreProvider;
pub struct ValueRootCertStoreProvider(pub RootCertStore);

impl RootCertStoreProvider for ValueRootCertStoreProvider {
    fn get_or_try_init(&self) -> Result<&RootCertStore, AnyError> {
        Ok(&self.0)
    }
}
