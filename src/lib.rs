#![forbid(unsafe_code)]

//! Hardware abstraction for ternary operations.
//!
//! Provides TernaryALU, TernaryMemory (tryte = 6 trits), TernaryRegister,
//! ternary-to-binary conversion, and balanced ternary arithmetic circuits.

// ---------------------------------------------------------------------------
// Trit and Tryte definitions
// ---------------------------------------------------------------------------

/// A balanced ternary trit: -1, 0, or +1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trit {
    Neg,    // -1
    Zero,   //  0
    Pos,    // +1
}

impl Trit {
    pub fn to_i8(self) -> i8 {
        match self {
            Trit::Neg => -1,
            Trit::Zero => 0,
            Trit::Pos => 1,
        }
    }

    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Trit::Neg),
            0 => Some(Trit::Zero),
            1 => Some(Trit::Pos),
            _ => None,
        }
    }

    /// Ternary NOT: Neg↔Pos, Zero stays.
    pub fn ternary_not(self) -> Trit {
        match self {
            Trit::Neg => Trit::Pos,
            Trit::Zero => Trit::Zero,
            Trit::Pos => Trit::Neg,
        }
    }

    /// Ternary AND (min).
    pub fn ternary_and(self, other: Trit) -> Trit {
        use Trit::*;
        match (self, other) {
            (Neg, _) | (_, Neg) => Neg,
            (Zero, _) | (_, Zero) => Zero,
            (Pos, Pos) => Pos,
        }
    }

    /// Ternary OR (max).
    pub fn ternary_or(self, other: Trit) -> Trit {
        use Trit::*;
        match (self, other) {
            (Pos, _) | (_, Pos) => Pos,
            (Zero, _) | (_, Zero) => Zero,
            (Neg, Neg) => Neg,
        }
    }

    /// Consensus of two trits: Pos if both Pos, Neg if both Neg, else Zero.
    pub fn consensus(self, other: Trit) -> Trit {
        if self == other {
            self
        } else {
            Trit::Zero
        }
    }
}

/// A tryte: 6 trits. Range: -364 to +364 in balanced ternary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tryte(pub [Trit; 6]);

impl Tryte {
    /// Zero tryte.
    pub fn zero() -> Self {
        Tryte([Trit::Zero; 6])
    }

    /// Create a tryte from an i32 value.
    pub fn from_i32(mut val: i32) -> Self {
        let mut trits = [Trit::Zero; 6];
        for i in 0..6 {
            let remainder = ((val % 3) + 3) % 3; // ensure non-negative remainder
            if remainder == 0 {
                trits[i] = Trit::Zero;
                val /= 3;
            } else if remainder == 1 {
                trits[i] = Trit::Pos;
                val = (val - 1) / 3;
            } else {
                // remainder == 2 → treat as -1 + carry
                trits[i] = Trit::Neg;
                val = (val + 1) / 3;
            }
        }
        Tryte(trits)
    }

    /// Convert tryte to i32.
    pub fn to_i32(&self) -> i32 {
        let powers: [i32; 6] = [1, 3, 9, 27, 81, 243];
        let mut val = 0i32;
        for i in 0..6 {
            val += self.0[i].to_i8() as i32 * powers[i];
        }
        val
    }

    /// Get individual trit at position (0 = least significant).
    pub fn trit(&self, pos: usize) -> Trit {
        self.0[pos]
    }

    /// Set individual trit.
    pub fn set_trit(&mut self, pos: usize, t: Trit) {
        self.0[pos] = t;
    }
}

// ---------------------------------------------------------------------------
// Ternary Register
// ---------------------------------------------------------------------------

/// A ternary register holding a single tryte.
#[derive(Debug, Clone)]
pub struct TernaryRegister {
    pub value: Tryte,
    pub name: String,
}

impl TernaryRegister {
    pub fn new(name: &str) -> Self {
        Self {
            value: Tryte::zero(),
            name: name.to_string(),
        }
    }

    pub fn load(&mut self, value: &Tryte) {
        self.value = value.clone();
    }

    pub fn read(&self) -> &Tryte {
        &self.value
    }

