use std::{ collections::HashMap, hash::{ BuildHasher, Hash }, sync::Mutex };

pub struct ChunkedHashMap<K, V, S> where S: BuildHasher + Default, K: Eq + Hash {
    inner: HashMap<K, V, S>,
    occupied: Mutex<usize>,
}

impl<K, V, S> ChunkedHashMap<K, V, S> where S: BuildHasher + Default, K: Eq + Hash {
    pub fn new(size: usize, hasher: S) -> Self {
        Self {
            inner: HashMap::with_capacity_and_hasher(size, hasher),
            occupied: Mutex::new(0),
        }
    }

    pub fn get_chunk(&self, size: usize) -> HashMapChunk<K, V, S> {
        todo!()
    }
}

pub struct HashMapChunk<'a, K, V, S> where S: BuildHasher + Default, K: Eq + Hash {
    inner: &'a mut HashMap<K, V, S>,
    size: usize,
    written: usize,
}
