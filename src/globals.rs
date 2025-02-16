use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use rand::Rng;

use crate::{HidCallback, HidType};

type Key = u64;
type Value = HidCallback;
type CallbackMap = HashMap<Key, Value>;

pub static GLOBAL_KEYBD_CALLBACKS: LazyLock<Mutex<CallbackMap>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static GLOBAL_MOUSE_CALLBACKS: LazyLock<Mutex<CallbackMap>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct GlobalCallback {
    hid_type: HidType,
    key: Key,
}

impl GlobalCallback {
    pub fn new(hid_type: HidType, callback: HidCallback) -> Self {
        let mut callback_map = match hid_type {
            HidType::Keyboard => GLOBAL_KEYBD_CALLBACKS.lock().unwrap(),
            HidType::Mouse => GLOBAL_MOUSE_CALLBACKS.lock().unwrap(),
        };
        let mut rng = rand::rng();
        loop {
            let key = rng.random::<Key>();
            if let std::collections::hash_map::Entry::Vacant(entry) = callback_map.entry(key) {
                entry.insert(callback);
                break Self { hid_type, key };
            }
        }
    }
}

impl Drop for GlobalCallback {
    fn drop(&mut self) {
        let mut callback_map = match self.hid_type {
            HidType::Keyboard => GLOBAL_KEYBD_CALLBACKS.lock().unwrap(),
            HidType::Mouse => GLOBAL_MOUSE_CALLBACKS.lock().unwrap(),
        };
        if callback_map.contains_key(&self.key) {
            callback_map
                .remove(&self.key)
                .expect("Callback map must be checked for a valid key before attempting to remove");
        }
    }
}
