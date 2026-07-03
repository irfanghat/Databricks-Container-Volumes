use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("internal error")]
    Internal,
}
