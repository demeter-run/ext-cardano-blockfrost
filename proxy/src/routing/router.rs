use super::config::BackendsConfig;
use super::trie::RouteTrie;
use super::Backend;

#[derive(Debug)]
pub struct Router {
    default_backend: Backend,
    trie: RouteTrie,
    backends: BackendsConfig,
}

impl Default for Router {
    fn default() -> Self {
        Self {
            default_backend: Backend::Blockfrost,
            trie: RouteTrie::new(),
            backends: BackendsConfig::default(),
        }
    }
}

impl Router {
    pub fn new(default_backend: Backend, trie: RouteTrie, backends: BackendsConfig) -> Self {
        Self {
            default_backend,
            trie,
            backends,
        }
    }

    pub fn resolve(&self, path: &str) -> Backend {
        self.trie.resolve(path).unwrap_or(self.default_backend)
    }

    pub fn default_backend(&self) -> Backend {
        self.default_backend
    }

    pub fn backend_template(&self, backend: Backend) -> &str {
        match backend {
            Backend::Blockfrost => &self.backends.blockfrost.template,
            Backend::Dolos => &self.backends.dolos.template,
            Backend::SubmitApi => &self.backends.submitapi.template,
        }
    }

    pub fn supports_network(&self, backend: Backend, network: &str) -> bool {
        match backend {
            Backend::Blockfrost => self.backends.blockfrost.supports_network(network),
            Backend::Dolos => self.backends.dolos.supports_network(network),
            Backend::SubmitApi => self.backends.submitapi.supports_network(network),
        }
    }
}
