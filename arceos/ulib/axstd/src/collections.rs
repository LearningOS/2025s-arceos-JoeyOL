use alloc::vec::Vec;
use alloc::vec;
// Re-export alloc::collections as alloc_collections to avoid conflict
pub use alloc::collections::*;
use arceos_api::modules::axhal::misc::random;

fn rotl(x: u64, n: u32) -> u64 {
    x.rotate_left(n)
}

struct SipHasher {
    seed: u128,
}

impl SipHasher {
    fn new() -> Self {
        Self {
            seed: random(),
        }
    }

    fn siphash24(&self, key: &[u8]) -> u64 {
        let key0 = (self.seed >> 64) as u64;
        let key1 = self.seed as u64;
        let mut v0 = key0 ^ 0x736f6d6570736575;
        let mut v1 = key1 ^ 0x646f72616e646f6d;
        let mut v2 = key0 ^ 0x6c7967656e657261;
        let mut v3 = key1 ^ 0x7465646279746573;

        let mut m = key;
        let mut last_block = [0u8; 8];
        let len = key.len() as u64;

        while m.len() >= 8 {
            let block = &m[..8];
            m = &m[8..];
            let k = u64::from_le_bytes(block.try_into().unwrap());
            v3 ^= k;
            for _ in 0..2 {
                // SipRound
                v0 = v0.wrapping_add(v1); v1 = rotl(v1, 13); v1 ^= v0; v0 = rotl(v0, 32);
                v2 = v2.wrapping_add(v3); v3 = rotl(v3, 16); v3 ^= v2;
                v0 = v0.wrapping_add(v3); v3 = rotl(v3, 21); v3 ^= v0;
                v2 = v2.wrapping_add(v1); v1 = rotl(v1, 17); v1 ^= v2; v2 = rotl(v2, 32);
            }
            v0 ^= k;
        }

        last_block[..m.len()].copy_from_slice(m);
        last_block[7] = len as u8;
        let k = u64::from_le_bytes(last_block);
        v3 ^= k;
        for _ in 0..2 {
            // SipRound
            v0 = v0.wrapping_add(v1); v1 = rotl(v1, 13); v1 ^= v0; v0 = rotl(v0, 32);
            v2 = v2.wrapping_add(v3); v3 = rotl(v3, 16); v3 ^= v2;
            v0 = v0.wrapping_add(v3); v3 = rotl(v3, 21); v3 ^= v0;
            v2 = v2.wrapping_add(v1); v1 = rotl(v1, 17); v1 ^= v2; v2 = rotl(v2, 32);
        }
        v0 ^= k;
            // 最终混合
        v2 ^= 0xff;
        for _ in 0..4 {
            v0 = v0.wrapping_add(v1); v1 = rotl(v1, 13); v1 ^= v0; v0 = rotl(v0, 32);
            v2 = v2.wrapping_add(v3); v3 = rotl(v3, 16); v3 ^= v2;
            v0 = v0.wrapping_add(v3); v3 = rotl(v3, 21); v3 ^= v0;
            v2 = v2.wrapping_add(v1); v1 = rotl(v1, 17); v1 ^= v2; v2 = rotl(v2, 32);
        }

        v0 ^ v1 ^ v2 ^ v3
    }
}


pub struct HashMap<K, V> {
    buckets: Vec<Option<(K, V)>>,
    len: usize,
    hasher: SipHasher,
}

impl <K: Clone + PartialEq , V: Clone> HashMap<K, V> {
    pub fn new() -> Self {
        let buckets: Vec<Option<(K, V)>> = vec![None; 16];
        Self {
            buckets: buckets, 
            len: 0, 
            hasher: SipHasher::new() 
        }
    }

    pub fn convert_key_to_u8(&self, key: &K) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                key as *const K as *const u8,
                core::mem::size_of::<K>(),
            )
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut index = self.hasher.siphash24(self.convert_key_to_u8(&key)) as usize % self.buckets.len();
        while let Some((k, _)) = &self.buckets[index] {
            if *k == key {
                self.buckets[index] = Some((key, value));
                return;
            }
            index = (index + 1) % self.buckets.len();
        }
        self.buckets[index] = Some((key, value));
        self.len += 1;
        if self.len > self.buckets.len() / 2 {
            self.resize();
        }
    }

    pub fn resize(&mut self) {
        let new_size = self.buckets.len() << 1;
        let mut new_buckets: Vec<Option<(K, V)>> = vec![None; new_size];
        self.buckets.iter().filter_map(|entry| entry.clone()).for_each(|(k, v)| {
            let mut index = self.hasher.siphash24(self.convert_key_to_u8(&k)) as usize % new_size;
            while new_buckets[index].is_some() {
                index = (index + 1) % new_size;
            }
            new_buckets[index] = Some((k, v));
        });
        self.buckets = new_buckets;
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut index = self.hasher.siphash24(self.convert_key_to_u8(key)) as usize % self.buckets.len();
        while let Some((k, v)) = &self.buckets[index] {
            if k == key {
                return Some(v);
            }
            index = (index + 1) % self.buckets.len();
        }
        None
    }
    
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut index = self.hasher.siphash24(self.convert_key_to_u8(key)) as usize % self.buckets.len();
        let st_index = index;
        while let Some((k, v)) = &self.buckets[index] {
            if k == key {
                break;
            }
            if index == st_index {
                return None;
            }
            index = (index + 1) % self.buckets.len();
        }
        let ret = self.buckets[index].take().unwrap().1;
        self.buckets[index] = None;
        self.len -= 1;
        Some(ret)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.buckets.clear();
        self.len = 0;
        self.buckets = vec![None; 16];
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.buckets.iter().filter_map(|entry| entry.as_ref()).map(|(k, v)| (k, v))
    }
}