use crate::PlatformKeepAwakeTrait;

#[derive(Debug, thiserror::Error)]
#[error("Keep-awake is not supported on this platform")]
pub struct UnsupportedError;

pub struct PlatformKeepAwake;

impl PlatformKeepAwakeTrait for PlatformKeepAwake {
    type Error = UnsupportedError;

    fn new(_reason: impl AsRef<str>) -> Result<Self, Self::Error> {
        Err(UnsupportedError)
    }

    fn prevent_sleep(&mut self) -> Result<(), Self::Error> {
        Err(UnsupportedError)
    }

    fn allow_sleep(&mut self) -> Result<(), Self::Error> {
        Err(UnsupportedError)
    }
}
