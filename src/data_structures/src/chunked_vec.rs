use std::{ cell::Cell, fmt::Debug };

pub struct ChunkedVec<T> where T: Default {
    inner: Vec<Option<T>>,
    occupied: Cell<usize>,
}

impl<T> ChunkedVec<T> where T: Default + Clone {
    pub fn new(size: usize) -> Self {
        Self {
            inner: vec![None; size],
            occupied: Cell::new(0),
        }
    }

    pub fn get_chunk(&self, size: usize) -> VecChunk<T> {
        let max_size = self.inner.capacity();

        let occupied = self.occupied.get();
        if occupied + size > max_size {
            panic!("ChunkedVec: Out of bounds");
        }

        let slice = &self.inner[occupied..occupied + size];

        self.occupied.set(occupied + size);

        VecChunk {
            inner: unsafe {
                std::slice::from_raw_parts_mut(slice.as_ptr() as *mut Option<T>, size)
            },
            size,
            written: 0,
        }
    }

    pub fn release(self) -> Vec<T> {
        self.inner
            .into_iter()
            .map(|opt| opt.expect("Every element should be written before releasing"))
            .collect()
    }
}

pub struct VecChunk<'a, T> {
    inner: &'a mut [Option<T>],
    size: usize,
    written: usize,
}

impl<T> VecChunk<'_, T> {
    pub fn push(&mut self, value: T) {
        if self.written >= self.size {
            panic!("VecChunk: Out of bounds");
        }

        self.inner[self.written] = Some(value);
        self.written += 1;
    }
}

impl Debug for ChunkedVec<u32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::ChunkedVec;
    use std::sync::{ LazyLock, Mutex };
    use threadpool::ThreadPool;
    use threadpool_scope::scope_with;

    const TOTAL_SIZE: usize = 5000000;
    const CHUNKS: usize = 10;
    const END_VEC: LazyLock<Vec<u32>> = LazyLock::new(|| (0..TOTAL_SIZE as u32).collect());

    #[test]
    fn test_all() {
        test_vec_mutex_multi_thread();
        println!("Test 1 done");
        test_vec_single_thread();
        println!("Test 2 done");
        test_chunked_vec_multi_thread();
        println!("Test 3 done");
        test_vec_multi_thread();
        println!("Test 4 done");
    }

    fn test_vec_multi_thread() {
        let start = std::time::Instant::now();

        let chunk_size = TOTAL_SIZE / CHUNKS;

        let final_vec = Mutex::new(Vec::with_capacity(TOTAL_SIZE));

        let threadpool = ThreadPool::new(CHUNKS);

        let now = std::time::Instant::now();

        scope_with(&threadpool, |scope| {
            let mut acc = 0;
            let final_vec_ref = &final_vec;

            for i in 0..CHUNKS {
                scope.execute(move || {
                    let mut vec = Vec::with_capacity(chunk_size);
                    for i in acc..chunk_size * (i + 1) {
                        vec.push(i as u32);
                    }
                    final_vec_ref.lock().unwrap().extend(vec);
                });
                acc += chunk_size;
            }
        });

        threadpool.join();

        let now_elapsed = now.elapsed();
        let total_elapsed = start.elapsed();

        println!("\n--- test_vec_multi_thread ---");
        println!("Loop runtime: {:?}", now_elapsed);
        println!("Total runtime: {:?}", total_elapsed);
    }

    fn test_vec_mutex_multi_thread() {
        let start = std::time::Instant::now();

        let chunk_size = TOTAL_SIZE / CHUNKS;

        let vec = Mutex::new(Vec::with_capacity(TOTAL_SIZE));

        let threadpool = ThreadPool::new(CHUNKS);

        let now = std::time::Instant::now();
        scope_with(&threadpool, |scope| {
            let mut acc = 0;
            let vec_ref = &vec;
            for i in 0..CHUNKS {
                scope.execute(move || {
                    let mut vec = vec_ref.lock().unwrap();
                    for i in acc..chunk_size * (i + 1) {
                        vec.push(i as u32);
                    }
                });
                acc += chunk_size;
            }
        });

        threadpool.join();

        let now_elapsed = now.elapsed();
        let total_elapsed = start.elapsed();

        println!("\n--- test_vec_mutex_multi_thread ---");
        println!("Loop runtime: {:?}", now_elapsed);
        println!("Total runtime: {:?}", total_elapsed);
    }

    fn test_vec_single_thread() {
        let start = std::time::Instant::now();
        let mut vec = Vec::with_capacity(TOTAL_SIZE);

        let now = std::time::Instant::now();
        for i in 0..TOTAL_SIZE {
            vec.push(i as u32);
        }

        let now_elapsed = now.elapsed();
        let total_elapsed = start.elapsed();

        println!("\n--- test_vec_single_thread ---");
        println!("Loop runtime: {:?}", now_elapsed);
        println!("Total runtime: {:?}", total_elapsed);
    }

    fn test_chunked_vec_multi_thread() {
        let start = std::time::Instant::now();

        let chunk_size = TOTAL_SIZE / CHUNKS;

        let threadpool = ThreadPool::new(CHUNKS);

        let chunked_vec = ChunkedVec::new(TOTAL_SIZE);

        let now = std::time::Instant::now();
        scope_with(&threadpool, |scope| {
            let mut acc = 0;
            let chunked_vec_ref = &chunked_vec;
            for i in 0..CHUNKS {
                let mut chunk = chunked_vec_ref.get_chunk(chunk_size);
                scope.execute(move || {
                    for i in acc..chunk_size * (i + 1) {
                        chunk.push(i as u32);
                    }
                });
                acc += chunk_size;
            }
        });

        threadpool.join();

        let now_elapsed = now.elapsed();
        let total_elapsed = start.elapsed();

        println!("\n--- test_chunked_vec_multi_thread ---");
        println!("Loop runtime: {:?}", now_elapsed);
        println!("Total runtime: {:?}", total_elapsed);
    }
}
