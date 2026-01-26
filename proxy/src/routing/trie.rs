use std::collections::HashMap;

use super::{Backend, RoutingError};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Static(String),
    Param(String),
}

#[derive(Debug, Default)]
struct Node {
    static_children: HashMap<String, Node>,
    param_child: Option<(String, Box<Node>)>,
    backend: Option<Backend>,
}

#[derive(Debug, Default)]
pub struct RouteTrie {
    root: Node,
}

impl RouteTrie {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, path: &str, backend: Backend) -> Result<(), RoutingError> {
        let segments = parse_path(path)?;
        let mut current = &mut self.root;

        for segment in segments {
            match segment {
                Segment::Static(value) => {
                    current = current.static_children.entry(value).or_default();
                }
                Segment::Param(name) => {
                    if let Some((existing, _)) = &current.param_child {
                        if existing != &name {
                            return Err(RoutingError::AmbiguousRoute(path.to_string()));
                        }
                    }

                    let child = current
                        .param_child
                        .get_or_insert_with(|| (name, Box::new(Node::default())));

                    current = &mut child.1;
                }
            }
        }

        if current.backend.is_some() {
            return Err(RoutingError::DuplicateRoute(path.to_string()));
        }

        current.backend = Some(backend);
        Ok(())
    }

    pub fn resolve(&self, path: &str) -> Option<Backend> {
        let segments = path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty());

        let mut current = &self.root;

        for segment in segments {
            if let Some(next) = current.static_children.get(segment) {
                current = next;
            } else if let Some((_, param)) = &current.param_child {
                current = param;
            } else {
                return None;
            }
        }

        current.backend
    }
}

fn parse_path(path: &str) -> Result<Vec<Segment>, RoutingError> {
    if !path.starts_with('/') {
        return Err(RoutingError::InvalidPath(path.to_string()));
    }

    let mut segments = Vec::new();

    for part in path.trim_start_matches('/').split('/') {
        if part.is_empty() {
            continue;
        }

        if part.starts_with(':') {
            let name = part.trim_start_matches(':');
            if name.is_empty() {
                return Err(RoutingError::InvalidPath(path.to_string()));
            }
            segments.push(Segment::Param(name.to_string()));
        } else if part.contains('*') || part.contains('{') || part.contains('}') {
            return Err(RoutingError::InvalidPath(path.to_string()));
        } else {
            segments.push(Segment::Static(part.to_string()));
        }
    }

    Ok(segments)
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
        trie.insert("/blocks/:hash", Backend::Dolos).unwrap();

        assert_eq!(trie.resolve("/blocks/abc"), Some(Backend::Dolos));
        assert_eq!(trie.resolve("/blocks/123"), Some(Backend::Dolos));
    }

    #[test]
    fn static_precedence_over_param() {
        let mut trie = RouteTrie::new();
        trie.insert("/blocks/:hash", Backend::Blockfrost).unwrap();
        trie.insert("/blocks/latest", Backend::Dolos).unwrap();

        assert_eq!(trie.resolve("/blocks/latest"), Some(Backend::Dolos));
        assert_eq!(trie.resolve("/blocks/abc"), Some(Backend::Blockfrost));
    }

    #[test]
    fn duplicate_route_error() {
        let mut trie = RouteTrie::new();
        trie.insert("/blocks/:hash", Backend::Dolos).unwrap();
        assert!(trie.insert("/blocks/:hash", Backend::Blockfrost).is_err());
    }

    #[test]
    fn ambiguous_param_error() {
        let mut trie = RouteTrie::new();
        trie.insert("/blocks/:hash", Backend::Dolos).unwrap();
        assert!(trie.insert("/blocks/:id", Backend::Blockfrost).is_err());
    }
}
