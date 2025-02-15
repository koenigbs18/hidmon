use std::sync::{Arc, Mutex};

use hidmon::{HidCallback, HidMonitor, HidType};
use windows::Win32::Foundation::{LPARAM, WPARAM};

#[derive(Default)]
struct MyType1 {
    call_counter: isize,
}

impl HidCallback for MyType1 {
    fn callback(&mut self, n_code: i32, w_param: WPARAM, l_param: LPARAM) {
        println!("[MyType1, call #{}]:", self.call_counter);
        println!("\tn_code: {n_code}, w_param: {w_param:?}, l_param: {l_param:?}");
        self.call_counter += 1;
    }
}

unsafe impl Send for MyType1 {}

#[derive(Default)]
struct MyType2 {
    call_counter: isize,
}

unsafe impl Send for MyType2 {}

impl HidCallback for MyType2 {
    fn callback(&mut self, n_code: i32, w_param: WPARAM, l_param: LPARAM) {
        println!("[MyType2, call #{}]:", self.call_counter);
        println!("\tn_code: {n_code}, w_param: {w_param:?}, l_param: {l_param:?}");
        self.call_counter -= 1;
    }
}

fn main() {
    // Create new HidMonitor with no hooks enabled
    let mut hid_monitor = HidMonitor::default();
    let my_type_1 = Arc::new(Mutex::new(MyType1::default()));
    let my_type_2 = Arc::new(Mutex::new(MyType2::default()));

    hid_monitor
        .enable(&HidType::Mouse, my_type_1)
        .expect("Error enabling mouse monitoring");

    hid_monitor
        .enable(&HidType::Mouse, my_type_2)
        .expect("Error enabling mouse monitoring");

    // Use HidMonitor's convience function for handling WinApi messages
    HidMonitor::message_loop();
}
