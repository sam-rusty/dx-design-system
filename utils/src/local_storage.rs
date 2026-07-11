use serde::Serialize;
use serde::de::DeserializeOwned;
use web_sys::Storage;

pub struct LocalStorage(Storage);

impl LocalStorage {
    pub fn new() -> Self {
        Self(
            web_sys::window()
                .unwrap()
                .local_storage()
                .ok()
                .unwrap()
                .unwrap(),
        )
    }
    pub fn set<T: Serialize>(&self, key: &str, value: &T) {
        if let Ok(s) = serde_json::to_string(value) {
            let _ = self.0.set_item(key, &s);
        }
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let json = self.0.get_item(key).ok()??;
        serde_json::from_str(&json).ok()
    }

    pub fn remove(&self, key: &str) {
        let _ = self.0.remove_item(key);
    }
}

impl Default for LocalStorage {
    fn default() -> Self {
        Self::new()
    }
}
