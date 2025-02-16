use windows::Win32::Foundation::{LPARAM, WPARAM};

// TODO: Generalize this to use HidEvent
pub trait Call {
    fn callback(&mut self, ncode: i32, wparam: WPARAM, lparam: LPARAM);
}