    pub fn clear(&mut self) {
        self.value = Tryte::zero();
    }
}

// ---------------------------------------------------------------------------
// Ternary Memory
// ---------------------------------------------------------------------------

/// Ternary memory: addressable array of trytes.
pub struct TernaryMemory {
    data: Vec<Tryte>,
    size: usize,
}

impl TernaryMemory {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![Tryte::zero(); size],
            size,
        }
    }

    pub fn read(&self, addr: usize) -> Option<&Tryte> {
        self.data.get(addr)
    }

    pub fn write(&mut self, addr: usize, value: Tryte) -> bool {
        if addr < self.size {
            self.data[addr] = value;
            true
        } else {
            false
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    /// Zero out all memory.
    pub fn clear_all(&mut self) {
        for t in &mut self.data {
            *t = Tryte::zero();
        }
    }
}

// ---------------------------------------------------------------------------
// Ternary ALU
// ---------------------------------------------------------------------------

/// Ternary Arithmetic Logic Unit.
pub struct TernaryALU {
    pub accumulator: TernaryRegister,
    pub temp: TernaryRegister,
}

impl TernaryALU {
    pub fn new() -> Self {
        Self {
            accumulator: TernaryRegister::new("ACC"),
            temp: TernaryRegister::new("TEMP"),
        }
    }

    /// Balanced ternary addition of two trytes with carry.
    fn add_trits(a: Trit, b: Trit, carry_in: Trit) -> (Trit, Trit) {
        use Trit::*;
        let sum = a.to_i8() + b.to_i8() + carry_in.to_i8();
        let (result, carry) = match sum {
            -3 => (Neg, Neg),   // -3 = -1 + carry(-1) → trit=-1, carry=-1
            -2 => (Pos, Neg),   // -2 → trit=+1, carry=-1
            -1 => (Neg, Zero),
            0 => (Zero, Zero),
            1 => (Pos, Zero),
            2 => (Neg, Pos),    // +2 → trit=-1, carry=+1
            3 => (Pos, Pos),    // +3 → trit=+1, carry=+1
            _ => (Zero, Zero),  // shouldn't happen
        };
        (result, carry)
    }

    /// Add two trytes. Returns the result tryte (ignoring overflow).
    pub fn add(&mut self, a: &Tryte, b: &Tryte) -> Tryte {
        let mut result = [Trit::Zero; 6];
        let mut carry = Trit::Zero;
        for i in 0..6 {
            let (r, c) = Self::add_trits(a.trit(i), b.trit(i), carry);
            result[i] = r;
            carry = c;
        }
        Tryte(result)
    }

    /// Subtract: a - b. Complement b and add.
    pub fn subtract(&mut self, a: &Tryte, b: &Tryte) -> Tryte {
        let neg_b = self.negate(b);
        self.add(a, &neg_b)
    }

    /// Negate a tryte (flip all trits).
    pub fn negate(&mut self, t: &Tryte) -> Tryte {
        let mut result = [Trit::Zero; 6];
        for i in 0..6 {
            result[i] = t.trit(i).ternary_not();
        }
        Tryte(result)
    }

    /// Multiply two trytes using shift-and-add.
    pub fn multiply(&mut self, a: &Tryte, b: &Tryte) -> Tryte {
        let av = a.to_i32();
        let bv = b.to_i32();
        // Do multiplication in integer space, convert back
        Tryte::from_i32(av * bv)
    }

    /// Ternary AND operation (tritwise min).
    pub fn and(&self, a: &Tryte, b: &Tryte) -> Tryte {
        let mut result = [Trit::Zero; 6];
        for i in 0..6 {
            result[i] = a.trit(i).ternary_and(b.trit(i));
        }
        Tryte(result)
    }

    /// Ternary OR operation (tritwise max).
    pub fn or(&self, a: &Tryte, b: &Tryte) -> Tryte {
        let mut result = [Trit::Zero; 6];
        for i in 0..6 {
            result[i] = a.trit(i).ternary_or(b.trit(i));
        }
        Tryte(result)
    }

    /// Ternary NOT (tritwise).
    pub fn not(&self, a: &Tryte) -> Tryte {
        let mut result = [Trit::Zero; 6];
        for i in 0..6 {
            result[i] = a.trit(i).ternary_not();
        }
        Tryte(result)
    }

    /// Compare two trytes: returns Neg if a<b, Zero if a==b, Pos if a>b.
    pub fn compare(&self, a: &Tryte, b: &Tryte) -> Trit {
        let av = a.to_i32();
        let bv = b.to_i32();
        if av < bv {
            Trit::Neg
        } else if av > bv {
            Trit::Pos
        } else {
            Trit::Zero
        }
    }

    /// Shift left by one trit (multiply by 3).
    pub fn shift_left(&self, a: &Tryte) -> Tryte {
        let mut result = [Trit::Zero; 6];
        for i in 1..6 {
            result[i] = a.trit(i - 1);
        }
        result[0] = Trit::Zero;
        Tryte(result)
    }

    /// Shift right by one trit (divide by 3, truncating).
    pub fn shift_right(&self, a: &Tryte) -> Tryte {
        let mut result = [Trit::Zero; 6];
        for i in 0..5 {
            result[i] = a.trit(i + 1);
        }
        result[5] = Trit::Zero;
        Tryte(result)
    }
}

// ---------------------------------------------------------------------------
// Ternary-to-Binary Conversion
// ---------------------------------------------------------------------------

/// Convert a tryte to binary representation (as a Vec of bits).
/// Each trit is encoded as 2 bits: Neg=00, Zero=01, Pos=10.
pub fn tryte_to_binary(t: &Tryte) -> Vec<u8> {
    let mut bits = Vec::with_capacity(12);
    for i in 0..6 {
        let pair = match t.trit(i) {
            Trit::Neg => [0u8, 0],
            Trit::Zero => [0, 1],
            Trit::Pos => [1, 0],
        };
        bits.push(pair[0]);
        bits.push(pair[1]);
    }
    bits
}

/// Convert binary (12 bits) back to a tryte.
pub fn binary_to_tryte(bits: &[u8]) -> Option<Tryte> {
    if bits.len() != 12 {
        return None;
    }
    let mut trits = [Trit::Zero; 6];
    for i in 0..6 {
        let hi = bits[i * 2];
        let lo = bits[i * 2 + 1];
        trits[i] = match (hi, lo) {
            (0, 0) => Trit::Neg,
            (0, 1) => Trit::Zero,
            (1, 0) => Trit::Pos,
            _ => return None,
        };
    }
    Some(Tryte(trits))
}

/// Convert a trit to a decimal integer.
pub fn trit_to_decimal(t: Trit) -> i8 {
    t.to_i8()
}

/// Convert a balanced ternary number (as trits, least significant first) to decimal.
pub fn balanced_ternary_to_decimal(trits: &[Trit]) -> i32 {
    let mut val = 0i32;
    let mut power = 1i32;
    for &t in trits {
        val += t.to_i8() as i32 * power;
        power *= 3;
    }
    val
}

/// Convert a decimal integer to balanced ternary trits.
pub fn decimal_to_balanced_ternary(mut val: i32) -> Vec<Trit> {
    if val == 0 {
        return vec![Trit::Zero];
    }
    let mut trits = Vec::new();
    while val != 0 {
        let remainder = ((val % 3) + 3) % 3;
        if remainder == 0 {
            trits.push(Trit::Zero);
            val /= 3;
        } else if remainder == 1 {
            trits.push(Trit::Pos);
            val = (val - 1) / 3;
        } else {
            // remainder == 2 → treat as -1 + carry
            trits.push(Trit::Neg);
            val = (val + 1) / 3;
        }
    }
    trits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trit_conversions() {
        assert_eq!(Trit::Neg.to_i8(), -1);
        assert_eq!(Trit::Zero.to_i8(), 0);
        assert_eq!(Trit::Pos.to_i8(), 1);
        assert_eq!(Trit::from_i8(-1), Some(Trit::Neg));
        assert_eq!(Trit::from_i8(2), None);
    }

    #[test]
    fn test_trit_not() {
        assert_eq!(Trit::Neg.ternary_not(), Trit::Pos);
        assert_eq!(Trit::Pos.ternary_not(), Trit::Neg);
        assert_eq!(Trit::Zero.ternary_not(), Trit::Zero);
    }

    #[test]
    fn test_trit_and() {
        assert_eq!(Trit::Neg.ternary_and(Trit::Pos), Trit::Neg);
        assert_eq!(Trit::Pos.ternary_and(Trit::Pos), Trit::Pos);
        assert_eq!(Trit::Zero.ternary_and(Trit::Pos), Trit::Zero);
    }

    #[test]
    fn test_trit_or() {
        assert_eq!(Trit::Neg.ternary_or(Trit::Pos), Trit::Pos);
        assert_eq!(Trit::Neg.ternary_or(Trit::Neg), Trit::Neg);
        assert_eq!(Trit::Zero.ternary_or(Trit::Pos), Trit::Pos);
    }

    #[test]
    fn test_trit_consensus() {
        assert_eq!(Trit::Pos.consensus(Trit::Pos), Trit::Pos);
        assert_eq!(Trit::Neg.consensus(Trit::Neg), Trit::Neg);
        assert_eq!(Trit::Pos.consensus(Trit::Neg), Trit::Zero);
        assert_eq!(Trit::Zero.consensus(Trit::Pos), Trit::Zero);
    }

    #[test]
    fn test_tryte_zero() {
        let t = Tryte::zero();
        assert_eq!(t.to_i32(), 0);
    }

    #[test]
    fn test_tryte_roundtrip() {
        for val in &[-100, -1, 0, 1, 42, 100, 364] {
            let tryte = Tryte::from_i32(*val);
            assert_eq!(tryte.to_i32(), *val, "Failed for val={}", val);
        }
    }

    #[test]
    fn test_tryte_from_i32_bounds() {
        let max = Tryte::from_i32(364);
        assert_eq!(max.to_i32(), 364);
        let min = Tryte::from_i32(-364);
        assert_eq!(min.to_i32(), -364);
    }

    #[test]
    fn test_tryte_trit_access() {
        let t = Tryte::from_i32(5); // 5 = 1*1 + 1*3 + (-1)*9 is wrong. 5 = 2*3 -1 = ... balanced: 5 = (-1)*1 + 0*3 + 0*9 + 0*27 + ...
        // 5 in balanced ternary: 5 = 1*9 + (-1)*3 + (-1)*1 = 9-3-1=5 → trits: [-1,-1,1,0,0,0] = 5? No. Let's verify via to_i32.
        assert_eq!(t.to_i32(), 5);
    }

    #[test]
    fn test_register_load_read() {
        let mut reg = TernaryRegister::new("R0");
        let val = Tryte::from_i32(42);
        reg.load(&val);
        assert_eq!(reg.read().to_i32(), 42);
    }

    #[test]
    fn test_register_clear() {
        let mut reg = TernaryRegister::new("R1");
        reg.load(&Tryte::from_i32(99));
        reg.clear();
        assert_eq!(reg.read().to_i32(), 0);
    }

    #[test]
    fn test_memory_read_write() {
        let mut mem = TernaryMemory::new(16);
        let val = Tryte::from_i32(100);
        assert!(mem.write(5, val));
        let read = mem.read(5).unwrap();
        assert_eq!(read.to_i32(), 100);
    }

    #[test]
    fn test_memory_out_of_bounds() {
        let mut mem = TernaryMemory::new(4);
        assert!(!mem.write(10, Tryte::zero()));
        assert!(mem.read(10).is_none());
    }

    #[test]
    fn test_memory_clear_all() {
        let mut mem = TernaryMemory::new(4);
        mem.write(0, Tryte::from_i32(10));
        mem.write(1, Tryte::from_i32(20));
        mem.clear_all();
        assert_eq!(mem.read(0).unwrap().to_i32(), 0);
        assert_eq!(mem.read(1).unwrap().to_i32(), 0);
    }

    #[test]
    fn test_alu_add() {
        let mut alu = TernaryALU::new();
        let a = Tryte::from_i32(10);
        let b = Tryte::from_i32(20);
        let result = alu.add(&a, &b);
        assert_eq!(result.to_i32(), 30);
    }

    #[test]
    fn test_alu_add_negative() {
        let mut alu = TernaryALU::new();
        let a = Tryte::from_i32(50);
        let b = Tryte::from_i32(-30);
        let result = alu.add(&a, &b);
        assert_eq!(result.to_i32(), 20);
    }

    #[test]
    fn test_alu_subtract() {
        let mut alu = TernaryALU::new();
        let a = Tryte::from_i32(50);
        let b = Tryte::from_i32(20);
        let result = alu.subtract(&a, &b);
        assert_eq!(result.to_i32(), 30);
    }

    #[test]
    fn test_alu_multiply() {
        let mut alu = TernaryALU::new();
        let a = Tryte::from_i32(6);
        let b = Tryte::from_i32(7);
        let result = alu.multiply(&a, &b);
        assert_eq!(result.to_i32(), 42);
    }

    #[test]
    fn test_alu_negate() {
        let mut alu = TernaryALU::new();
        let a = Tryte::from_i32(10);
        let result = alu.negate(&a);
        assert_eq!(result.to_i32(), -10);
    }

    #[test]
    fn test_alu_compare() {
        let alu = TernaryALU::new();
        let a = Tryte::from_i32(10);
        let b = Tryte::from_i32(20);
        assert_eq!(alu.compare(&a, &b), Trit::Neg);
        assert_eq!(alu.compare(&b, &a), Trit::Pos);
        assert_eq!(alu.compare(&a, &a), Trit::Zero);
    }

    #[test]
    fn test_alu_shift_left() {
        let alu = TernaryALU::new();
        let a = Tryte::from_i32(5);
        let shifted = alu.shift_left(&a);
        assert_eq!(shifted.to_i32(), 15); // 5 * 3 = 15
    }

    #[test]
    fn test_alu_shift_right() {
        let alu = TernaryALU::new();
        let a = Tryte::from_i32(15);
        let shifted = alu.shift_right(&a);
        assert_eq!(shifted.to_i32(), 5); // 15 / 3 = 5
    }

    #[test]
    fn test_tryte_to_binary_roundtrip() {
        let original = Tryte::from_i32(42);
        let binary = tryte_to_binary(&original);
        assert_eq!(binary.len(), 12);
        let restored = binary_to_tryte(&binary).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_binary_to_tryte_invalid() {
        assert!(binary_to_tryte(&[0; 10]).is_none()); // wrong length
        assert!(binary_to_tryte(&[1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).is_none()); // 1,1 is invalid
    }

    #[test]
    fn test_balanced_ternary_conversions() {
        assert_eq!(balanced_ternary_to_decimal(&[Trit::Pos]), 1);
        assert_eq!(balanced_ternary_to_decimal(&[Trit::Neg]), -1);
        assert_eq!(balanced_ternary_to_decimal(&[Trit::Zero]), 0);
        assert_eq!(balanced_ternary_to_decimal(&[Trit::Pos, Trit::Pos]), 4); // 1 + 1*3
        assert_eq!(balanced_ternary_to_decimal(&[Trit::Neg, Trit::Pos]), 2); // -1 + 1*3
    }

    #[test]
    fn test_decimal_to_balanced_ternary() {
        assert_eq!(decimal_to_balanced_ternary(0), vec![Trit::Zero]);
        assert_eq!(decimal_to_balanced_ternary(1), vec![Trit::Pos]);
        assert_eq!(decimal_to_balanced_ternary(-1), vec![Trit::Neg]);
        assert_eq!(decimal_to_balanced_ternary(4), vec![Trit::Pos, Trit::Pos]);
        let t = decimal_to_balanced_ternary(5);
        assert_eq!(balanced_ternary_to_decimal(&t), 5);
    }

    #[test]
    fn test_decimal_roundtrip() {
        for val in &[-50, -1, 0, 1, 13, 42, 100] {
            let trits = decimal_to_balanced_ternary(*val);
            assert_eq!(balanced_ternary_to_decimal(&trits), *val);
        }
    }
}
