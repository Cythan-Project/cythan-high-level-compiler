use std::io::Read;

/// This Cythan implementation is optimized to take advantage of a fixed step of 2 and a base value of 0 to get very good performances!
/// This implementation is the fastest on small codes but on larger codes the chunked implemenetation is faster
///
/// ```rust
/// use cythan::{Cythan,InterruptedCythan};
/// // This function create a Cythan Machine with a step of 2 and a base value of 0
/// let machine = InterruptedCythan::new(vec![12,23,45,20,0]);
/// ```
pub struct InterruptedCythan {
    pub cases: Vec<usize>,
    pub base: u8,
    pub interrupt_place: usize,
    pub print_provider: Box<dyn Fn(u8)>,
    pub input_provider: Box<dyn Fn() -> u8>,
}

impl std::fmt::Display for InterruptedCythan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cythan{:?}", self.cases)
    }
}

impl InterruptedCythan {
    /// Create a chunked Cythan Machine with a step of 2 and a base value of 0
    pub fn new(
        cases: Vec<usize>,
        base: u8,
        interrupt_place: usize,
        print_provider: impl Fn(u8) + 'static,
        input_provider: impl Fn() -> u8 + 'static,
    ) -> Self {
        Self {
            cases,
            base: base,
            interrupt_place,
            print_provider: Box::new(print_provider),
            input_provider: Box::new(input_provider),
        }
    }
    pub fn new_stdio(cases: Vec<usize>, base: u8, interrupt_place: usize) -> Self {
        Self::new(
            cases,
            base,
            interrupt_place,
            |a| print!("{}", a as char),
            || std::io::stdin().bytes().next().unwrap().unwrap(),
        )
    }
}

use crate::cythan::Cythan;

impl Cythan for InterruptedCythan {
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
        if index == self.interrupt_place {
            if value == 1 {
                let mut char = 0_u8;
                for n in 0..(8/self.base) {
                    char *= 2_u64.pow(self.base as u32) as u8;
                    char += self.get_value(self.interrupt_place + n as usize +1) as u8 % 2_u64.pow(self.base as u32) as u8;
                }
                char *= 2_u64.pow((8 % self.base) as u32) as u8;
                char += self.get_value(self.interrupt_place + 8_usize/ (self.base as usize) + 2 as usize) as u8 % 2_u64.pow((8 % self.base) as u32) as u8;
                // let a = self.get_value(self.interrupt_place + 1);
                // let b = self.get_value(self.interrupt_place + 2);
                // let char = ((a % self.base) * self.base_as_pow) + (b % self.base_as_pow);
                // (self.print_provider)(char as u8);
                (self.print_provider)( char);
            }
            if value == 2 {
                // println!("INPUT");
                //let o: u8 = std::io::stdin().bytes().next().unwrap().unwrap();
                let mut o: u8 = (self.input_provider)();
                // let a = o % 2_u64.pow(self.base as u32) as u8;
                // let b = o / 2_u64.pow(self.base as u32) as u8;
                // println!("vals:{} {}",a,b);
                
                for n in (0..(8/self.base)).rev() {
                    self.set_value(self.interrupt_place + 1 + (n as usize), (o % (2_u64.pow(self.base as u32) as u8)) as usize);
                    println!("o={}, n={}, set={}",o,n,(o % (2_u64.pow(self.base as u32) as u8)));
                    o /= 2_u64.pow(self.base as u32) as u8;
                }
                println!("o={}",o);
                // self.set_value(self.interrupt_place + 1, b as usize);
                // self.set_value(self.interrupt_place + 2, a as usize);
            }
        }
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
