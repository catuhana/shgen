#[derive(Debug, thiserror::Error)]
pub enum KeepAwakeError {
    #[error("Platform error: {0}")]
    PlatformError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Keep-awake is not supported on this platform")]
    Unsupported,
}

pub type Result<T, E = KeepAwakeError> = std::result::Result<T, E>;

trait PlatformKeepAwakeTrait: Sized {
    type Error: std::error::Error + Send + Sync + 'static;

    fn new(reason: impl AsRef<str>) -> Result<Self, Self::Error>;

    fn prevent_sleep(&mut self) -> Result<(), Self::Error>;
    fn allow_sleep(&mut self) -> Result<(), Self::Error>;
}

#[cfg(target_os = "windows")]
#[path = "platform/windows.rs"]
mod platform;

#[cfg(not(target_os = "windows"))]
#[path = "platform/unsupported.rs"]
mod platform;

pub struct KeepAwake {
    inner: platform::PlatformKeepAwake,
}

impl KeepAwake {
    pub fn new(reason: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            inner: platform::PlatformKeepAwake::new(reason)
                .map_err(|e| KeepAwakeError::PlatformError(Box::new(e)))?,
        })
    }

    pub fn prevent_sleep(&mut self) -> Result<()> {
        self.inner
            .prevent_sleep()
            .map_err(|error| KeepAwakeError::PlatformError(Box::new(error)))
    }

    pub fn allow_sleep(&mut self) -> Result<()> {
        self.inner
            .allow_sleep()
            .map_err(|error| KeepAwakeError::PlatformError(Box::new(error)))
    }
}
