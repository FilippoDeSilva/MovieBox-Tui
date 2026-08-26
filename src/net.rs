use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

use hickory_resolver::TokioResolver;
use hickory_resolver::config::{
    CLOUDFLARE_IPS, GOOGLE_IPS, LookupIpStrategy, NameServerConfigGroup, QUAD9_IPS, ResolverConfig,
};
use hickory_resolver::name_server::TokioConnectionProvider;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};

const FALLBACK_DNS_PORT: u16 = 53;

#[derive(Debug, Default, Clone)]
pub struct FallbackResolver {
    inner: Arc<OnceLock<Arc<TokioResolver>>>,
}

impl FallbackResolver {
    pub fn new() -> Self {
        Self::default()
    }
}

fn fallback_servers() -> Vec<std::net::IpAddr> {
    let mut servers = Vec::with_capacity(CLOUDFLARE_IPS.len() + GOOGLE_IPS.len() + QUAD9_IPS.len());
    servers.extend_from_slice(CLOUDFLARE_IPS);
    servers.extend_from_slice(GOOGLE_IPS);
    servers.extend_from_slice(QUAD9_IPS);
    servers
}

fn fallback_config() -> ResolverConfig {
    ResolverConfig::from_parts(
        None,
        Vec::new(),
        NameServerConfigGroup::from_ips_clear(&fallback_servers(), FALLBACK_DNS_PORT, true),
    )
}

fn build_resolver() -> TokioResolver {
    let mut builder = match TokioResolver::builder_tokio() {
        Ok(builder) => builder,
        Err(_) => TokioResolver::builder_with_config(
            fallback_config(),
            TokioConnectionProvider::default(),
        ),
    };
    builder.options_mut().ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
    builder.build()
}

impl Resolve for FallbackResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let inner = Arc::clone(&self.inner);
        Box::pin(async move {
            let resolver = inner.get_or_init(|| Arc::new(build_resolver()));
            let lookup = resolver.lookup_ip(name.as_str()).await?;
            let addrs: Addrs = Box::new(
                lookup
                    .into_iter()
                    .map(|address| SocketAddr::new(address, 0)),
            );
            Ok(addrs)
        })
    }
}

pub fn http_client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder().dns_resolver(Arc::new(FallbackResolver::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_config_contains_public_resolvers() {
        let config = fallback_config();
        let servers = config.name_servers();
        assert!(servers.len() >= 12);
        assert!(servers.iter().any(|server| server.socket_addr.ip()
            == std::net::IpAddr::V4(std::net::Ipv4Addr::new(1, 1, 1, 1))));
        assert!(
            servers
                .iter()
                .any(|server| server.socket_addr.port() == FALLBACK_DNS_PORT)
        );
    }

    #[test]
    fn fallback_servers_deduplicate_nothing_and_cover_all_providers() {
        let servers = fallback_servers();
        assert_eq!(
            servers.len(),
            CLOUDFLARE_IPS.len() + GOOGLE_IPS.len() + QUAD9_IPS.len()
        );
    }

    #[tokio::test]
    async fn built_resolver_prefers_ipv4_and_ipv6_lookup() {
        let resolver = build_resolver();
        assert_eq!(
            resolver.options().ip_strategy,
            LookupIpStrategy::Ipv4AndIpv6
        );
    }

    #[test]
    fn clones_share_lazy_state_slot() {
        let original = FallbackResolver::new();
        let clone = original.clone();
        assert!(original.inner.get().is_none());
        assert!(clone.inner.get().is_none());
    }
}
