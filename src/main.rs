use cpu6502::*;

fn main() {
    let cpu = Cpu6502::default();
    println!("{cpu:?}");
    println!("{:?}", LOOKUP);
}
