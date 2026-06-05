# ternary-hardware

Hardware abstraction layer for balanced ternary computing — trits, trytes, registers, memory, and a full ALU operating on {-1, 0, +1}.

## Why This Exists

Binary computing has been the dominant paradigm for decades, but balanced ternary — where each digit carries three states: negative (-1), zero (0), and positive (+1) — has compelling theoretical advantages. A ternary digit (trit) carries more information per symbol than a bit (~1.58× the entropy), and balanced ternary arithmetic produces naturally symmetric results around zero. This crate provides the foundational building blocks for exploring ternary computing: from individual trits up through a complete ALU with ternary-to-binary conversion for interfacing with conventional hardware.

This crate is part of the **Negative Space Intelligence** ecosystem — a suite of crates that apply ternary {-1, 0, +1} logic across domains from quantum computing to economics.

## Core Concepts

- **Trit** — A balanced ternary digit: `Neg` (-1), `Zero` (0), or `Pos` (+1). Supports ternary NOT, AND, OR, and consensus operations.
- **Tryte** — A word of 6 trits, analogous to a byte. Range: -364 to +364 in balanced ternary. Convertible to/from `i32`.
- **TernaryRegister** — A named register holding a single tryte. Supports load, read, and clear.
- **TernaryMemory** — Addressable array of trytes. Random access read/write with bounds checking.
- **TernaryALU** — Arithmetic Logic Unit performing balanced ternary addition (with carry), subtraction, multiplication, negation, tritwise AND/OR/NOT, comparison (returning a trit), and trit shifts (multiply/divide by 3).
- **Ternary-to-Binary Conversion** — Encode trits as 2-bit pairs and convert trytes to/from binary representation.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-hardware = "0.1"
```

```rust
use ternary_hardware::*;

// Create trits and trytes
let t = Trit::Pos;                          // +1
let tryte = Tryte::from_i32(42);            // encode 42 as balanced ternary
assert_eq!(tryte.to_i32(), 42);

// Use the ALU
let mut alu = TernaryALU::new();
let a = Tryte::from_i32(10);
let b = Tryte::from_i32(20);
let sum = alu.add(&a, &b);
assert_eq!(sum.to_i32(), 30);

// Compare returns a trit: Neg (<), Zero (==), Pos (>)
assert_eq!(alu.compare(&a, &b), Trit::Neg);

// Shift operations multiply/divide by 3
let shifted = alu.shift_left(&Tryte::from_i32(5));
assert_eq!(shifted.to_i32(), 15); // 5 × 3

// Ternary-to-binary conversion
let binary = tryte_to_binary(&tryte);       // 12 bits
let restored = binary_to_tryte(&binary);    // round-trip
assert_eq!(restored.unwrap(), tryte);

// Convert arbitrary integers to balanced ternary representation
let trits = decimal_to_balanced_ternary(14);
assert_eq!(balanced_ternary_to_decimal(&trits), 14);
```

## API Overview

| Type / Function | Description |
|---|---|
| `Trit` | Balanced ternary digit enum (`Neg`, `Zero`, `Pos`) |
| `Tryte` | 6-trit word (-364 to +364) |
| `TernaryRegister` | Named register holding a tryte |
| `TernaryMemory` | Addressable tryte array |
| `TernaryALU` | Arithmetic Logic Unit for ternary math |
| `tryte_to_binary` / `binary_to_tryte` | Ternary ↔ binary encoding |
| `decimal_to_balanced_ternary` / `balanced_ternary_to_decimal` | Integer ↔ trit conversion |

### Key Trit operations
- `ternary_not()` — Swap Neg ↔ Pos, Zero stays
- `ternary_and(other)` — Min of two trits
- `ternary_or(other)` — Max of two trits
- `consensus(other)` — Returns matching value if equal, Zero otherwise

### Key ALU operations
- `add(a, b)` / `subtract(a, b)` / `multiply(a, b)` / `negate(a)`
- `and(a, b)` / `or(a, b)` / `not(a)` — Tritwise logical operations
- `compare(a, b)` — Returns `Neg`, `Zero`, or `Pos`
- `shift_left(a)` / `shift_right(a)` — Multiply/divide by 3

## How It Works

Balanced ternary represents numbers using digits {-1, 0, +1} with place values that are powers of 3. For example, the number 5 is represented as `+1·9 + (-1)·3 + (-1)·1`. This symmetric representation means negation is just flipping all digits — no two's complement needed.

The ALU's addition uses a full carry-propagation algorithm adapted for three-valued logic. Each trit position produces both a sum trit and a carry trit (which itself can be -1, 0, or +1). The carry cases differ from binary: for instance, a sum of 2 in a column resolves to a trit of -1 with a carry of +1. This makes ternary addition more compact per digit than binary.

The tryte uses 6 trits, giving a range of ±364 — close to what a 9-bit unsigned binary value covers, but with natural handling of signed arithmetic since the representation is inherently balanced around zero.

## Use Cases

1. **Ternary CPU simulation** — Build a balanced ternary processor simulator for research or education, using `TernaryALU`, `TernaryMemory`, and `TernaryRegister` as the core components.

2. **Ternary-to-binary interface design** — When building hybrid systems that need to bridge ternary and binary hardware, use `tryte_to_binary` / `binary_to_tryte` for the encoding layer.

3. **Educational tools** — Teach alternative number systems and computer architecture concepts. Balanced ternary is historically significant (Setun computer, 1958) and provides a rich pedagogical contrast to binary.

4. **Signal processing research** — The natural {-1, 0, +1} representation maps directly to signed discrete signals, making ternary arithmetic useful for ternary-valued filter design.

## Ecosystem

| Crate | Relationship |
|---|---|
| `ternary-logic` | Logical operations on ternary values (Kleene, Łukasiewicz, etc.) |
| `ternary-quantum` | Quantum (qutrit) extensions of the trit concept |
| `ternary-cell` | Cellular computing using ternary state machines |
| `ternary-network` | Ternary-weighted graph algorithms |
| `ternary-attention` | Ternary-valued attention mechanisms |

## Known Limitations

- **Silent overflow on addition.** `TernaryALU::add` discards carry out of the 6th trit. Values exceeding ±364 wrap silently.
- **Multiply converts to `i32`.** `multiply` performs the product in integer space and converts back — it is not a true ternary multiplication circuit.
- **No division operation.** The ALU supports add, subtract, multiply, negate, and shifts, but cannot divide.
- **Ternary-to-binary encoding wastes a code point.** The 2-bit encoding (Neg=00, Zero=01, Pos=10) leaves `11` unused and doesn't align with any standard ternary encoding scheme.
- **No bounds-checked mutable read.** `TernaryMemory` provides `read(&self, addr)` with bounds checking but no equivalent `read_mut` that returns a mutable reference safely.

## See Also

- **ternary-circuit** — Circuit and logic design with ternary values
- **ternary-esp32-firmware** — ESP32 firmware for ternary hardware
- **ternary-ring** — Ring arithmetic for ternary algebra
- **ternary-compiler-v2** — Next-gen compiler for ternary instructions
- **ternary-quantum** — Quantum computing with qutrits

## License

MIT
