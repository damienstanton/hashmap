use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;

// TODO: convert lib tests to doc tests
// TODO: entry API, 1:35 in video

const INITIAL_SIZE: usize = 1;

// a linked hashmap
pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    num_items: usize,
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            num_items: 0,
        }
    }

    fn bucket_idx<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>, // Note: read more about the borrow trait.
        Q: Hash + Eq + ?Sized,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % self.buckets.len() as u64) as usize
    }

    /// Inserts the given key and value
    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        if self.buckets.is_empty() || self.num_items * 3 > self.buckets.len() / 4 {
            self.resize();
        }

        let bucket = self.bucket_idx(&key);
        let bucket = &mut self.buckets[bucket];
        self.num_items += 1;
        for &mut (ref e_key, ref mut e_val) in bucket.iter_mut() {
            if e_key == &key {
                return Some(mem::replace(e_val, val));
            }
        }

        bucket.push((key, val));
        None
    }

    /// Returns the size of the hashmap
    pub fn len(&self) -> usize {
        self.num_items
    }

    /// Returns true if the hashmap is empty, otherwise false
    pub fn is_empty(&self) -> bool {
        match self.num_items {
            0 => true,
            _ => false,
        }
    }

    /// Deletes the value of a given key
    pub fn delete<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let target = self.bucket_idx(key);
        let target = &mut self.buckets[target];
        let i = target
            .iter()
            .position(|&(ref e_key, _)| e_key.borrow() == key)?;
        Some(target.swap_remove(i).1)
    }

    /// Resizes the hashmap
    pub fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_SIZE,
            n => n * 2,
        };

        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Vec::new()));
        for (key, val) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let bucket: usize = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[bucket].push((key, val))
        }

        mem::replace(&mut self.buckets, new_buckets);
    }

    /// Returns the value of a given key
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.buckets[self.bucket_idx(key)]
            .iter()
            .find(|&(ref e_key, _)| e_key.borrow() == key)
            .map(|&(_, ref v)| v)
    }

    /// Returns true if the hashmap contains the given key, otherwise false
    pub fn contains_key(&self, key: &K) -> bool {
        self.buckets[self.bucket_idx(key)]
            .iter()
            .any(|&(ref e_key, _)| e_key.borrow() == key)
    }
}

// Note: check the implied lifetime feature in 2018 edition, might make this type signature a bit less cluttered.
pub struct Iter<'a, K: 'a, V: 'a> {
    map: &'a HashMap<K, V>,
    bucket: usize,
    i: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => match bucket.get(self.i) {
                    Some(&(ref k, ref v)) => {
                        self.i += 1;
                        break Some((k, v));
                    }
                    None => {
                        self.bucket += 1;
                        self.i = 0;
                        continue;
                    }
                },
                None => break None,
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map: self,
            bucket: 0,
            i: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get_delete() {
        let mut map = HashMap::new();
        assert_eq!(map.is_empty(), true);
        map.insert("A", 1);
        assert_eq!(map.is_empty(), false);
        assert_eq!(map.contains_key(&"A"), true);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&"A"), Some(&1));
        assert_eq!(map.delete(&"A"), Some(1));
        assert_eq!(map.get(&"A"), None);
    }

    #[test]
    fn iteration() {
        let mut map = HashMap::new();
        map.insert("A", 1);
        map.insert("B", 2);
        map.insert("C", 3);
        map.insert("D", 4);
        assert_eq!(map.into_iter().count(), 4);
        for (&k, &v) in map.into_iter() {
            match k {
                "A" => assert_eq!(v, 1),
                "B" => assert_eq!(v, 2),
                "C" => assert_eq!(v, 3),
                "D" => assert_eq!(v, 4),
                &_ => (),
            };
        }
    }
}
