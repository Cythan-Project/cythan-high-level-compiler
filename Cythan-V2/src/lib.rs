/*!
 * The Cythan machine emulator librairy.
 *
 * The Cythan machine is a mathematical Turing Complete computer.
 * The machine is composed of one vector. Each value of the vector is a positive integer, "pointing" to another value.
 *
 * ### For every iteration of the machine
 *
 *  - The first case (the pointer), is incremented by 2.
 *
 *  - The 2 cases pointed by the first case before the incrementation is "executed". In a pair of executed cases, the case that as for index the second value is set to the value of the case that have as index the first value
 *
 * For instance, `1,5,3,0,0,999` will copy the content of the 5th case (999) into the 3rd one. The result after one iteration will be `3,5,3,999,0,999`
 *
 * ### Example
 *
 * ```rust
 * use cythan::{Cythan,BasicCythan};
 * let mut cythan = BasicCythan::new( vec![1,9,5,10,1,0,0,11,0,1,20,21] );
 * println!("Cythan start:{}",cythan);
 * for a in 0..10 {
 *    cythan.next();
 *    println!("Cythan iteration {}:{}",a,cythan)
 * }
 * ```
 *
 * ### Implementations
 *  - [`BasicCythan`](implementations/basic/struct.BasicCythan.html) Fast, simple, for most use cases
 *  - [`ChunkedCythan`](implementations/chunked/struct.ChunkedCythan.html) Use memory chunking to increase speed on large programs
 *  - [`CompleteCythan`](implementations/complete/struct.CompleteCythan.html) This is a fully configurable Cythan Machine but twice as slow as than the basic one.
 *
*/

pub mod cythan;
pub mod implementations;
pub use implementations::*;

pub use crate::cythan::Cythan;
