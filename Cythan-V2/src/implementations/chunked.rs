const CHUNK_SIZE: usize = 32;
const CHUNK_SIZE_INDEXED: usize = CHUNK_SIZE - 1;

const VEC: [usize; CHUNK_SIZE] = get_vec();

const fn get_vec() -> [usize; CHUNK_SIZE] {
    let mut vec: [usize; CHUNK_SIZE] = [0; CHUNK_SIZE];
    vec[0] = 2;
    vec
}

/// This Cythan implementation uses memory chunking to increase memory efficiency on very large codes!
/// This implementation is slighly slower than the base one but get far more efficient on big codes
///
/// ```rust
/// use cythan::{Cythan,ChunkedCythan};
/// // This function create a Cythan Machine with a step of 2 and a base value of 0
/// let machine = ChunkedCythan::new(vec![12,23,45,20,0]);
/// ```
pub struct ChunkedCythan {
    pub cases: Vec<[usize; CHUNK_SIZE]>,
}

impl std::fmt::Display for ChunkedCythan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cythan{:?}", self.cases)
    }
}

impl ChunkedCythan {
    /// Create a chunked Cythan Machine with a step of 2 and a base value of 0
    pub fn new(cases: Vec<usize>) -> Self {
        let mut vec = Vec::new();
        let mut buffer = [0; CHUNK_SIZE];
        for (i, x) in cases.iter().enumerate() {
            buffer[i % 32] = *x;
            if i % 32 == 31 {
                vec.push(buffer);
                buffer = [0; CHUNK_SIZE];
            }
        }
        vec.push(buffer);
        //println!("{:?}", vec);
        Self { cases: vec }
    }

    #[inline]
    unsafe fn get_double_value(&self, index: usize) -> (usize, usize) {
        let i = index % CHUNK_SIZE;
        let tmp = index / CHUNK_SIZE;
        if i != CHUNK_SIZE {
            if let Some(e) = self.cases.get(tmp) {
                let mut i = e.iter().skip(index);
                (*i.next().unwrap_or(&0), *i.next().unwrap_or(&0))
            } else {
                (0, 0)
            }
        } else if let Some(e) = self.cases.get(tmp + 1) {
            (self.cases.get_unchecked(tmp)[CHUNK_SIZE_INDEXED], e[0])
        } else if let Some(e) = self.cases.get(tmp) {
            (e[CHUNK_SIZE_INDEXED], 0)
        } else {
            (0, 0)
        }
    }

    #[allow(unused)]
    fn as_vec(&self) -> Vec<usize> {
        let mut linear = Vec::new();
        let mut unused = false;
        for i in self.cases.iter().rev() {
            for p in i.iter().rev() {
                if !unused {
                    if p == &0 {
                        continue;
                    }
                    unused = true;
                }
                linear.push(p);
            }
        }
        linear.into_iter().rev().copied().collect::<Vec<usize>>()
    }
}

use crate::cythan::Cythan;

impl Cythan for ChunkedCythan {
    #[inline]
    fn next(&mut self) {
        unsafe {
            let index = if self.cases.is_empty() {
                self.cases.push(VEC);
                0
            } else {
                let index = self.cases.get_unchecked_mut(0);
                let o = index[0];
                index[0] += 2;
                o
            };

            let (c2, c1) = self.get_double_value(index);

            self.set_value(c1, self.get_value(c2));
        }
    }

    #[inline]
    fn get_value(&self, index: usize) -> usize {
        if let Some(e) = self.cases.get(index / CHUNK_SIZE) {
            e[index % CHUNK_SIZE]
        } else {
            0
        }
    }

    #[inline]
    fn set_value(&mut self, index: usize, value: usize) {
        let chunk_size = index / CHUNK_SIZE;
        if self.cases.len() <= chunk_size {
            self.cases
                .extend((self.cases.len()..chunk_size).map(|_| [0; CHUNK_SIZE]));
            let mut chunk = [0; CHUNK_SIZE];
            chunk[index % CHUNK_SIZE] = value;
            self.cases.push(chunk);
        } else {
            unsafe {
                self.cases.get_unchecked_mut(chunk_size)[index % CHUNK_SIZE] = value;
            }
        }
    }
}

#[test]
fn basic_test_if() {
    let mut cythan = ChunkedCythan::new(vec![1, 9, 5, 10, 1, 0, 0, 11, 0, 1, 20, 21]);
    for a in 0..10 {
        cythan.next();
    }
    assert_eq!(
        cythan.as_vec(),
        vec![34, 20, 5, 10, 1, 1, 0, 11, 0, 1, 20, 21]
    );
}
#[test]
fn basic_test_simple() {
    let mut cythan = ChunkedCythan::new(vec![1, 5, 3, 0, 0, 999]);
    cythan.next();
    assert_eq!(cythan.as_vec(), vec![3, 5, 3, 999, 0, 999]);
}
