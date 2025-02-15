use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::globals::GlobalCallback;
use crate::Result;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::HHOOK;

#[derive(Clone, Copy)]
pub enum HidType {
    Keyboard,
    Mouse,
}

pub enum HidEvent {
    Keyboard {
        time: SystemTime,
        code: i32,
        value: i32,
    },
    Mouse,
}

pub struct HidCallback(pub Arc<Mutex<dyn Call + Send>>);

struct Hook {
    hook: HHOOK,
    global_callback: GlobalCallback,
}

pub struct HidMonitor {
    keybd_hook: Option<Hook>,
    mouse_hook: Option<Hook>,
}

impl Default for HidMonitor {
    /// Creates a new `HidMonitor` with all hooks disabled
    ///
    /// To start monitoring call [`HidMonitor::enable`]
    fn default() -> Self {
        Self {
            keybd_hook: None,
            mouse_hook: None,
        }
    }
}

impl HidMonitor {
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
    ///     // Use HidMonitor's convenience function for handling WinApi messages
    ///     HidMonitor::message_loop();
    /// }
    /// ```
    ///
    /// ## Errors
    ///
    /// Windows: Only one unique `HidType` can be monitored per running process.
    pub fn enable(&mut self, hid_type: HidType, callback: HidCallback) -> Result<()> {
        match hid_type {
            HidType::Keyboard => &mut self.keybd_hook,
            HidType::Mouse => &mut self.mouse_hook,
        }
        .replace((
            Self::hook(hid_type)?,
            GlobalCallback::new(hid_type, callback),
        ));
        Ok(())
    }

    /// Disables HID monitoring for a given `HidType`
    ///
    /// ## Errors
    ///
    /// Windows: [`UnhookWindowsHookEx`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwindowshookex)
    /// returned an error
    pub fn disable(&mut self, hid_type: HidType) -> Result<()> {
        if let Some(hook) = match hid_type {
            HidType::Keyboard => &mut self.keybd_hook,
            HidType::Mouse => &mut self.mouse_hook,
        } {
            Self::unhook(hook.0)?;
        }
        hook.Ok(())
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
