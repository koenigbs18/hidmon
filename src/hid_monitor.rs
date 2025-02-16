use std::sync::{Arc, Mutex};

use crate::globals::GlobalCallback;
use crate::traits::Call;
use crate::windows;
use crate::Result;
use ::windows::Win32::UI::WindowsAndMessaging::HHOOK;

#[derive(Clone, Copy)]
pub enum HidType {
    Keyboard,
    Mouse,
}

#[derive(Clone)]
pub struct HidCallback(pub Arc<Mutex<dyn Call + Send>>);

/// Wrapper type which couples a raw hook and its associated global callbacks
#[derive(Default)]
struct Hook {
    raw: HHOOK,
    callbacks: Vec<GlobalCallback>,
}

impl Hook {
    /// Hook must be **valid** before calling
    fn register_global_callback(&mut self, hid_type: HidType, hid_callback: HidCallback) {
        assert!(!self.raw.is_invalid());
        self.callbacks
            .push(GlobalCallback::new(hid_type, hid_callback));
    }
    /// Hook must be **invalid** before calling
    fn hook(&mut self, hid_type: HidType, hid_callbacks: &Vec<HidCallback>) -> Result<()> {
        assert!(self.raw.is_invalid());
        self.raw = windows::hook(hid_type)?;
        for hid_callback in hid_callbacks {
            // Enable "local" HID callbacks by inserting them into the global callback registry
            self.register_global_callback(hid_type, hid_callback.clone());
        }
        Ok(())
    }
    /// Hook must be **valid** before calling
    fn unhook(&mut self) -> Result<()> {
        assert!(!self.raw.is_invalid());
        windows::unhook(self.raw)?;
        self.raw = HHOOK::default();
        // Clear global callbacks, effectively disabling them
        self.callbacks.clear();
        Ok(())
    }
    fn valid(&self) -> bool {
        !self.raw.is_invalid()
    }
    fn clear_global_callbacks(&mut self) {
        self.callbacks.clear();
    }
}

/// Callback-based HID event monitoring
pub struct HidMonitor {
    keybd_hook: Hook,
    mouse_hook: Hook,
    keybd_callbacks: Vec<HidCallback>,
    mouse_callbacks: Vec<HidCallback>,
}

impl Default for HidMonitor {
    /// Creates a new `HidMonitor` with all callbacks disabled
    fn default() -> Self {
        Self {
            keybd_hook: Hook::default(),
            mouse_hook: Hook::default(),
            keybd_callbacks: Vec::default(),
            mouse_callbacks: Vec::default(),
        }
    }
}

impl HidMonitor {
    /// Enables HID callbacks
    ///
    /// ## ⚠️ Warning
    ///
    /// * Windows targets
    ///     * You ***MUST*** have a [message loop](https://learn.microsoft.com/en-us/windows/win32/winmsg/using-messages-and-message-queues#creating-a-message-loop)
    ///       running on the same thread as the `HidMonitor` hooks enabled by this function, otherwise your system may become
    ///       unresponsive!  For maximum safety, ensure the message loop is running **before** enabling the HID monitor, or shortly after.
    ///       For applications which otherwise do not care about handling `WinApi` messages, [`HidMonitor::message_loop`] serves
    ///       as a convenience function for starting a simple message handler.
    ///     * Read more about the implications of this function on the `WinApi` documentation for
    ///       [`SetWindowsHookExA`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexa#remarks)
    ///
    /// ## Errors
    ///
    /// Windows: [`SetWindowsHookExA`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexa)
    /// returned an error
    pub fn enable(&mut self, hid_type: HidType) -> Result<&mut Self> {
        let (hook, callbacks) = match hid_type {
            HidType::Keyboard => (&mut self.keybd_hook, &self.keybd_callbacks),
            HidType::Mouse => (&mut self.mouse_hook, &self.mouse_callbacks),
        };
        if !hook.valid() {
            hook.hook(hid_type, callbacks)?;
        }
        Ok(self)
    }

    /// Disables HID callbacks
    ///
    /// ## Errors
    ///
    /// Windows: [`UnhookWindowsHookEx`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwindowshookex)
    /// returned an error
    pub fn disable(&mut self, hid_type: HidType) -> Result<&mut Self> {
        let hook = match hid_type {
            HidType::Keyboard => &mut self.keybd_hook,
            HidType::Mouse => &mut self.mouse_hook,
        };
        if hook.valid() {
            hook.unhook()?;
        }
        Ok(self)
    }

    /// Adds a new HID callback
    ///
    /// Any number of callbacks may be added for a given `HidType`, but callback order is non-deterministic.
    ///
    /// Callbacks can be enabled by calling [`HidMonitor::enable`]
    pub fn add_callback(&mut self, hid_type: HidType, hid_callback: HidCallback) -> &mut Self {
        let (hook, callbacks) = match hid_type {
            HidType::Keyboard => (&mut self.keybd_hook, &mut self.keybd_callbacks),
            HidType::Mouse => (&mut self.mouse_hook, &mut self.mouse_callbacks),
        };
        if hook.valid() {
            hook.register_global_callback(hid_type, hid_callback.clone());
        }
        callbacks.push(hid_callback);
        self
    }

    /// Removes all callbacks set by this `HidMonitor` instance
    pub fn clear_callbacks(&mut self, hid_type: HidType) {
        let (hook, callbacks) = match hid_type {
            HidType::Keyboard => (&mut self.keybd_hook, &mut self.keybd_callbacks),
            HidType::Mouse => (&mut self.mouse_hook, &mut self.mouse_callbacks),
        };
        hook.clear_global_callbacks();
        callbacks.clear();
    }
}

impl Drop for HidMonitor {
    fn drop(&mut self) {
        let _ = self.disable(HidType::Keyboard);
        let _ = self.disable(HidType::Mouse);
    }
}
