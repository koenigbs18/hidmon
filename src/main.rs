use std::sync::{Arc, Mutex};

use hidmon::{Call, HidCallback, HidMonitor, HidType};
use windows::Win32::Foundation::{LPARAM, WPARAM};

#[derive(Default)]
struct MyType1 {
    call_counter: isize,
}

impl Call for MyType1 {
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

impl Call for MyType2 {
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
        .enable(HidType::Mouse, HidCallback(my_type_1))
        .expect("Error enabling mouse monitoring for MyType1");

    hid_monitor
        .enable(HidType::Mouse, HidCallback(my_type_2))
        .expect("Error enabling mouse monitoring for MyType2");

    // Convience function for handling WinApi messages
    hidmon::message_loop();
}
