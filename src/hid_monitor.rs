use crate::Result;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, PeekMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
    HHOOK, MSG, PM_REMOVE, WM_QUIT,
};

#[allow(non_snake_case)]
mod inner {
    use super::{HidCallback, HidType, LPARAM, LRESULT, WPARAM};

    use std::sync::{LazyLock, Mutex};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, HOOKPROC, WH_KEYBOARD_LL, WH_MOUSE_LL, WINDOWS_HOOK_ID,
    };
    static GLOBAL_KEYBD_CALLBACK: LazyLock<Mutex<Option<HidCallback>>> =
        LazyLock::new(|| Mutex::new(None));
    static GLOBAL_MOUSE_CALLBACK: LazyLock<Mutex<Option<HidCallback>>> =
        LazyLock::new(|| Mutex::new(None));

    unsafe extern "system" fn KeyboardProc(nCode: i32, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let callback = *GLOBAL_KEYBD_CALLBACK.lock().unwrap();
        if let Some(callback) = callback {
            callback(nCode, wParam, lParam);
        } else {
            println!("HidMonitor: Keyboard callback was never set!");
        }
        CallNextHookEx(None, nCode, wParam, lParam)
    }

    unsafe extern "system" fn MouseProc(nCode: i32, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let callback = *GLOBAL_MOUSE_CALLBACK.lock().unwrap();
        if let Some(callback) = callback {
            callback(nCode, wParam, lParam);
        } else {
            println!("HidMonitor: Mouse callback was never set!");
        }
        CallNextHookEx(None, nCode, wParam, lParam)
    }

    impl From<HidType> for HOOKPROC {
        fn from(value: HidType) -> Self {
            match value {
                HidType::Keyboard => Some(KeyboardProc),
                HidType::Mouse => Some(MouseProc),
            }
        }
    }
    impl From<HidType> for WINDOWS_HOOK_ID {
        fn from(value: HidType) -> Self {
            match value {
                HidType::Keyboard => WH_KEYBOARD_LL,
                HidType::Mouse => WH_MOUSE_LL,
            }
        }
    }
    pub fn set_global_callback(r#type: HidType, callback: HidCallback) {
        match r#type {
            HidType::Keyboard => GLOBAL_KEYBD_CALLBACK.lock().unwrap().replace(callback),
            HidType::Mouse => GLOBAL_MOUSE_CALLBACK.lock().unwrap().replace(callback),
        };
    }
}

use inner::set_global_callback;

pub type HidCallback = fn(i32, WPARAM, LPARAM);

#[derive(Clone, Copy)]
pub enum HidType {
    Keyboard,
    Mouse,
}

pub struct HidMonitor {
    keybd_hook: HHOOK,
    mouse_hook: HHOOK,
}

impl Default for HidMonitor {
    /// Creates a new `HidMonitor` with all hooks disabled
    ///
    /// To start monitoring call [`HidMonitor::start`]
    fn default() -> Self {
        Self {
            keybd_hook: HHOOK::default(),
            mouse_hook: HHOOK::default(),
        }
    }
}

impl HidMonitor {
    fn hook(r#type: HidType, callback: HidCallback) -> Result<HHOOK> {
        let hook;
        unsafe {
            hook = SetWindowsHookExW(r#type.into(), r#type.into(), None, 0)?;
        }
        set_global_callback(r#type, callback);
        Ok(hook)
    }

    fn unhook(hook: HHOOK) -> Result<()> {
        unsafe { Ok(UnhookWindowsHookEx(hook)?) }
    }

    /// Barebones WINAPI message handler
    ///
    /// Exits if it receives a `WM_QUIT` message.
    pub fn message_loop() {
        let mut msg = MSG::default();
        loop {
            let msg_ptr = std::ptr::from_mut::<MSG>(&mut msg);
            if unsafe { PeekMessageW(msg_ptr, None, 0, 0, PM_REMOVE).0 } != 0 {
                if msg.message == WM_QUIT {
                    return;
                }
                let _ = unsafe { TranslateMessage(msg_ptr) };
                unsafe { DispatchMessageW(msg_ptr) };
            }
        }
    }

    /// Enables HID monitoring for a given `HidType`
    ///
    /// To stop monitoring call [`HidMonitor::disable`]
    ///
    /// ## ⚠️ Warning
    ///
    /// * Windows targets
    ///     * You ***MUST*** have a [message loop](https://learn.microsoft.com/en-us/windows/win32/winmsg/using-messages-and-message-queues#creating-a-message-loop)
    ///       running on the same thread as the `HidMonitor` hooks enabled by this function, otherwise your system may become
    ///       unresponsive!  For maximum safety, ensure the message loop is running **before** enabling any hooks, or shortly after.
    ///       For applications which otherwise do not care about handling `WinApi` messages, [`HidMonitor::message_loop`] serves
    ///       as a convenience function for starting a simple message handler.
    ///     * Only one unique `HidType` can be monitored per running process.  For example, attempting to start multiple
    ///       `HidMonitor`'s with `HidType::Mouse` will result in an error.
    ///     * Read more about the implications of this function on the `WinApi` documentation for
    ///       [`SetWindowsHookExA`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexa#remarks)
    ///
    ///
    ///
    /// ## Usage
    ///
    /// ```Rust
    /// use hidmon::{HidMonitor, HidType};
    ///
    /// fn main() {
    ///     // Create new HidMonitor with no hooks enabled
    ///     let mut hid_monitor = HidMonitor::default();
    ///
    ///     // Enable mouse monitoring
    ///     hid_monitor
    ///         .enable(HidType::Mouse, |n_code, w_param, l_param| {
    ///             println!("HidMonitor mouse callback with the following args:");
    ///             println!("\tn_code: {n_code}, w_param: {w_param:?}, l_param: {l_param:?}");
    ///         })
    ///         .expect("Failed to enable mouse monitoring");
    ///
    ///     // Use HidMonitor's convenience function for handling WinApi functions
    ///     HidMonitor::message_loop();
    /// }
    /// ```
    ///
    /// ## Errors
    ///
    /// Windows: Only one unique `HidType` can be monitored per running process.
    pub fn enable(&mut self, r#type: HidType, callback: HidCallback) -> Result<()> {
        let hook = Self::hook(r#type, callback)?;
        match r#type {
            HidType::Keyboard => self.keybd_hook = hook,
            HidType::Mouse => self.mouse_hook = hook,
        }
        Ok(())
    }

    /// Disables HID monitoring for a given `HidType`
    ///
    /// ## Errors
    ///
    /// Windows: [`UnhookWindowsHookEx`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwindowshookex)
    /// returned an error
    pub fn disable(&mut self, r#type: HidType) -> Result<()> {
        match r#type {
            HidType::Keyboard => {
                Self::unhook(self.keybd_hook)?;
                self.keybd_hook = HHOOK::default();
            }
            HidType::Mouse => {
                Self::unhook(self.mouse_hook)?;
                self.mouse_hook = HHOOK::default();
            }
        }
        Ok(())
    }
}

impl Drop for HidMonitor {
    fn drop(&mut self) {
        if !self.keybd_hook.is_invalid() {
            let _ = Self::unhook(self.keybd_hook);
        }
        if !self.mouse_hook.is_invalid() {
            let _ = Self::unhook(self.mouse_hook);
        }
    }
}
