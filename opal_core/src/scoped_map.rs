use std::{collections::HashMap, hash::Hash};

pub struct ScopedMap<K, V> {
    map: HashMap<K, V>,
    layer_entries: Vec<(K, Option<V>)>,
    layer_starts: Vec<usize>,
}

impl<K: Hash + Eq + Clone, V> ScopedMap<K, V> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            layer_entries: Vec::new(),
            layer_starts: Vec::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let prev = self.map.insert(key.clone(), value);
        self.layer_entries.push((key, prev));
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    pub fn push_layer(&mut self) {
        self.layer_starts.push(self.layer_entries.len());
    }
    pub fn pop_layer(&mut self) {
        let start = self.layer_starts.pop().expect("no layers in stack");
        for (key, value) in self.layer_entries.drain(start..) {
            match value {
                Some(value) => self.map.insert(key, value),
                None => self.map.remove(&key),
            };
        }
    }
}

impl<K: Hash + Eq + Clone, V> Default for ScopedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::ScopedMap;

    #[test]
    fn test_insert_and_get() {
        let mut map = ScopedMap::new();
        map.insert("x", 42);
        assert_eq!(map.get(&"x"), Some(&42));
    }

    #[test]
    fn test_get_missing_key() {
        let map: ScopedMap<&str, i32> = ScopedMap::new();
        assert_eq!(map.get(&"x"), None);
    }

    #[test]
    fn test_insert_overwrites() {
        let mut map = ScopedMap::new();
        map.insert("x", 42);
        map.insert("x", 100);
        assert_eq!(map.get(&"x"), Some(&100));
    }

    #[test]
    fn test_single_layer_push_pop() {
        let mut map = ScopedMap::new();
        map.insert("x", 42);
        map.push_layer();
        map.insert("x", 100);
        assert_eq!(map.get(&"x"), Some(&100));
        map.pop_layer();
        assert_eq!(map.get(&"x"), Some(&42));
    }

    #[test]
    fn test_layer_removes_key() {
        let mut map = ScopedMap::new();
        map.insert("x", 42);
        map.push_layer();
        map.insert("x", 100);
        map.pop_layer();
        assert_eq!(map.get(&"x"), Some(&42));
    }

    #[test]
    fn test_new_key_in_layer_removed_on_pop() {
        let mut map = ScopedMap::new();
        map.push_layer();
        map.insert("x", 42);
        assert_eq!(map.get(&"x"), Some(&42));
        map.pop_layer();
        assert_eq!(map.get(&"x"), None);
    }
}
