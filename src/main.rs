use log::{error, trace};
use rand::Rng;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
extern crate env_logger;

fn u8_to_8_bools(input: u8) -> [bool; 8] {
    let mut ret = [false; 8];
    for i in 0..8 {
        let extracted = input & (1 << i);
        ret[i] = extracted != 0;
    }
    ret.reverse();
    ret
}

struct Chip8 {
    registers: [u8; 16],
    mem_addr: u16, // memory address register
    program_counter: u16,
    stack_pointer: u8,
    stack: [u16; 16],
    screen: [[bool; 63]; 31],
    ram: [u8; 4096],
    timer: u8,
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            registers: [0; 16],
            mem_addr: 0,
            program_counter: 0,
            stack_pointer: 0,
            stack: [0; 16],
            screen: [[false; 63]; 31],
            ram: [0; 4096],
            timer: 0,
        }
    }
    pub fn execute_instruction(&mut self, byte1: u8, byte2: u8) {
        let first_nibble = (byte1 & 0xF0) >> 4;
        let second_nibble = byte1 & 0x0F;
        let third_nibble = (byte2 & 0xF0) >> 4;
        let fourth_nibble = byte2 & 0x0F;
        trace!(
            "Executing instruction: {:x?} {:x?} {:x?} {:x?}",
            first_nibble,
            second_nibble,
            third_nibble,
            fourth_nibble
        );
        self.program_counter += 2;
        match (first_nibble, second_nibble, third_nibble, fourth_nibble) {
            (0, 0, 0xE, 0) => {
                trace!("Clearing screen");
                self.screen = [[false; 63]; 31];
            }
            (0, 0, 0xE, 0xE) => {
                trace!("Returning from subroutine");
                self.program_counter = self.stack[0];
                self.stack_pointer -= 1;
            }
            (1, _, _, _) => {
                let val = ((second_nibble as u16) << 8)
                    + (third_nibble << 4) as u16
                    + fourth_nibble as u16;
                trace!("Jumping to address {}", val);
                self.program_counter = val;
            }
            (2, _, _, _) => {
                self.stack_pointer += 1;
                self.stack[0] = self.program_counter;
                let val = ((second_nibble as u16) << 8)
                    + (third_nibble << 4) as u16
                    + fourth_nibble as u16;
                self.program_counter = val;
                trace!("Call subroutine ")
            }
            (3, _, _, _) => {
                let val = (third_nibble << 4) + fourth_nibble;
                if self.registers[second_nibble as usize] == val {
                    trace!(
                        "Register {}: {} is equal to {}, skipping next instruction",
                        second_nibble,
                        self.registers[second_nibble as usize],
                        val
                    );
                    self.program_counter += 2
                } else {
                    trace!(
                        "Register {}: {} is different to {}, not skipping next instruction",
                        second_nibble,
                        self.registers[second_nibble as usize],
                        val
                    );
                }
            }
            (4, _, _, _) => {
                trace!("Skip next instruction if register is not equal to value");
                //Jump if register not equal to value
                let val = (third_nibble << 4) + fourth_nibble;
                if self.registers[second_nibble as usize] != val {
                    trace!(
                        "Register {}: {} is not equal to {}, skipping next instruction",
                        second_nibble,
                        self.registers[second_nibble as usize],
                        val
                    );
                    self.program_counter += 2
                } else {
                    trace!(
                        "Register {}: {} is equal to {}, not skipping next instruction",
                        second_nibble,
                        self.registers[second_nibble as usize],
                        val
                    );
                }
            }
            (5, _, _, 0) => {
                trace!("Skip next instruction if registers are equal");
                // jump if two registers are equal
                let val1 = self.registers[second_nibble as usize];
                let val2 = self.registers[third_nibble as usize];
                if val1 == val2 {
                    trace!("Registers are equal, skipping next instruction");
                    self.program_counter += 2
                } else {
                    trace!(
                        "Registers {} and {} are not equal, not skipping next instruction",
                        second_nibble,
                        third_nibble
                    );
                }
            }
            (0x6, _, _, _) => {
                // load val into register
                let target = second_nibble as usize;
                let val = (third_nibble << 4) + fourth_nibble;
                self.registers[target] = val;
                trace!("Loading value {} into register {}", val, target);
            }
            (0x07, _, _, _) => {
                // add register + val
                let target = second_nibble as usize;
                let val: u16 = (third_nibble << 4) as u16 + fourth_nibble as u16;
                self.registers[target] = ((self.registers[target] as u16) + val) as u8;
                trace!(
                    "Adding value {} to register {}, new value: {}",
                    val,
                    target,
                    self.registers[target]
                );
            }
            (0x08, _, _, 0) => {
                // set reg x to regy
                trace!(
                    "Setting register x {}:{} to register y {}:{}",
                    second_nibble,
                    self.registers[second_nibble as usize],
                    third_nibble,
                    self.registers[third_nibble as usize]
                );
                let xpos = second_nibble as usize;
                let ypos = third_nibble as usize;
                self.registers[xpos] = self.registers[ypos];
            }
            (0x08, _, _, 1) => {
                let val1 = self.registers[second_nibble as usize];
                let val2 = self.registers[third_nibble as usize];
                self.registers[second_nibble as usize] = val1 | val2;
                trace!(
                    "Bitwise or between registers {} {}",
                    second_nibble,
                    third_nibble
                );
            }
            (0x08, _, _, 2) => {
                let val1 = self.registers[second_nibble as usize];
                let val2 = self.registers[third_nibble as usize];
                self.registers[second_nibble as usize] = val1 & val2;
                trace!(
                    "Bitwise and between registers {} {}",
                    second_nibble,
                    third_nibble
                );
            }
            (0x08, _, _, 3) => {
                let val1 = self.registers[second_nibble as usize];
                let val2 = self.registers[third_nibble as usize];
                self.registers[second_nibble as usize] = val1 ^ val2;
                trace!(
                    "Bitwise XOR between registers {} {}",
                    second_nibble,
                    third_nibble
                );
            }
            (0x08, _, _, 4) => {
                let xpos = second_nibble as usize;
                let ypos = third_nibble as usize;
                let res: u16 = self.registers[xpos] as u16 + self.registers[ypos] as u16;
                if res > 0xFF {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[xpos] = res as u8;
                trace!("Adding registers {} and {}, result: {}", xpos, ypos, res);
            }
            (0x08, _, _, 5) => {
                let xpos = second_nibble as usize;
                let ypos = third_nibble as usize;
                let vx = self.registers[xpos];
                let vy = self.registers[ypos];
                if vx > vy {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                let res: u16 =
                    (self.registers[xpos] as u16).wrapping_sub(self.registers[ypos] as u16);
                self.registers[xpos] = res as u8;
                trace!(
                    "Substracting registers {} and {}, result: {}",
                    xpos,
                    ypos,
                    res
                );
            }
            (0x08, _, _, 0xE) => {
                let val = self.registers[second_nibble as usize];
                // carry bit
                self.registers[0xF] = 0b10000000 & val;
                self.registers[second_nibble as usize] = (val as u16 * 2) as u8;
                trace!("Left shift on register {}", second_nibble);
            }
            (0x08, _, _, 6) => {
                let val = self.registers[second_nibble as usize];
                // carry bit
                self.registers[0xF] = 0b00000001 & val;
                self.registers[second_nibble as usize] = val >> 1;
                trace!("Right shift on register {}", second_nibble);
            }
            (0x09, _, _, 0) => {
                trace!("Skip next instruction if registers are different");
                // jump if two registers are equal
                let val1 = self.registers[second_nibble as usize];
                let val2 = self.registers[third_nibble as usize];
                if val1 != val2 {
                    trace!("Registers are different, skipping next instruction");
                    self.program_counter += 2
                } else {
                    trace!(
                        "Registers {} and {} are equal, not skipping next instruction",
                        second_nibble,
                        third_nibble
                    );
                }
            }
            (0xA, _, _, _) => {
                // set memory pointer
                let val: u32 = ((third_nibble << 4) + fourth_nibble).try_into().unwrap();
                let val = (val + ((second_nibble as u32) << 8)).try_into().unwrap();
                self.mem_addr = val;
                trace!("Setting memory pointer to {}", val);
            }
            (0xC, _, _, _) => {
                let random_num: u8 = rand::random();
                let val: u8 = (third_nibble << 4) + fourth_nibble;
                self.registers[second_nibble as usize] = random_num & val;
                trace!("Setting random number to reg {} ", second_nibble);
            }
            (0xD, _, _, _) => {
                //draw
                let y: usize = second_nibble.into();
                let x: usize = third_nibble.into();
                let mut vx: usize = (self.registers[x] % 31) as usize;
                let mut vy: usize = (self.registers[y] % 63) as usize;
                let sprite_len: usize = fourth_nibble.into();
                let mem_pos: usize = self.mem_addr.into();
                let sprite = &self.ram[mem_pos..mem_pos + sprite_len];
                trace!(
                    "Drawing sprite at x: {}, y: {}, len: {}",
                    vx,
                    vy,
                    sprite_len
                );
                let mut erased = 0;
                for elem in sprite {
                    for pixel in u8_to_8_bools(*elem as u8) {
                        // pixels are added to the screen with XOR
                        if self.screen[vx as usize][vy] == true {
                            if pixel == true {
                                erased = 1
                            }
                        }
                        self.screen[vx as usize][vy] ^= pixel;
                        vy = ((vy + 1) % 63) as usize;
                    }
                    vx = ((vx + 1) % 31) as usize;
                    vy = (self.registers[y] % 63) as usize;
                }
                self.registers[0xF] = erased;
            }
            (0xF, _, 0x1, 0x5) => {
                trace!("Setting delay timer");
                let val = self.registers[second_nibble as usize];
                self.timer = val;
            }
            (0xF, _, 0x1, 0xE) => {
                self.mem_addr = self.mem_addr + self.registers[second_nibble as usize] as u16;
                trace!("Incrementing I with register {}", second_nibble);
            }
            (0xF, _, 0x3, 0x3) => {
                let b = self.registers[second_nibble as usize] / 100;
                let c = (self.registers[second_nibble as usize] % 100) / 10;
                let d = (self.registers[second_nibble as usize] % 100) % 10;
                self.ram[self.mem_addr as usize] = b;
                self.ram[self.mem_addr as usize + 1] = c;
                self.ram[self.mem_addr as usize + 2] = d;
                trace!(
                    "Converting reg {} into BCD and storing it into [i..i+3]",
                    second_nibble
                );
            }
            (0xF, _, 0x5, 0x5) => {
                let mut pointer = self.mem_addr as usize;
                let mut curr_reg = 0;
                let lim = second_nibble;
                for elem in self.registers {
                    self.ram[pointer] = elem;
                    pointer += 1;
                    curr_reg += 1;
                    if curr_reg > lim {
                        break;
                    }
                }
                trace!(
                    "All registers from 0 to {} written in mem pos {}",
                    second_nibble,
                    self.mem_addr
                );
            }
            (0xF, _, 0x6, 0x05) => {
                for i in 0..second_nibble + 1 {
                    let j = i as usize;
                    self.registers[j] = self.ram[self.mem_addr as usize + j]
                }
                trace!("Reading all registers from 0 to {} from ram", second_nibble);
            }
            (_, _, _, _) => {
                error!(
                    "Unknown instruction {:x?} {:x?} {:x?} {:x?}",
                    first_nibble, second_nibble, third_nibble, fourth_nibble
                );
                panic!("Unknown instruction");
            }
        }
    }

    pub fn load_instructions(&mut self, path: String) -> Result<(), std::io::Error> {
        let instructions: Vec<u8> = fs::read(path).unwrap();
        let mut i = 0;
        while i < instructions.len() - 1 {
            self.ram[0x200 + i] = instructions[i];
            i = i + 1;
        }
        self.program_counter = 0x200;
        Ok(())
    }
    pub fn state_to_string(&self) -> String {
        format!("Regs: {:x?}", self.registers)
    }
    pub fn print_display(&self) {
        // println!("{:?}", self.screen);
        println!("-----------------------------------------------------------------");
        for row in self.screen {
            print!("|");
            for column in row {
                if column {
                    print!("#")
                } else {
                    print!(" ")
                }
            }
            print!("|");
            println!("");
        }
        println!("-----------------------------------------------------------------");
    }
}

fn main() {
    env_logger::init();
    let mut chip8 = Chip8::new();
    // read parameter from command line
    let args: Vec<String> = std::env::args().collect();
    let rom = &args[1];
    chip8.print_display();
    chip8.load_instructions(rom.to_string()).unwrap();
    loop {
        trace!("Executing instruction at {}", chip8.program_counter);
        let byte1 = chip8.ram[chip8.program_counter as usize];
        let byte2 = chip8.ram[(chip8.program_counter + 1) as usize];
        chip8.execute_instruction(byte1, byte2);
        chip8.print_display();
        // sleep
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
