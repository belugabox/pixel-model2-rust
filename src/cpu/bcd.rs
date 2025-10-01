//! Instructions BCD (Binary Coded Decimal) NEC V60

use super::registers::ProcessorStatusWord;

/// Résultat d'une opération BCD
#[derive(Debug)]
pub struct BcdResult {
    pub value: u32,
    pub carry: bool,
    pub overflow: bool,
    pub zero: bool,
    pub adjust_needed: bool,
}

impl BcdResult {
    /// Met à jour le mot d'état du processeur
    pub fn update_psw(&self, psw: &mut ProcessorStatusWord) {
        psw.set(ProcessorStatusWord::ZERO, self.zero);
        psw.set(ProcessorStatusWord::CARRY, self.carry);
        psw.set(ProcessorStatusWord::OVERFLOW, self.overflow);
        psw.set(ProcessorStatusWord::PARITY, self.adjust_needed);
    }
}

/// Unité de calcul BCD
pub struct BcdUnit;

impl BcdUnit {
    /// Vérifie si une valeur est un BCD valide
    pub fn is_valid_bcd(value: u32) -> bool {
        // Chaque nibble (4 bits) doit être entre 0 et 9
        for i in 0..8 {
            let nibble = (value >> (i * 4)) & 0xF;
            if nibble > 9 {
                return false;
            }
        }
        true
    }

    /// Convertit un BCD en binaire
    pub fn bcd_to_binary(bcd: u32) -> u32 {
        let mut result = 0;
        let mut multiplier = 1;
        
        for i in 0..8 {
            let nibble = (bcd >> (i * 4)) & 0xF;
            if nibble > 9 {
                break; // BCD invalide
            }
            result += nibble * multiplier;
            multiplier *= 10;
        }
        
        result
    }

    /// Convertit un binaire en BCD
    pub fn binary_to_bcd(mut binary: u32) -> u32 {
        let mut result = 0;
        let mut shift = 0;
        
        while binary > 0 && shift < 32 {
            let digit = binary % 10;
            result |= digit << shift;
            binary /= 10;
            shift += 4;
        }
        
        result
    }

    /// Addition BCD
    pub fn add(a: u32, b: u32) -> BcdResult {
        if !Self::is_valid_bcd(a) || !Self::is_valid_bcd(b) {
            return BcdResult {
                value: 0,
                carry: false,
                overflow: true,
                zero: false,
                adjust_needed: true,
            };
        }

        let mut result = 0;
        let mut carry = 0;
        let mut final_carry = false;
        
        for i in 0..8 {
            let nibble_a = (a >> (i * 4)) & 0xF;
            let nibble_b = (b >> (i * 4)) & 0xF;
            
            let sum = nibble_a + nibble_b + carry;
            
            if sum > 9 {
                result |= (sum - 10) << (i * 4);
                carry = 1;
            } else {
                result |= sum << (i * 4);
                carry = 0;
            }
            
            // Si on sort de la boucle avec un carry
            if i == 7 && carry == 1 {
                final_carry = true;
            }
        }

        BcdResult {
            value: result,
            carry: final_carry,
            overflow: false,
            zero: result == 0,
            adjust_needed: false,
        }
    }

    /// Soustraction BCD
    pub fn sub(a: u32, b: u32) -> BcdResult {
        if !Self::is_valid_bcd(a) || !Self::is_valid_bcd(b) {
            return BcdResult {
                value: 0,
                carry: false,
                overflow: true,
                zero: false,
                adjust_needed: true,
            };
        }

        let mut result = 0;
        let mut borrow = 0;
        let mut final_borrow = false;
        
        for i in 0..8 {
            let nibble_a = (a >> (i * 4)) & 0xF;
            let nibble_b = (b >> (i * 4)) & 0xF;
            
            let mut diff = nibble_a as i32 - nibble_b as i32 - borrow;
            
            if diff < 0 {
                diff += 16; // Emprunter du nibble suivant
                result |= ((diff - 6) as u32 & 0xF) << (i * 4);
                borrow = 1;
                if i == 7 {
                    final_borrow = true;
                }
            } else if diff > 9 {
                result |= ((diff - 6) as u32 & 0xF) << (i * 4);
                borrow = 0;
            } else {
                result |= (diff as u32) << (i * 4);
                borrow = 0;
            }
        }

        BcdResult {
            value: result,
            carry: final_borrow,
            overflow: false,
            zero: result == 0,
            adjust_needed: false,
        }
    }

