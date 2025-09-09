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
    core::PWSTR,
};

pub struct KeepAwake {
    handle: HANDLE,
    sleep_active: bool,
}

impl KeepAwake {
    pub fn new(reason: &str) -> Self {
        unsafe {
            let mut reason_wide: Vec<u16> =
                reason.encode_utf16().chain(std::iter::once(0)).collect();
            let context = REASON_CONTEXT {
                Flags: POWER_REQUEST_CONTEXT_SIMPLE_STRING,
                Version: POWER_REQUEST_CONTEXT_VERSION,
                Reason: REASON_CONTEXT_0 {
                    SimpleReasonString: PWSTR(reason_wide.as_mut_ptr()),
                },
            };
            let handle =
                PowerCreateRequest(&raw const context).expect("failed to create power request");

            Self {
                handle,
                sleep_active: false,
            }
        }
    }

    pub fn prevent_sleep(&mut self) {
        if !self.sleep_active {
            unsafe {
                PowerSetRequest(self.handle, PowerRequestExecutionRequired)
                    .expect("failed to set power request");
            }
            self.sleep_active = true;
        }
    }

    pub fn allow_sleep(&mut self) {
        if self.sleep_active {
            unsafe {
                PowerClearRequest(self.handle, PowerRequestExecutionRequired)
                    .expect("failed to clear power request");
            }
            self.sleep_active = false;
        }
    }
}

impl Drop for KeepAwake {
    fn drop(&mut self) {
        if self.sleep_active {
            self.allow_sleep();
        }

        unsafe {
            CloseHandle(self.handle).expect("failed to close power request handle");
        }
    }
}
