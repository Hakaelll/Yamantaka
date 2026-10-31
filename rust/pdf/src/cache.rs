use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use yamantaka_domain::RenderKey;

/// Delivery generations share pixels only within the same native document epoch.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct RasterKey {
    epoch: u64,
    content: RenderKey,
}
impl RasterKey {
    pub fn new(key: &RenderKey, epoch: u64) -> Self {
        let mut content = key.clone();
        content.generation = 0;
        Self { epoch, content }
    }
    pub fn document_is(&self, id: &str) -> bool {
        self.content.document_id == id
    }
}

/// CPU cost only. Presentation must separately account GPU copies and live consumers.
pub(crate) struct Cache<K, V> {
    entries: HashMap<K, (V, usize)>,
    order: VecDeque<K>,
    bytes: usize,
    limit: usize,
}
impl<K: Eq + Hash + Clone, V: Clone> Cache<K, V> {
    pub fn new(limit: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            bytes: 0,
            limit,
        }
    }
    pub fn get(&mut self, key: &K) -> Option<V> {
        let value = self.entries.get(key)?.0.clone();
        self.order.retain(|item| item != key);
        self.order.push_back(key.clone());
        Some(value)
    }
    pub fn insert(&mut self, key: K, value: V, bytes: usize) {
        if bytes > self.limit {
            return;
        }
        if let Some((_, old)) = self.entries.remove(&key) {
            self.bytes -= old;
        }
        self.order.retain(|item| item != &key);
        while self.bytes + bytes > self.limit {
            let Some(old) = self.order.pop_front() else {
                break;
            };
            if let Some((_, cost)) = self.entries.remove(&old) {
                self.bytes -= cost;
            }
        }
        self.entries.insert(key.clone(), (value, bytes));
        self.order.push_back(key);
        self.bytes += bytes;
    }
    pub fn retain(&mut self, keep: impl Fn(&K) -> bool) {
        self.entries.retain(|key, (_, cost)| {
            if keep(key) {
                true
            } else {
                self.bytes -= *cost;
                false
            }
        });
        self.order.retain(|key| self.entries.contains_key(key));
    }
    pub fn evict_oldest(&mut self) -> bool {
        let Some(key) = self.order.pop_front() else {
            return false;
        };
        if let Some((_, bytes)) = self.entries.remove(&key) {
            self.bytes -= bytes;
        }
        true
    }
}
