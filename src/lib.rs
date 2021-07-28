mod chip8;

use chip8::Cpu;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Chip8Cpu {
    cpu: Cpu,
}

#[wasm_bindgen]
impl Chip8Cpu {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { cpu: Cpu::new() }
    }

    pub fn load(&mut self, js_rom: Vec<u8>) {
        let mut rom = [0u8; 3584];
        for (addr, &byte) in js_rom.iter().enumerate() {
            rom[addr] = byte;
        }
        self.cpu.load(&rom)
    }

    pub fn emulate_cycle(&mut self) {
        self.cpu.emulate_cycle()
    }

    pub fn update_keypad(&mut self, keycode: char, state: bool) {
        self.cpu.update_keypad(keycode, state)
    }

    pub fn frame_buffer(&self) -> Vec<u8> {
        self.cpu
            .frame_buffer
            .iter()
            .map(|s| s.iter().map(|&c| c.clone()).collect::<Vec<u8>>())
            .flatten()
            .collect()
    }

    pub fn frame_buffer_updated(&self) -> bool {
        self.cpu.frame_buffer_updated
    }
}
