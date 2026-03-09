use matchit::{InsertError, Router as MatchRouter};

use super::{Backend, RoutingError};

#[derive(Debug, Default)]
pub struct RouteTrie {
    router: MatchRouter<Backend>,
}

impl RouteTrie {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, path: &str, backend: Backend) -> Result<(), RoutingError> {
        let normalized = normalize_path(path)?;
        if has_legacy_param(&normalized) {
            return Err(RoutingError::InvalidPath(path.to_string()));
        }

        self.router
            .insert(&normalized, backend)
            .map_err(|err| map_insert_error(err, path, &normalized))
    }

    pub fn resolve(&self, path: &str) -> Option<Backend> {
        let normalized = normalize_path(path).ok()?;
        self.router
            .at(&normalized)
            .ok()
            .map(|matched| *matched.value)
    }
}

fn normalize_path(path: &str) -> Result<String, RoutingError> {
    if !path.starts_with('/') {
        return Err(RoutingError::InvalidPath(path.to_string()));
    }

    let mut segments = path.split('/').filter(|s| !s.is_empty());

    let Some(first) = segments.next() else {
        return Ok("/".to_string());
    };

    let mut normalized = String::new();
    normalized.push('/');
    normalized.push_str(first);

    for segment in segments {
        normalized.push('/');
        normalized.push_str(segment);
    }

    Ok(normalized)
}

fn has_legacy_param(path: &str) -> bool {
    path.split('/')
        .filter(|s| !s.is_empty())
        .any(|segment| segment.starts_with(':'))
}

fn map_insert_error(err: InsertError, original: &str, normalized: &str) -> RoutingError {
    match err {
        InsertError::Conflict { with } => {
            if with == normalized {
                RoutingError::DuplicateRoute(original.to_string())
            } else {
                RoutingError::AmbiguousRoute(original.to_string())
            }
        }
        InsertError::InvalidCatchAll
        | InsertError::InvalidParam
        | InsertError::InvalidParamSegment => RoutingError::InvalidPath(original.to_string()),
        _ => RoutingError::InvalidPath(original.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_route_match() {
        let mut trie = RouteTrie::new();
        trie.insert("/blocks/latest", Backend::Dolos).unwrap();

        assert_eq!(trie.resolve("/blocks/latest"), Some(Backend::Dolos));
        assert_eq!(trie.resolve("/blocks/old"), None);
    }

    #[test]
    fn param_route_match() {
        let mut trie = RouteTrie::new();
        trie.insert("/blocks/{hash}", Backend::Dolos).unwrap();

        assert_eq!(trie.resolve("/blocks/abc"), Some(Backend::Dolos));
        assert_eq!(trie.resolve("/blocks/123"), Some(Backend::Dolos));
    }

    #[test]
    fn static_precedence_over_param() {
        let mut trie = RouteTrie::new();
        trie.insert("/blocks/{hash}", Backend::Blockfrost).unwrap();
        trie.insert("/blocks/latest", Backend::Dolos).unwrap();

        assert_eq!(trie.resolve("/blocks/latest"), Some(Backend::Dolos));
        assert_eq!(trie.resolve("/blocks/abc"), Some(Backend::Blockfrost));
    }

    #[test]
    fn duplicate_route_error() {
        let mut trie = RouteTrie::new();
        trie.insert("/blocks/{hash}", Backend::Dolos).unwrap();
        assert!(trie.insert("/blocks/{hash}", Backend::Blockfrost).is_err());
    }

    #[test]
    fn ambiguous_param_error() {
        let mut trie = RouteTrie::new();
        trie.insert("/blocks/{hash}", Backend::Dolos).unwrap();
        assert!(trie.insert("/blocks/{id}", Backend::Blockfrost).is_err());
    }

    #[test]
    fn legacy_param_is_invalid() {
        let mut trie = RouteTrie::new();
        assert!(trie.insert("/blocks/:hash", Backend::Dolos).is_err());
    }

    #[test]
    fn static_route_matches_with_trailing_slash() {
        let mut trie = RouteTrie::new();
        trie.insert("/blocks/latest", Backend::Dolos).unwrap();

        assert_eq!(trie.resolve("/blocks/latest/"), Some(Backend::Dolos));
        assert_eq!(trie.resolve("/blocks/latest///"), Some(Backend::Dolos));
    }

    #[test]
    fn param_route_matches_with_trailing_slash() {
        let mut trie = RouteTrie::new();
        trie.insert("/blocks/{hash}", Backend::Dolos).unwrap();

        assert_eq!(trie.resolve("/blocks/abc/"), Some(Backend::Dolos));
        assert_eq!(trie.resolve("/blocks/abc///"), Some(Backend::Dolos));
    }

    #[test]
    fn trailing_slash_on_insert_normalizes_path() {
        let mut trie = RouteTrie::new();
        trie.insert("/blocks/latest/", Backend::Dolos).unwrap();

        assert_eq!(trie.resolve("/blocks/latest"), Some(Backend::Dolos));
        assert_eq!(trie.resolve("/blocks/latest/"), Some(Backend::Dolos));
    }
}
