use super::config::BackendTemplateConfig;
use super::trie::RouteTrie;
use super::Backend;

#[derive(Debug)]
pub struct Router {
    default_backend: Backend,
    trie: RouteTrie,
    backend_templates: BackendTemplateConfig,
}

impl Default for Router {
    fn default() -> Self {
        Self {
            default_backend: Backend::Blockfrost,
            trie: RouteTrie::new(),
            backend_templates: BackendTemplateConfig::default(),
        }
    }
}

impl Router {
    pub fn new(
        default_backend: Backend,
        trie: RouteTrie,
        backend_templates: BackendTemplateConfig,
    ) -> Self {
        Self {
            default_backend,
            trie,
            backend_templates,
        }
    }

    pub fn resolve(&self, path: &str) -> Backend {
        self.trie.resolve(path).unwrap_or(self.default_backend)
    }

    pub fn backend_template(&self, backend: Backend) -> &str {
        match backend {
            Backend::Blockfrost => &self.backend_templates.blockfrost,
            Backend::Dolos => &self.backend_templates.dolos,
            Backend::SubmitApi => &self.backend_templates.submitapi,
        }
    }
}
