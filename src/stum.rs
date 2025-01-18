//! Spéfications Techniques d'Utilisation du Minitel
//!
//! This module defines the general constants extracted from the STUM1B specification.
//! Reference: <https://jbellue.github.io/stum1b/>

pub mod protocol;
pub mod videotex;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn read_stroke() {
        let seq: Vec<_> = "He?! ".bytes().collect();
        let mut minitel = Minitel::new(std::io::Cursor::new(seq));
        assert_eq!(minitel.read_s0_stroke().unwrap(), UserInput::Char('H'));
        assert_eq!(minitel.read_s0_stroke().unwrap(), UserInput::Char('e'));
        assert_eq!(minitel.read_s0_stroke().unwrap(), UserInput::Char('?'));
        assert_eq!(minitel.read_s0_stroke().unwrap(), UserInput::Char('!'));
        assert_eq!(minitel.read_s0_stroke().unwrap(), UserInput::Char(' '));

        let seq: Vec<_> = vec![0x20, 0x13, 0x41, 0x13, 0x49, 0x20, 0x1B, 0x54];
        let mut minitel = Minitel::new(std::io::Cursor::new(seq));
        assert_eq!(minitel.read_s0_stroke().unwrap(), UserInput::Char(' '));
        assert_eq!(
            minitel.read_s0_stroke().unwrap(),
            UserInput::FunctionKey(FunctionKey::Envoi)
        );
        assert_eq!(
            minitel.read_s0_stroke().unwrap(),
            UserInput::FunctionKey(FunctionKey::ConnexionFin)
        );
        assert_eq!(minitel.read_s0_stroke().unwrap(), UserInput::Char(' '));
        assert_eq!(minitel.read_s0_stroke().unwrap(), UserInput::C1(C1::BgBlue));

        let seq: Vec<_> = vec![0x19, 0x42, 0x65, 0x19, 0x3D]; // SS2, ', e, SS2, ½
        let mut minitel = Minitel::new(std::io::Cursor::new(seq));
        assert_eq!(minitel.read_s0_stroke().unwrap(), UserInput::Char('é'));
        assert_eq!(minitel.read_s0_stroke().unwrap(), UserInput::Char('½'));
    }

    #[test]
    pub fn write_str() {
        let seq: Vec<u8> = Vec::new();
        let mut minitel = Minitel::new(std::io::Cursor::new(seq));
        minitel.write_str("Hé½").unwrap();
        let written = minitel.port.into_inner();
        assert_eq!(written, vec![0x48, 0x19, 0x42, 0x65, 0x19, 0x3D]); // H, SS2, ', e, SS2, ½
    }
}
