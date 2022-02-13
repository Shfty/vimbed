use std::borrow::Borrow;

// Character length trait
pub trait CharLen {
    fn char_len(&self) -> usize;
}

impl<T> CharLen for T
where
    T: Borrow<str>,
{
    fn char_len(&self) -> usize {
        self.borrow().chars().count()
    }
}
