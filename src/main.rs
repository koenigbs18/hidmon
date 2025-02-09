use hidmon::{HidMonitor, HidType};

fn main() {
    // Create new HidMonitor with no hooks enabled
    let mut hid_monitor = HidMonitor::default();

    // Enable mouse monitoring
    hid_monitor
        .enable(HidType::Mouse, |n_code, w_param, l_param| {
            println!("HidMonitor mouse callback with the following args:");
            println!("\tn_code: {n_code}, w_param: {w_param:?}, l_param: {l_param:?}");
        })
        .expect("Failed to enable mouse monitoring");

    // Use HidMonitor's convience function for handling WinApi messages
    HidMonitor::message_loop();
}
