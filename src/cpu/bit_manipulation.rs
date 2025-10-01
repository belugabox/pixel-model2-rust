//! Instructions de manipulation de bits NEC V60

use super::registers::ProcessorStatusWord;

/// Résultat d'une opération sur les bits
#[derive(Debug)]
pub struct BitResult {
    pub value: u32,
    pub bit_found: bool,
    pub bit_position: Option<u8>,
}

impl BitResult {
    /// Met à jour le mot d'état du processeur
    pub fn update_psw(&self, psw: &mut ProcessorStatusWord) {
        psw.set(ProcessorStatusWord::ZERO, !self.bit_found);
        psw.set(ProcessorStatusWord::CARRY, self.bit_found);
    }
}

/// Unité de manipulation de bits
pub struct BitManipulationUnit;

impl BitManipulationUnit {
    /// Test d'un bit (BIT_TEST)
    pub fn test_bit(value: u32, bit_position: u32) -> BitResult {
        if bit_position >= 32 {
            return BitResult {
                value: 0,
                bit_found: false,
                bit_position: None,
            };
        }
        
        let bit_set = (value & (1 << bit_position)) != 0;
        
        BitResult {
            value: if bit_set { 1 } else { 0 },
            bit_found: bit_set,
            bit_position: Some(bit_position as u8),
        }
    }

    /// Mise à 1 d'un bit (BIT_SET)
    pub fn set_bit(value: u32, bit_position: u32) -> BitResult {
        if bit_position >= 32 {
            return BitResult {
                value,
                bit_found: false,
                bit_position: None,
            };
        }
        
        let new_value = value | (1 << bit_position);
        let was_set = (value & (1 << bit_position)) != 0;
        
        BitResult {
            value: new_value,
            bit_found: was_set,
            bit_position: Some(bit_position as u8),
        }
    }

    /// Mise à 0 d'un bit (BIT_CLEAR)
    pub fn clear_bit(value: u32, bit_position: u32) -> BitResult {
        if bit_position >= 32 {
            return BitResult {
                value,
                bit_found: false,
                bit_position: None,
            };
        }
        
        let new_value = value & !(1 << bit_position);
        let was_set = (value & (1 << bit_position)) != 0;
        
        BitResult {
            value: new_value,
            bit_found: was_set,
            bit_position: Some(bit_position as u8),
        }
    }

    /// Recherche de bit (BIT_SCAN) - trouve le premier bit à 1
    pub fn scan_forward(value: u32) -> BitResult {
        if value == 0 {
            return BitResult {
                value: 32, // Convention : 32 si aucun bit trouvé
                bit_found: false,
                bit_position: None,
            };
        }
        
        // Utilise l'instruction BSF (Bit Scan Forward) équivalente
        let position = value.trailing_zeros();
        
        BitResult {
            value: position,
            bit_found: true,
            bit_position: Some(position as u8),
        }
    }

    /// Recherche de bit inversée - trouve le premier bit à 0
    pub fn scan_reverse(value: u32) -> BitResult {
        if value == 0xFFFFFFFF {
            return BitResult {
                value: 32,
                bit_found: false,
                bit_position: None,
            };
        }
        
        // Inverse les bits et cherche le premier 1
        let inverted = !value;
        let position = inverted.trailing_zeros();
        
        BitResult {
            value: position,
            bit_found: true,
            bit_position: Some(position as u8),
        }
    }

    /// Rotation à gauche (ROTATE_LEFT)
    pub fn rotate_left(value: u32, count: u32) -> u32 {
        let count = count % 32; // Limite la rotation à 32 bits
        (value << count) | (value >> (32 - count))
    }

    /// Rotation à droite (ROTATE_RIGHT)
    pub fn rotate_right(value: u32, count: u32) -> u32 {
        let count = count % 32; // Limite la rotation à 32 bits
        (value >> count) | (value << (32 - count))
    }

    /// Compte le nombre de bits à 1 (POPCOUNT)
    pub fn popcount(value: u32) -> u32 {
        value.count_ones()
    }

    /// Trouve le bit de poids fort (Most Significant Bit)
    pub fn find_msb(value: u32) -> BitResult {
        if value == 0 {
            return BitResult {
                value: 32,
                bit_found: false,
                bit_position: None,
            };
        }
        
        let position = 31 - value.leading_zeros();
        
        BitResult {
            value: position,
            bit_found: true,
            bit_position: Some(position as u8),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_operations() {
        // Test BIT_TEST
        let result = BitManipulationUnit::test_bit(0b1010, 1);
        assert!(result.bit_found);
        assert_eq!(result.bit_position, Some(1));

        let result = BitManipulationUnit::test_bit(0b1010, 0);
        assert!(!result.bit_found);

        // Test BIT_SET
        let result = BitManipulationUnit::set_bit(0b1010, 0);
        assert_eq!(result.value, 0b1011);

        // Test BIT_CLEAR
        let result = BitManipulationUnit::clear_bit(0b1010, 1);
        assert_eq!(result.value, 0b1000);
    }

    #[test]
    fn test_bit_scan() {
        // Test SCAN_FORWARD
        let result = BitManipulationUnit::scan_forward(0b1010);
        assert!(result.bit_found);
        assert_eq!(result.value, 1); // Premier bit à 1 en position 1

        // Test avec 0
        let result = BitManipulationUnit::scan_forward(0);
        assert!(!result.bit_found);
        assert_eq!(result.value, 32);
    }

    #[test]
    fn test_rotations() {
        let value = 0b11000000000000000000000000000001u32;
        
        // Rotation à gauche de 1
        let rotated = BitManipulationUnit::rotate_left(value, 1);
        assert_eq!(rotated, 0b10000000000000000000000000000011u32);
        
        // Rotation à droite de 1
        let rotated = BitManipulationUnit::rotate_right(value, 1);
        assert_eq!(rotated, 0b11100000000000000000000000000000u32);
    }

    #[test]
    fn test_popcount() {
        let count = BitManipulationUnit::popcount(0b1010101);
        assert_eq!(count, 4);
    }
}