//! Opérations arithmétiques avancées pour le NEC V60

use super::registers::ProcessorStatusWord;
use anyhow::Result;

/// Résultat d'une opération arithmétique avec flags
#[derive(Debug, Clone)]
pub struct ArithmeticResult {
    pub value: u32,
    pub zero: bool,
    pub negative: bool,
    pub carry: bool,
    pub overflow: bool,
    pub parity: bool,
}

impl ArithmeticResult {
    /// Crée un nouveau résultat d'opération arithmétique
    pub fn new(value: u32, carry: bool, overflow: bool) -> Self {
        Self {
            value,
            zero: value == 0,
            negative: (value as i32) < 0,
            carry,
            overflow,
            parity: value.count_ones() % 2 == 0,
        }
    }
    
    /// Met à jour le registre PSW avec les flags calculés
    pub fn update_psw(&self, psw: &mut ProcessorStatusWord) {
        psw.set_zero_flag(self.zero);
        psw.set_negative_flag(self.negative);
        psw.set_carry_flag(self.carry);
        psw.set_overflow_flag(self.overflow);
        psw.set_parity_flag(self.parity);
    }
}

/// Opérations arithmétiques sécurisées avec gestion avancée des flags
pub struct ArithmeticUnit;

impl ArithmeticUnit {
    /// Addition 32-bit avec détection de retenue et débordement
    pub fn add(operand1: u32, operand2: u32) -> ArithmeticResult {
        let (result, carry) = operand1.overflowing_add(operand2);
        
        // Détection de débordement signé
        let overflow = ((operand1 as i32 > 0) && (operand2 as i32 > 0) && ((result as i32) < 0)) ||
                      (((operand1 as i32) < 0) && ((operand2 as i32) < 0) && (result as i32 > 0));
        
        ArithmeticResult::new(result, carry, overflow)
    }
    
    /// Soustraction 32-bit avec détection de retenue et débordement  
    pub fn sub(operand1: u32, operand2: u32) -> ArithmeticResult {
        let (result, carry) = operand1.overflowing_sub(operand2);
        
        // Détection de débordement signé
        let overflow = ((operand1 as i32 > 0) && ((operand2 as i32) < 0) && ((result as i32) < 0)) ||
                      (((operand1 as i32) < 0) && (operand2 as i32 > 0) && (result as i32 > 0));
        
        ArithmeticResult::new(result, carry, overflow)
    }
    
    /// Multiplication 32-bit avec détection de débordement
    pub fn mul(operand1: u32, operand2: u32) -> ArithmeticResult {
        let result_64 = (operand1 as u64) * (operand2 as u64);
        let result = result_64 as u32;
        let overflow = result_64 > u32::MAX as u64;
        let carry = overflow; // En multiplication, carry = overflow
        
        ArithmeticResult::new(result, carry, overflow)
    }
    
    /// Division 32-bit avec gestion des erreurs
    pub fn div(operand1: u32, operand2: u32) -> Result<ArithmeticResult> {
        if operand2 == 0 {
            return Err(anyhow::anyhow!("Division par zéro"));
        }
        
        let result = operand1 / operand2;
        // En division, pas de carry ni overflow dans l'implémentation basique
        Ok(ArithmeticResult::new(result, false, false))
    }
    
    /// Addition avec retenue (ADC - Add with Carry)
    pub fn adc(operand1: u32, operand2: u32, carry_in: bool) -> ArithmeticResult {
        let carry_value = if carry_in { 1 } else { 0 };
        
        // Addition en deux étapes pour gérer la retenue
        let (temp_result, carry1) = operand1.overflowing_add(operand2);
        let (final_result, carry2) = temp_result.overflowing_add(carry_value);
        
        let carry_out = carry1 || carry2;
        
        // Détection de débordement signé avec retenue
        let overflow = ((operand1 as i32 > 0) && (operand2 as i32 > 0) && ((final_result as i32) < 0)) ||
                      (((operand1 as i32) < 0) && ((operand2 as i32) < 0) && (final_result as i32 > 0));
        
        ArithmeticResult::new(final_result, carry_out, overflow)
    }
    
    /// Soustraction avec retenue (SBB - Subtract with Borrow)
    pub fn sbb(operand1: u32, operand2: u32, borrow_in: bool) -> ArithmeticResult {
        let borrow_value = if borrow_in { 1 } else { 0 };
        
        // Soustraction en deux étapes pour gérer l'emprunt
        let (temp_result, borrow1) = operand1.overflowing_sub(operand2);
        let (final_result, borrow2) = temp_result.overflowing_sub(borrow_value);
        
        let borrow_out = borrow1 || borrow2;
        
        // Détection de débordement signé avec emprunt
        let overflow = ((operand1 as i32 > 0) && ((operand2 as i32) < 0) && ((final_result as i32) < 0)) ||
                      (((operand1 as i32) < 0) && (operand2 as i32 > 0) && (final_result as i32 > 0));
        
        ArithmeticResult::new(final_result, borrow_out, overflow)
    }
    
    /// Incrémentation avec gestion des flags
    pub fn inc(operand: u32) -> ArithmeticResult {
        Self::add(operand, 1)
    }
    
    /// Décrémentation avec gestion des flags
    pub fn dec(operand: u32) -> ArithmeticResult {
        Self::sub(operand, 1)
    }
    
    /// Négation (complément à deux)
    pub fn neg(operand: u32) -> ArithmeticResult {
        let result = (!operand).wrapping_add(1);
        let overflow = operand == 0x8000_0000; // Débordement si on nie -2^31
        let carry = operand != 0; // Retenue si l'opérande n'est pas zéro
        
        ArithmeticResult::new(result, carry, overflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arithmetic_add() {
        let result = ArithmeticUnit::add(10, 20);
        assert_eq!(result.value, 30);
        assert!(!result.carry);
        assert!(!result.overflow);
        assert!(!result.zero);
        assert!(!result.negative);
    }
    
    #[test]
    fn test_arithmetic_add_overflow() {
        let result = ArithmeticUnit::add(u32::MAX, 1);
        assert_eq!(result.value, 0);
        assert!(result.carry);
        assert!(result.zero);
    }
    
    #[test]
    fn test_arithmetic_sub() {
        let result = ArithmeticUnit::sub(30, 10);
        assert_eq!(result.value, 20);
        assert!(!result.carry);
        assert!(!result.overflow);
    }
    
    #[test]
    fn test_arithmetic_mul() {
        let result = ArithmeticUnit::mul(6, 7);
        assert_eq!(result.value, 42);
        assert!(!result.overflow);
    }
    
    #[test]
    fn test_arithmetic_div() {
        let result = ArithmeticUnit::div(42, 6).unwrap();
        assert_eq!(result.value, 7);
    }
    
    #[test]
    fn test_arithmetic_div_by_zero() {
        let result = ArithmeticUnit::div(42, 0);
        assert!(result.is_err());
    }
}