use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("invalid bind address: {0}")]
  InvalidAddr(#[from] std::net::AddrParseError),

  #[error("failed to bind socket: {0}")]
  BindFailed(#[source] std::io::Error),

  #[error("server runtime error: {0}")]
  ServeFailed(#[source] std::io::Error),

  #[error(transparent)]
  Other(#[from] anyhow::Error),
}

impl AppError {
  pub fn code(&self) -> u8 {
    match self {
      AppError::InvalidAddr(_) => 2,

      AppError::BindFailed(e) => match e.kind() {
        std::io::ErrorKind::AddrInUse => 20,
        std::io::ErrorKind::PermissionDenied => 21,
        _ => 3,
      },

      AppError::ServeFailed(_) => 4,

      AppError::Other(_) => 1,
    }
  }
}
