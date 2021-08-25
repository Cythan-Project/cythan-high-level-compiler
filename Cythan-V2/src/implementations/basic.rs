/// This Cythan implementation is optimized to take advantage of a fixed step of 2 and a base value of 0 to get very good performances!
/// This implementation is the fastest on small codes but on larger codes the chunked implemenetation is faster
///
/// ```rust
/// use cythan::{Cythan,BasicCythan};
/// // This function create a Cythan Machine with a step of 2 and a base value of 0
/// let machine = BasicCythan::new(vec![12,23,45,20,0]);
/// ```
pub struct BasicCythan {
    pub cases: Vec<usize>,
}

impl std::fmt::Display for BasicCythan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cythan{:?}", self.cases)
    }
}

impl BasicCythan {
    /// Create a chunked Cythan Machine with a step of 2 and a base value of 0
    pub fn new(cases: Vec<usize>) -> Self {
        Self { cases }
    }
}

use crate::cythan::Cythan;

impl Cythan for BasicCythan {
    #[inline]
    fn next(&mut self) {
        unsafe {
            let index = if self.cases.is_empty() {
                self.cases.push(2);
                0
            } else {
                let index = self.cases.get_unchecked_mut(0);
                let o = *index;
                *index += 2;
                o
            };

            let (c2, c1) = {
                let mut i = self.cases.iter().skip(index);
                (*i.next().unwrap_or(&0), *i.next().unwrap_or(&0))
            };

            self.set_value(c1, self.get_value(c2));
        }
    }

    #[inline]
    fn get_value(&self, index: usize) -> usize {
        if let Some(e) = self.cases.get(index) {
            *e
        } else {
            0
        }
    }

    #[inline]
    fn set_value(&mut self, index: usize, value: usize) {
        if self.cases.len() <= index {
            if value != 0 {
                self.cases.extend((self.cases.len()..index).map(|_| 0));
                self.cases.push(value);
            }
        } else {
            unsafe {
                *self.cases.get_unchecked_mut(index) = value;
            }
        }
    }
}

#[test]
fn basic_test_if() {
    let mut cythan = BasicCythan::new(vec![1, 9, 5, 10, 1, 0, 0, 11, 0, 1, 20, 21]);
    for a in 0..10 {
        cythan.next();
    }
    assert_eq!(cythan.cases, vec![34, 20, 5, 10, 1, 1, 0, 11, 0, 1, 20, 21]);
}
#[test]
fn basic_test_simple() {
    let mut cythan = BasicCythan::new(vec![1, 5, 3, 0, 0, 999]);
    cythan.next();
    assert_eq!(cythan.cases, vec![3, 5, 3, 999, 0, 999]);
}
