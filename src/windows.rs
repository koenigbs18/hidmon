#![allow(non_snake_case)]
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
        UnhookWindowsHookEx, HHOOK, HOOKPROC, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WINDOWS_HOOK_ID,
        WM_QUIT,
    },
};

use crate::{
    globals::{GLOBAL_KEYBD_CALLBACKS, GLOBAL_MOUSE_CALLBACKS},
    HidType, Result,
};

pub unsafe extern "system" fn LowLevelKeyboardProc(
    ncode: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if ncode < 0 {
        return CallNextHookEx(None, ncode, wparam, lparam);
    }
    let callback_map = GLOBAL_KEYBD_CALLBACKS.lock().unwrap();
    for entry in &mut callback_map.values() {
        entry.0.lock().unwrap().callback(ncode, wparam, lparam);
    }
    drop(callback_map);
    CallNextHookEx(None, ncode, wparam, lparam)
}

pub unsafe extern "system" fn LowLevelMouseProc(
    ncode: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if ncode < 0 {
        return CallNextHookEx(None, ncode, wparam, lparam);
    }
    let callback_map = GLOBAL_MOUSE_CALLBACKS.lock().unwrap();
    for entry in &mut callback_map.values() {
        entry.0.lock().unwrap().callback(ncode, wparam, lparam);
    }
    drop(callback_map);
    CallNextHookEx(None, ncode, wparam, lparam)
}

pub fn hook(hid_type: HidType) -> Result<HHOOK> {
    unsafe {
        Ok(SetWindowsHookExW(
            hid_type.into(),
            hid_type.into(),
            None,
            0,
        )?)
    }
}

pub fn unhook(hook: HHOOK) -> Result<()> {
    unsafe { Ok(UnhookWindowsHookEx(hook)?) }
}

impl From<HidType> for HOOKPROC {
    fn from(value: HidType) -> Self {
        match value {
            HidType::Keyboard => Some(LowLevelKeyboardProc),
            HidType::Mouse => Some(LowLevelMouseProc),
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

/// Barebones WINAPI message handler
///
/// Exits if it receives a `WM_QUIT` message.
#[allow(dead_code)]
pub fn message_loop() {
    let mut msg = MSG::default();
    loop {
        let msg_ptr = std::ptr::from_mut::<MSG>(&mut msg);
        unsafe {
            if GetMessageW(msg_ptr, None, 0, 0).into() {
                let _ = TranslateMessage(msg_ptr);
                DispatchMessageW(msg_ptr);
            }
        }
        if msg.message == WM_QUIT {
            return;
        }
    }
}
