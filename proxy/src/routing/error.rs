use thiserror::Error;

#[derive(Debug, Error)]
pub enum RoutingError {
    #[error("unknown backend: {0}")]
    UnknownBackend(String),

    #[error("invalid route path: {0}")]
    InvalidPath(String),

    #[error("duplicate route definition: {0}")]
    DuplicateRoute(String),

    #[error("ambiguous route definition: {0}")]
    AmbiguousRoute(String),
}
