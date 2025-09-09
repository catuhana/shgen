pub struct KeepAwake {
    inner: PlatformKeepAwake,
}

impl KeepAwake {
    #[must_use]
    pub fn new(reason: &str) -> Self {
        Self {
            inner: PlatformKeepAwake::new(reason),
        }
    }

    pub fn prevent_sleep(&mut self) {
        self.inner.prevent_sleep();
    }

    pub fn allow_sleep(&mut self) {
        self.inner.allow_sleep();
    }
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::KeepAwake as PlatformKeepAwake;

#[cfg(not(target_os = "windows"))]
mod noop;
#[cfg(not(target_os = "windows"))]
use noop::KeepAwake as PlatformKeepAwake;
