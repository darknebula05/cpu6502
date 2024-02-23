use emu6502::prelude::*;

fn main() {
    foo();
}

#[derive(AddressingMode)]
enum Modes {
    Absolute,
    AbsoluteIndexedIndirect,
    AbsoluteIndexedX,
    AbsoluteIndexedY,
    AbsoluteIndirect,
    Accumulator,
    Immediate,
    Implied,
    ProgramCounterRelative,
    Stack,
    ZeroPage,
    ZeroPageIndexedIndirect,
    ZeroPageIndexedX,
    ZeroPageIndexedY,
    ZeroPageIndirect,
    ZeroPageIndirectIndexedY,
}
