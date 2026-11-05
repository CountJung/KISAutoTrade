//! Shared HTTP TLS policy: the same bundled Mozilla roots and ring provider as
//! reqwest 0.12's rustls-tls feature, without consulting the OS certificate store.
use std::sync::Arc;

use reqwest::{Client, ClientBuilder};
use rustls::{ClientConfig, RootCertStore};

/// Preserve reqwest's request defaults; callers retain their own timeout,
/// cookie, proxy, header, redirect and retry settings.
pub(crate) fn client_builder() -> ClientBuilder {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    builder_with_roots(roots)
}

/// Equivalent to Client::new with the application's unchanged TLS trust policy.
pub(crate) fn client() -> Client {
    client_builder().build().expect("HTTP client should build")
}

fn builder_with_roots(roots: RootCertStore) -> ClientBuilder {
    // Select ring per client. Never install a process-wide crypto provider or
    // use reqwest 0.13's platform verifier / system certificate store.
    let mut tls =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .expect("ring supports the default TLS 1.2 and TLS 1.3 versions")
            .with_root_certificates(roots)
            .with_no_client_auth();
    // The previous resolved reqwest feature set had HTTP/1.1 only. Explicit
    // preconfigured TLS bypasses reqwest's automatic ALPN configuration.
    tls.alpn_protocols = vec![b"http/1.1".to_vec()];
    Client::builder().tls_backend_preconfigured(tls)
}

#[cfg(test)]
mod tests;
