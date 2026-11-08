use cr_errors::{RelayError, Result};
use opendal::{services, Operator};
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
#[derive(Clone)]
pub struct ArtifactStore {
    operator: Operator,
}
impl ArtifactStore {
    pub fn memory() -> Result<Self> {
        Ok(Self {
            operator: Operator::new(services::Memory::default())
                .map_err(|e| RelayError::Storage(e.to_string()))?,
        })
    }
    pub fn filesystem(root: &Path) -> Result<Self> {
        std::fs::create_dir_all(root).map_err(|e| RelayError::Storage(e.to_string()))?;
        let b = services::Fs::default().root(root.to_string_lossy().as_ref());
        Ok(Self {
            operator: Operator::new(b).map_err(|e| RelayError::Storage(e.to_string()))?,
        })
    }
    fn key(ns: &str, kind: &str, id: &str) -> Result<String> {
        if [ns, kind, id]
            .iter()
            .any(|s| s.is_empty() || s.contains("..") || s.contains('/'))
        {
            return Err(RelayError::Validation("unsafe key".into()));
        }
        Ok(format!("{ns}/{kind}/{id}.json"))
    }
    pub async fn put<T: Serialize>(&self, ns: &str, kind: &str, id: &str, v: &T) -> Result<()> {
        self.operator
            .write(
                &Self::key(ns, kind, id)?,
                serde_json::to_vec(v).map_err(|e| RelayError::Integrity(e.to_string()))?,
            )
            .await
            .map_err(|e| RelayError::Storage(e.to_string()))?;
        Ok(())
    }
    pub async fn get<T: DeserializeOwned>(&self, ns: &str, kind: &str, id: &str) -> Result<T> {
        let b = self
            .operator
            .read(&Self::key(ns, kind, id)?)
            .await
            .map_err(|e| RelayError::NotFound(e.to_string()))?;
        serde_json::from_slice(&b.to_vec()).map_err(|e| RelayError::Integrity(e.to_string()))
    }
    pub async fn list(&self, ns: &str, kind: &str) -> Result<Vec<String>> {
        let mut e = self
            .operator
            .list(&format!("{ns}/{kind}/"))
            .await
            .map_err(|e| RelayError::Storage(e.to_string()))?;
        e.sort_by(|a, b| a.path().cmp(b.path()));
        Ok(e.into_iter().map(|x| x.path().to_owned()).collect())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn roundtrip() {
        let s = ArtifactStore::memory().unwrap();
        s.put("t", "audit", "a", &1).await.unwrap();
        let v: i32 = s.get("t", "audit", "a").await.unwrap();
        assert_eq!(v, 1)
    }
    #[tokio::test]
    async fn unsafe_or_missing_objects_fail_closed() {
        let store = ArtifactStore::memory().unwrap();
        assert!(store.put("../escape", "audit", "a", &1).await.is_err());
        assert!(store.get::<i32>("t", "audit", "missing").await.is_err());
    }
}
