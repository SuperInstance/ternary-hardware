# Future Integration: ternary-hardware

## Current State
Provides hardware abstraction for ternary operations: TernaryALU, TernaryMemory (tryte = 6 trits), TernaryRegister, ternary-to-binary conversion, and balanced ternary arithmetic circuits.

## Integration Opportunities

### With ternary-compiler-v2
The compiler generates ternary IR and needs a target. `ternary-hardware` defines the target architecture: ALU operations, register file, memory layout. The compiler's `Trybble` type maps directly to `TernaryRegister`. Together they form a complete ternary toolchain: source → IR → machine code for the TernaryALU.

### With ternary-esp32-firmware
ESP32 deployment of ternary agents requires hardware abstraction. `TernaryMemory` models the tryte-based memory that ternary-esp32-firmware implements on bare metal. `ternary-to-binary` conversion bridges the gap: ternary algorithms that run on the ternary abstraction, compiled to binary for the physical ESP32.

### With compiled-policy-c
Policy deployment on microcontrollers needs ternary hardware support. A `compiled_policy` that uses ternary decisions maps to `TernaryALU` operations. The C implementation in `compiled-policy-c` targets real hardware described by `ternary-hardware`.

## Potential in Mature Systems
In room-as-codespace, the hardware abstraction is the portability layer. Whether a room runs on a Codespace (x86-64, binary), a Jetson (ARM64 + CUDA, binary), or an ESP32 (bare metal, ternary-optimized), `ternary-hardware` provides the unified interface. The ternary-to-binary converter is the shim that makes ternary algorithms run on binary hardware.

## Cross-Pollination Ideas
- Ternary memory as a compressed representation: 6 trits per tryte gives 729 states vs 256 per byte
- Balanced ternary ALU as a natural fit for {-1, 0, +1} agent decisions on microcontrollers
- Ternary register file as the state representation for room-local agent memory on ESP32

## Dependencies for Next Steps
- ternary-compiler-v2 code generation targeting TernaryALU
- ternary-esp32-firmware using TernaryMemory layout
- Emulator/simulator for ternary hardware on binary platforms
