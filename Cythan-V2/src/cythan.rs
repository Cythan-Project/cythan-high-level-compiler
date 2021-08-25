
/// This is the generic trait for all Cythan machine implementations
pub trait Cythan: std::fmt::Display {

    /// This function is a generic function to compute a Cythan iteration
    fn next(&mut self);

    /// This function is a generic function to get a value from Cythan memory
    fn get_value(&self, index: usize) -> usize;
    
    /// This function is a generic function to set a value from Cythan memory
    fn set_value(&mut self, index: usize, value: usize);
}