    /// Ajustement décimal après addition (DAA - Decimal Adjust Accumulator)
    pub fn decimal_adjust_add(value: u32, carry_in: bool) -> BcdResult {
        let mut result = value;
        let mut carry_out = carry_in;
        let mut adjust_needed = false;

        // Ajuster chaque nibble
        for i in 0..8 {
            let nibble = (result >> (i * 4)) & 0xF;
            
            if nibble > 9 || (i == 0 && carry_in) {
                let adjusted = nibble + 6;
                result = (result & !(0xF << (i * 4))) | ((adjusted & 0xF) << (i * 4));
                
                if adjusted > 0xF {
                    if i < 7 {
                        let next_nibble = (result >> ((i + 1) * 4)) & 0xF;
                        result = (result & !(0xF << ((i + 1) * 4))) | ((next_nibble + 1) << ((i + 1) * 4));
                    } else {
                        carry_out = true;
                    }
                }
                adjust_needed = true;
            }
        }

        BcdResult {
            value: result,
            carry: carry_out,
            overflow: false,
            zero: result == 0,
            adjust_needed,
        }
    }

    /// Ajustement décimal après soustraction (DAS - Decimal Adjust Subtraction)
    pub fn decimal_adjust_sub(value: u32, borrow_in: bool) -> BcdResult {
        let mut result = value;
        let mut borrow_out = borrow_in;
        let mut adjust_needed = false;

        // Ajuster chaque nibble
        for i in 0..8 {
            let nibble = (result >> (i * 4)) & 0xF;
            
            if nibble > 9 || (i == 0 && borrow_in) {
                let adjusted = nibble.wrapping_sub(6);
                result = (result & !(0xF << (i * 4))) | ((adjusted & 0xF) << (i * 4));
                adjust_needed = true;
                
                if nibble < 6 {
                    borrow_out = true;
                }
            }
        }

        BcdResult {
            value: result,
            carry: borrow_out,
            overflow: false,
            zero: result == 0,
            adjust_needed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bcd_validation() {
        assert!(BcdUnit::is_valid_bcd(0x1234));
        assert!(BcdUnit::is_valid_bcd(0x9876));
        assert!(!BcdUnit::is_valid_bcd(0xABCD)); // A, B, C, D > 9
        assert!(!BcdUnit::is_valid_bcd(0x123A)); // A > 9
    }

    #[test]
    fn test_bcd_conversion() {
        assert_eq!(BcdUnit::bcd_to_binary(0x1234), 1234);
        assert_eq!(BcdUnit::bcd_to_binary(0x9876), 9876);
        
        assert_eq!(BcdUnit::binary_to_bcd(1234), 0x1234);
        assert_eq!(BcdUnit::binary_to_bcd(9876), 0x9876);
    }

    #[test]
    fn test_bcd_add() {
        // 23 + 45 = 68 en BCD
        let result = BcdUnit::add(0x23, 0x45);
        assert_eq!(result.value, 0x68);
        assert!(!result.carry);
        assert!(!result.zero);

        // 99 + 01 = 100 en BCD (pas de carry car le résultat tient dans 8 nibbles)
        let result = BcdUnit::add(0x99, 0x01);
        assert_eq!(result.value, 0x100);
        assert!(!result.carry);
        assert!(!result.zero);
    }

    #[test]
    fn test_bcd_sub() {
        // 68 - 23 = 45 en BCD
        let result = BcdUnit::sub(0x68, 0x23);
        assert_eq!(result.value, 0x45);
        assert!(!result.carry);

        // 23 - 45 nécessiterait un emprunt
        let result = BcdUnit::sub(0x23, 0x45);
        assert!(result.carry); // Borrow set
    }

    #[test]
    fn test_decimal_adjust() {
        // Test après addition avec overflow
        let result = BcdUnit::decimal_adjust_add(0x9F, false); // F > 9, doit être ajusté
        assert!(result.adjust_needed);
    }
}