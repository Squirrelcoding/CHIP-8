mod lib;

use std::io::prelude::*;

fn main() {
    let mut bytes: Vec<u8> = Vec::new();

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open("test_opcode.ch8")
        .unwrap();
    file.read_to_end(&mut bytes).unwrap();

    println!("{}", bytes.len());

    let mut cpu = lib::cpu::CPU::new_with_memory(&bytes);

    // cpu.setannn(0x050);

    // cpu.set6xnn(0, 65);

    // cpu.set6xnn(1, 5);

    // cpu.draw(0, 1, 5);

    // println!("################################################################");
    cpu.run();
}
