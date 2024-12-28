pub mod protocol;
pub mod videotex;

pub trait IntoSequence<const N: usize> {
    /// Sequence of bytes, including the escape sequence
    fn sequence(self) -> [u8; N];
}
