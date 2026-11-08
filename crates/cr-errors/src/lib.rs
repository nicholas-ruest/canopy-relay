use thiserror::Error;
#[derive(Debug, Error)]
pub enum RelayError {
    #[error("validation: {0}")]
    Validation(String),
    #[error("unauthenticated: {0}")]
    Unauthenticated(String),
    #[error("denied: {0}")]
    Denied(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("upstream {upstream}: {detail}")]
    Upstream {
        upstream: &'static str,
        detail: String,
    },
    #[error("storage: {0}")]
    Storage(String),
    #[error("integrity: {0}")]
    Integrity(String),
    #[error("timeout: {0}")]
    Timeout(String),
}
impl RelayError {
    pub fn upstream(upstream: &'static str, detail: impl Into<String>) -> Self {
        Self::Upstream {
            upstream,
            detail: detail.into(),
        }
    }
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation",
            Self::Unauthenticated(_) => "unauthenticated",
            Self::Denied(_) => "denied",
            Self::Conflict(_) => "conflict",
            Self::NotFound(_) => "not_found",
            Self::Upstream { .. } => "upstream",
            Self::Storage(_) => "storage",
            Self::Integrity(_) => "integrity",
            Self::Timeout(_) => "timeout",
        }
    }
}
pub type Result<T> = std::result::Result<T, RelayError>;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_code_hides_detail() {
        assert_eq!(
            RelayError::upstream("worldgraph", "secret").code(),
            "upstream"
        )
    }
}
