/// This provides an implementation for every kind of Cythan machine.
/// The complete implementation is twice as slow than the basic implementation.
///
/// You can configure the Cythan machine by using different constructors:
/// ```rust
/// use cythan::{Cythan,CompleteCythan};
///
/// // This function create a Cythan Machine with a step of 2 and a base value of 0
/// let machine = CompleteCythan::new(vec![12,23,45,20,0]);
///
/// // This function create a Cythan Machine with a step of 3 and a base value of 1
/// let machine = CompleteCythan::new_static_value(vec![12,23,45,20,0],3,1);
///
/// // This function create a Cythan Machine with a step of 4 and a base value of index * 2
/// let machine = CompleteCythan::new_config(vec![12,23,45,20,0],4,Box::new(|x| x*2));
/// ```
pub struct CompleteCythan {
    pub cases: Vec<usize>,
    pub step: usize,
    pub generator: DefaultGenerator,
}

impl std::fmt::Display for CompleteCythan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cythan(step: {}): {:?}", self.step, self.cases)
    }
}

impl CompleteCythan {
    /// The constructor of the Cythan machine with a step of 2 and a base value of 0
    pub fn new(cases: Vec<usize>) -> CompleteCythan {
        CompleteCythan {
            cases,
            step: 2,
            generator: DefaultGenerator::FixedValue(0),
        }
    }

    /// The constructor of the Cythan machine with selected step and a base value computed by a function (index) -> (base value)
    pub fn new_config(
        cases: Vec<usize>,
        step: usize,
        generator: Box<dyn Fn(usize) -> usize>,
    ) -> CompleteCythan {
        CompleteCythan {
            cases,
            step,
            generator: DefaultGenerator::Function(generator),
        }
    }

    /// The constructor of the Cythan machine with a selected step and a fixed base value
    pub fn new_static_value(cases: Vec<usize>, step: usize, generator: usize) -> CompleteCythan {
        CompleteCythan {
            cases,
            step,
            generator: DefaultGenerator::FixedValue(generator),
        }
    }

    #[inline]
    fn get_both_values(&self, index: usize) -> (usize, usize) {
        let mut i = self.cases.iter().skip(index);
        (
            *i.next().unwrap_or(&(self.generator.generate(index))),
            *i.next().unwrap_or(&(self.generator.generate(index + 1))),
        )
    }

    #[inline]
    unsafe fn get_mut_value(&mut self, index: usize) -> &mut usize {
        if self.cases.len() <= index {
            let iter = (self.cases.len()..index + 1)
                .map(|x| self.generator.generate(x))
                .collect::<Vec<usize>>();
            self.cases.extend(iter);
        }
        self.cases.get_unchecked_mut(index)
    }
}

use crate::cythan::Cythan;

impl Cythan for CompleteCythan {
    #[inline]
    fn next(&mut self) {
        unsafe {
            let step = self.step;
            let index = {
                let index = self.get_mut_value(0);
                *index += step;
                *index
            };

            let (c2, c1) = self.get_both_values(index - step);

            self.set_value(c1, self.get_value(c2));
        }
    }

    #[inline]
    fn get_value(&self, index: usize) -> usize {
        *self
            .cases
            .get(index)
            .unwrap_or(&(self.generator.generate(index)))
    }

    #[inline]
    fn set_value(&mut self, index: usize, value: usize) {
        if self.cases.len() <= index {
            let iter = (self.cases.len()..index)
                .map(|x| self.generator.generate(x))
                .collect::<Vec<usize>>();
            self.cases.extend(iter);
            self.cases.push(value);
        } else {
            unsafe {
                *self.cases.get_unchecked_mut(index) = value;
            }
        }
    }
}

pub enum DefaultGenerator {
    Function(Box<dyn Fn(usize) -> usize>),
    FixedValue(usize),
}

impl DefaultGenerator {
    #[inline]
    fn generate(&self, index: usize) -> usize {
        match self {
            DefaultGenerator::Function(fct) => (fct)(index),
            DefaultGenerator::FixedValue(f) => *f,
        }
    }
}

#[test]
fn test_if() {
    let mut cythan = CompleteCythan::new(vec![1, 9, 5, 10, 1, 0, 0, 11, 0, 1, 20, 21]);
    for a in 0..10 {
        cythan.next();
    }
    assert_eq!(cythan.cases, vec![34, 20, 5, 10, 1, 1, 0, 11, 0, 1, 20, 21]);
}
#[test]
fn test_simple() {
    let mut cythan = CompleteCythan::new(vec![1, 5, 3, 0, 0, 999]);
    cythan.next();
    assert_eq!(cythan.cases, vec![3, 5, 3, 999, 0, 999]);
}

#[test]
fn test_junk() {
    let mut cythan = CompleteCythan::new_static_value(vec![1, 0, 10], 2, 3);
    cythan.next();
    assert_eq!(cythan.cases, vec![3, 0, 10, 3, 3, 3, 3, 3, 3, 3, 3]);
}

#[test]
fn test_double() {
    let mut cythan = CompleteCythan::new_config(vec![1], 2, Box::new(|x| x * 2));
    for a in 0..10 {
        cythan.next();
    }
    assert_eq!(
        cythan.cases,
        vec![
            21, 2, 4, 6, 12, 10, 12, 14, 16, 18, 20, 22, 20, 26, 28, 30, 28, 34, 36, 38, 44, 42,
            44, 46, 48, 50, 52, 54, 60, 58, 60, 62, 64, 66, 68, 70, 68, 74, 76, 78, 80, 82, 84, 86,
            76
        ]
    );
}
