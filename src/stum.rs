//! Spéfications Techniques d'Utilisation du Minitel
//!
//! This module defines the general constants extracted from the STUM1B specification.
//! Reference: https://jbellue.github.io/stum1b/

pub mod protocol;
pub mod videotex;

/// Types that can be converted into a sequence of bytes in the
/// minitel serial protocol
pub trait IntoSequence<const N: usize> {
    /// Sequence of bytes, including the escape sequence
    fn sequence(self) -> [u8; N];
}
