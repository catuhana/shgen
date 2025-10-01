use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::{
            Power::{
                PowerClearRequest, PowerCreateRequest, PowerRequestExecutionRequired,
                PowerSetRequest,
            },
            SystemServices::POWER_REQUEST_CONTEXT_VERSION,
            Threading::{POWER_REQUEST_CONTEXT_SIMPLE_STRING, REASON_CONTEXT, REASON_CONTEXT_0},
        },
    },
    core::{Error as WindowsError, PWSTR},
};

use crate::PlatformKeepAwakeTrait;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, thiserror::Error)]
pub enum PlatformKeepAwakeError {
    #[error("Failed to create power request: {0}")]
    CreateRequest(WindowsError),
    #[error("Failed to set power request: {0}")]
    SetRequest(WindowsError),
    #[error("Failed to clear power request: {0}")]
    ClearRequest(WindowsError),
}

pub struct PlatformKeepAwake {
    handle: HANDLE,
    is_active: bool,
}

impl PlatformKeepAwakeTrait for PlatformKeepAwake {
    type Error = PlatformKeepAwakeError;

    fn new(reason: impl AsRef<str>) -> Result<Self, Self::Error> {
        let mut reason_wide: Vec<u16> = reason
            .as_ref()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let context = REASON_CONTEXT {
            Flags: POWER_REQUEST_CONTEXT_SIMPLE_STRING,
            Version: POWER_REQUEST_CONTEXT_VERSION,
            Reason: REASON_CONTEXT_0 {
                SimpleReasonString: PWSTR(reason_wide.as_mut_ptr()),
            },
        };

        let handle = unsafe {
            PowerCreateRequest(&raw const context).map_err(PlatformKeepAwakeError::CreateRequest)?
        };

        Ok(Self {
            handle,
            is_active: false,
        })
    }

    fn prevent_sleep(&mut self) -> Result<(), Self::Error> {
        if !self.is_active {
            unsafe { PowerSetRequest(self.handle, PowerRequestExecutionRequired) }
                .map_err(PlatformKeepAwakeError::SetRequest)?;

            self.is_active = true;
        }

        Ok(())
    }

    fn allow_sleep(&mut self) -> Result<(), Self::Error> {
        if self.is_active {
            unsafe { PowerClearRequest(self.handle, PowerRequestExecutionRequired) }
                .map_err(PlatformKeepAwakeError::ClearRequest)?;

            self.is_active = false;
        }

        Ok(())
    }
}

impl Drop for PlatformKeepAwake {
    fn drop(&mut self) {
        let _ = self.allow_sleep();

        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
