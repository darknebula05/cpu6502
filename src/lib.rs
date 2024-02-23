pub mod prelude {
    pub use emu6502_macros::*;
    pub fn foo() {
        println!("foo");
    }
}

pub trait AddressingMode {
    fn call(&self);
}
