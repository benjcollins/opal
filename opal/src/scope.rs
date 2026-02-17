use std::{collections::HashMap, hash::Hash};

pub struct Scope<K, V> {
    map: HashMap<K, V>,
    aliases: Vec<(K, Option<V>)>,
    blocks: Vec<usize>,
}

impl<K: Hash + Eq, V> Default for Scope<K, V> {
    fn default() -> Self {
        Scope::new()
    }
}

impl<K: Hash + Eq, V> Scope<K, V> {
    pub fn new() -> Self {
        Scope {
            map: HashMap::new(),
            aliases: Vec::new(),
            blocks: Vec::new(),
        }
    }
}

impl<K: Hash + Eq + Clone, V> Scope<K, V> {
    pub fn enter_block(&mut self) {
        self.blocks.push(self.aliases.len());
    }
    pub fn insert(&mut self, key: K, value: V) {
        let alias = self.map.insert(key.clone(), value);
        self.aliases.push((key, alias));
    }
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    pub fn exit_block(&mut self) {
        let block = self.blocks.pop().unwrap();
        for (key, value) in self.aliases.drain(block..).rev() {
            match value {
                Some(value) => self.map.insert(key, value),
                None => self.map.remove(&key),
            };
        }
    }
}
