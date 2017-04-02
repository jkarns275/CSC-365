use std::hash::{ Hash, Hasher };
use std::collections::hash_map::DefaultHasher;
use std::collections::LinkedList;
use std::io::{ Seek, SeekFrom, Read, Write };
use std::io;

#[derive(PartialEq, Eq)]
struct Entry<K, V> where K: Hash + Eq, V: Eq {
    key: K,
    value: V,
    hash: u64
}
impl<K, V> Entry<K, V> where K: Hash + Eq, V: Eq {
    fn new(key: K, value: V, hash: u64) -> Entry<K, V> {
        Entry {
            key: key,
            value: value,
            hash: hash
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct TwitHash<K, V> where K: Hash + Eq, V: Eq {
    entries: Vec<LinkedList<Box<Entry<K, V>>>>,
    len: usize,
    pub count: usize,
}

impl<K, V> TwitHash<K, V> where K: Hash + Eq, V: Eq {
    pub fn new() -> TwitHash<K, V> {
        let mut t = TwitHash {
            entries: Vec::with_capacity(16),
            count: 0,
            len: 16
        };
        for _ in 0..16 { t.entries.push(LinkedList::new()); }
        t
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    fn insert_entry(&mut self, e: Box<Entry<K, V>>) {
        self.entries[e.hash as usize & (self.len - 1)].push_front(e);
    }

    /// Will dynamically resize the size of the hash once alpha is > .75 (alpha being
    /// the total number of elements divided by the number of slots).
    pub fn insert(&mut self, key: K, value: V) {
        if let Some(v) = self.get_mut(&key) {
            *v = value;
            return;
        }
        let hash = self.hash(&key);
        self.insert_entry(Box::new(Entry::new(key, value, hash)));
        self.count += 1;

        if (self.count as f32) / (self.len as f32) > 0.75 {
            self.len *= 2;
            let mut elements = Vec::with_capacity(self.len);
            while let Some(element) = self.entries.pop() {
                elements.append(&mut element.into_iter().collect());
            }
            for _ in 0 .. self.len {
                self.entries.push(LinkedList::new());
            }
            while let Some(element) = elements.pop() {
                self.insert_entry(element);
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.hash(&key);
        let ind = (hash as usize) & (self.len - 1);
        if let Some(b) = self.entries[ind].iter().find(|ref x| x.key == *key && x.hash == hash) {
            Some(&b.value)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let hash = self.hash(&key);
        let ind = (hash as usize) & (self.len - 1);
        if let Some(b) = self.entries[ind].iter_mut().find(|ref x| x.key == *key && x.hash == hash) {
            Some(&mut b.value)
        } else {
            None
        }
    }

    fn hash(&self, k: &K) -> u64 {
        let mut hasher = DefaultHasher::new();
        k.hash(&mut hasher);
        hasher.finish()
    }

    pub fn contains_key(&self, key: &K) -> bool {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let ind = (hash as usize) & (self.len - 1);
        if self.entries[ind].is_empty() {
            false
        } else {
            !self.entries[ind].iter().find(|ref x| x.key == *key && x.hash == hash).is_none()
        }
    }

    pub fn keys(&self) -> Vec<&K> {
        let mut ret = Vec::new();
        for list in self.entries.iter() {
            for entry in list.iter() {
                if !ret.contains(&&entry.key) {
                    ret.push(&entry.key);
                }
            }
        }
        ret
    }

    pub fn values(&self) -> Vec<&V> {
        let mut ret = Vec::new();
        for list in self.entries.iter() {
            for entry in list.iter() {
                if !ret.contains(&&entry.value) {
                    ret.push(&entry.value);
                }
            }
        }
        ret
    }

    pub fn pairs(&self) -> Vec<(&K, &V)> {
        let mut ret = Vec::new();
        for list in self.entries.iter() {
            for entry in list.iter() {
                if !ret.contains(&(&entry.key, &entry.value)) {
                    ret.push((&entry.key, &entry.value));
                }
            }
        }
        ret
    }
}

use std::ops::Index;

/// Index operator
impl<K, V> Index<K> for TwitHash<K, V> where K: Eq + Hash, V: Eq {
    type Output = V;

    fn index(&self, key: K) -> &V {
        &self.get(&key).unwrap()
    }
}
