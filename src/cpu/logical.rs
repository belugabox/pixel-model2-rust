//! Opérations logiques et binaires pour le NEC V60

use super::arithmetic::ArithmeticResult;

/// Unité logique et binaire pour le NEC V60
pub struct LogicalUnit;

impl LogicalUnit {
    /// Opération AND logique
    pub fn and(operand1: u32, operand2: u32) -> ArithmeticResult {
        let result = operand1 & operand2;
        // Les opérations logiques n'ont pas de carry ni d'overflow
        ArithmeticResult::new(result, false, false)
    }

    /// Opération OR logique
    pub fn or(operand1: u32, operand2: u32) -> ArithmeticResult {
        let result = operand1 | operand2;
        ArithmeticResult::new(result, false, false)
    }

    /// Opération XOR logique
    pub fn xor(operand1: u32, operand2: u32) -> ArithmeticResult {
        let result = operand1 ^ operand2;
        ArithmeticResult::new(result, false, false)
    }

    /// Opération NOT logique (complément à un)
    pub fn not(operand: u32) -> ArithmeticResult {
        let result = !operand;
        ArithmeticResult::new(result, false, false)
    }

    /// Décalage logique à gauche (Shift Left Logical)
    pub fn shl(operand: u32, shift_amount: u32) -> ArithmeticResult {
        let shift = shift_amount & 0x1F; // Masquer à 5 bits (0-31)

        if shift == 0 {
            return ArithmeticResult::new(operand, false, false);
        }

        let result = operand << shift;
        // Carry = dernier bit décalé vers l'extérieur
        let carry = if shift <= 32 {
            (operand >> (32 - shift)) & 1 != 0
        } else {
            false
        };

        ArithmeticResult::new(result, carry, false)
    }

    /// Décalage logique à droite (Shift Right Logical)
    pub fn shr(operand: u32, shift_amount: u32) -> ArithmeticResult {
        let shift = shift_amount & 0x1F; // Masquer à 5 bits (0-31)

        if shift == 0 {
            return ArithmeticResult::new(operand, false, false);
        }

        let result = operand >> shift;
        // Carry = dernier bit décalé vers l'extérieur
        let carry = if shift <= 32 {
            (operand >> (shift - 1)) & 1 != 0
        } else {
            false
        };

        ArithmeticResult::new(result, carry, false)
    }

    /// Décalage arithmétique à droite (Shift Right Arithmetic)
    /// Préserve le bit de signe
    pub fn sar(operand: u32, shift_amount: u32) -> ArithmeticResult {
        let shift = shift_amount & 0x1F; // Masquer à 5 bits (0-31)

        if shift == 0 {
            return ArithmeticResult::new(operand, false, false);
        }

        let signed_operand = operand as i32;
        let result = (signed_operand >> shift) as u32;

        // Carry = dernier bit décalé vers l'extérieur
        let carry = if shift <= 32 {
            (operand >> (shift - 1)) & 1 != 0
        } else {
            signed_operand < 0 // Si tout est décalé, carry = bit de signe
        };

        ArithmeticResult::new(result, carry, false)
    }

    /// Rotation à gauche (Rotate Left)
    pub fn rol(operand: u32, rotation_amount: u32) -> ArithmeticResult {
        let rotation = rotation_amount & 0x1F; // Masquer à 5 bits

        if rotation == 0 {
            return ArithmeticResult::new(operand, false, false);
        }

        let result = operand.rotate_left(rotation);
        // Carry = bit qui a été tourné dans la position LSB
        let carry = result & 1 != 0;

        ArithmeticResult::new(result, carry, false)
    }

    /// Rotation à droite (Rotate Right)
    pub fn ror(operand: u32, rotation_amount: u32) -> ArithmeticResult {
        let rotation = rotation_amount & 0x1F; // Masquer à 5 bits

        if rotation == 0 {
            return ArithmeticResult::new(operand, false, false);
        }

        let result = operand.rotate_right(rotation);
        // Carry = bit qui a été tourné dans la position MSB
        let carry = (result >> 31) & 1 != 0;

        ArithmeticResult::new(result, carry, false)
    }

    /// Test de bits (comme AND mais ne stocke pas le résultat)
    pub fn test(operand1: u32, operand2: u32) -> ArithmeticResult {
        let result = operand1 & operand2;
        ArithmeticResult::new(result, false, false)
    }

    /// Compte les bits à 1 dans un mot
    pub fn bit_count(operand: u32) -> ArithmeticResult {
        let result = operand.count_ones();
        ArithmeticResult::new(result, false, false)
    }

    /// Trouve la position du premier bit à 1 (BSF - Bit Scan Forward)
    pub fn bit_scan_forward(operand: u32) -> ArithmeticResult {
        if operand == 0 {
            // Convention: si aucun bit n'est trouvé, résultat est indéfini mais zero flag est mis
            return ArithmeticResult::new(0, false, false);
        }

        let result = operand.trailing_zeros();
        ArithmeticResult::new(result, false, false)
    }

    /// Trouve la position du dernier bit à 1 (BSR - Bit Scan Reverse)
    pub fn bit_scan_reverse(operand: u32) -> ArithmeticResult {
        if operand == 0 {
            return ArithmeticResult::new(0, false, false);
        }

        let result = 31 - operand.leading_zeros();
        ArithmeticResult::new(result, false, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logical_and() {
        let result = LogicalUnit::and(0xFF00, 0x0FFF);
        assert_eq!(result.value, 0x0F00);
        assert!(!result.carry);
        assert!(!result.overflow);
    }

    #[test]
    fn test_logical_or() {
        let result = LogicalUnit::or(0xFF00, 0x00FF);
        assert_eq!(result.value, 0xFFFF);
    }

    #[test]
    fn test_logical_xor() {
        let result = LogicalUnit::xor(0xFFFF, 0xFF00);
        assert_eq!(result.value, 0x00FF);
    }

    #[test]
    fn test_logical_not() {
        let result = LogicalUnit::not(0xFFFF0000);
        assert_eq!(result.value, 0x0000FFFF);
    }

    #[test]
    fn test_shift_left() {
        let result = LogicalUnit::shl(0x12345678, 4);
        assert_eq!(result.value, 0x23456780);
        assert!(result.carry); // Bit 28 était à 1
    }

    #[test]
    fn test_shift_right() {
        let result = LogicalUnit::shr(0x12345678, 4);
        assert_eq!(result.value, 0x01234567);
        assert!(result.carry); // Bit 3 était à 1 (8 >> 4)
    }

    #[test]
    fn test_rotate_left() {
        let result = LogicalUnit::rol(0x80000001, 1);
        assert_eq!(result.value, 0x00000003);
        assert!(result.carry);
    }

    #[test]
    fn test_bit_scan_forward() {
        let result = LogicalUnit::bit_scan_forward(0x00000008); // Bit 3
        assert_eq!(result.value, 3);
    }

    #[test]
    fn test_bit_scan_reverse() {
        let result = LogicalUnit::bit_scan_reverse(0x80000000); // Bit 31
        assert_eq!(result.value, 31);
    }
}
