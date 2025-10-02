//! Unité de calcul en virgule flottante NEC V60

use super::registers::ProcessorStatusWord;

/// Résultat d'une opération en virgule flottante
#[derive(Debug)]
pub struct FloatResult {
    pub value: f32,
    pub overflow: bool,
    pub underflow: bool,
    pub zero: bool,
    pub nan: bool,
    pub infinite: bool,
}

impl FloatResult {
    /// Met à jour le mot d'état du processeur avec les flags appropriés
    pub fn update_psw(&self, psw: &mut ProcessorStatusWord) {
        psw.set(ProcessorStatusWord::ZERO, self.zero);
        psw.set(ProcessorStatusWord::OVERFLOW, self.overflow);
        psw.set(ProcessorStatusWord::CARRY, self.underflow);

        // Flag spécial pour les NaN et infinis
        if self.nan || self.infinite {
            psw.insert(ProcessorStatusWord::PARITY);
        } else {
            psw.remove(ProcessorStatusWord::PARITY);
        }
    }

    /// Convertit le résultat float en représentation u32 (IEEE 754)
    pub fn to_u32(&self) -> u32 {
        self.value.to_bits()
    }
}

/// Unité de calcul en virgule flottante
pub struct FloatingPointUnit;

impl FloatingPointUnit {
    /// Addition en virgule flottante
    pub fn add(a_bits: u32, b_bits: u32) -> FloatResult {
        let a = f32::from_bits(a_bits);
        let b = f32::from_bits(b_bits);

        let result = a + b;

        FloatResult {
            value: result,
            overflow: result.is_infinite() && a.is_finite() && b.is_finite(),
            underflow: result == 0.0 && (a != 0.0 || b != 0.0),
            zero: result == 0.0,
            nan: result.is_nan(),
            infinite: result.is_infinite(),
        }
    }

    /// Soustraction en virgule flottante
    pub fn sub(a_bits: u32, b_bits: u32) -> FloatResult {
        let a = f32::from_bits(a_bits);
        let b = f32::from_bits(b_bits);

        let result = a - b;

        FloatResult {
            value: result,
            overflow: result.is_infinite() && a.is_finite() && b.is_finite(),
            underflow: result == 0.0 && (a != 0.0 || b != 0.0),
            zero: result == 0.0,
            nan: result.is_nan(),
            infinite: result.is_infinite(),
        }
    }

    /// Multiplication en virgule flottante
    pub fn mul(a_bits: u32, b_bits: u32) -> FloatResult {
        let a = f32::from_bits(a_bits);
        let b = f32::from_bits(b_bits);

        let result = a * b;

        FloatResult {
            value: result,
            overflow: result.is_infinite() && a.is_finite() && b.is_finite(),
            underflow: result == 0.0 && (a != 0.0 && b != 0.0),
            zero: result == 0.0,
            nan: result.is_nan(),
            infinite: result.is_infinite(),
        }
    }

    /// Division en virgule flottante
    pub fn div(a_bits: u32, b_bits: u32) -> FloatResult {
        let a = f32::from_bits(a_bits);
        let b = f32::from_bits(b_bits);

        if b == 0.0 && a != 0.0 {
            return FloatResult {
                value: if a > 0.0 {
                    f32::INFINITY
                } else {
                    f32::NEG_INFINITY
                },
                overflow: true,
                underflow: false,
                zero: false,
                nan: false,
                infinite: true,
            };
        }

        let result = a / b;

        FloatResult {
            value: result,
            overflow: result.is_infinite() && a.is_finite() && b.is_finite(),
            underflow: result == 0.0 && a != 0.0 && b.is_finite(),
            zero: result == 0.0,
            nan: result.is_nan(),
            infinite: result.is_infinite(),
        }
    }

    /// Comparaison en virgule flottante
    pub fn compare(a_bits: u32, b_bits: u32) -> FloatResult {
        let a = f32::from_bits(a_bits);
        let b = f32::from_bits(b_bits);

        // Si l'un des nombres est NaN, le résultat est indéterminé
        if a.is_nan() || b.is_nan() {
            return FloatResult {
                value: 0.0,
                overflow: false,
                underflow: false,
                zero: false,
                nan: true,
                infinite: false,
            };
        }

        FloatResult {
            value: 0.0, // Les comparaisons ne retournent pas de valeur
            overflow: false,
            underflow: false,
            zero: a == b,
            nan: false,
            infinite: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_add() {
        let a = std::f32::consts::PI.to_bits();
        let b = 2.86f32.to_bits();
        let result = FloatingPointUnit::add(a, b);

        assert!((result.value - 6.0).abs() < 0.01);
        assert!(!result.overflow);
        assert!(!result.zero);
    }

    #[test]
    fn test_float_div_by_zero() {
        let a = 1.0f32.to_bits();
        let b = 0.0f32.to_bits();
        let result = FloatingPointUnit::div(a, b);

        assert!(result.infinite);
        assert!(result.overflow);
        assert!(result.value.is_infinite());
    }

    #[test]
    fn test_float_compare() {
        let a = 5.0f32.to_bits();
        let b = 5.0f32.to_bits();
        let result = FloatingPointUnit::compare(a, b);

        assert!(result.zero); // Égaux
        assert!(!result.nan);
    }
}
