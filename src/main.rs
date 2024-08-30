use chip8_emu::instructions::{instruction_parser, Instruction};
use log::trace;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
extern crate env_logger;

struct Chip8 {
    registers: Rc<RefCell<[u8; 16]>>,
    mem_addr: Rc<RefCell<u16>>, // memory address register
    program_counter: Rc<RefCell<u16>>,
    stack_pointer: u8,
    stack: [u16; 16],
    screen: Rc<RefCell<[[bool; 63]; 31]>>,
    program: Vec<Box<dyn Instruction>>,
    ram: Rc<RefCell<[u8; 4096]>>,
    timer: Rc<RefCell<u8>>,
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            registers: Rc::new(RefCell::new([0; 16])),
            mem_addr: Rc::new(RefCell::new(0)),
            program_counter: Rc::new(RefCell::new(0)),
            stack_pointer: 0,
            stack: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            screen: Rc::new(RefCell::new([[false; 63]; 31])),
            program: vec![],
            ram: Rc::new(RefCell::new([0; 4096])),
            timer: Rc::new(RefCell::new(0)),
        }
    }
    pub fn load_instructions(&self, path: String) -> Result<Vec<(u8, u8)>, std::io::Error> {
        let instructions: Vec<u8> = fs::read(path).unwrap();
        let mut instruction_tuples: Vec<(u8, u8)> = vec![];
        let mut i = 0;
        while i < instructions.len() - 1 {
            instruction_tuples.push((instructions[i], instructions[i + 1]));
            i = i + 2;
        }
        Ok(instruction_tuples)
    }
    pub fn state_to_string(&self) -> String {
        format!("Regs: {:x?}", self.registers)
    }
    pub fn print_display(&self) {
        let screen = *self.screen.borrow();
        println!("{:?}", self.screen);
        println!("-----------------------------------------------------------------");
        for row in screen {
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
    {
        // same as dropping the mutref
        let mut test = chip8.screen.borrow_mut();
        test[30][58] = true;
        test[0][0] = true;
    }
    chip8.print_display();
    let raw_instructions = chip8
        .load_instructions("./roms/zero.ch8".to_string())
        .unwrap();
    for i in 0..raw_instructions.len() {
        chip8.program.push(instruction_parser(
            raw_instructions[i],
            chip8.registers.clone(),
            chip8.mem_addr.clone(),
            chip8.screen.clone(),
            chip8.ram.clone(),
            chip8.timer.clone(),
            chip8.program_counter.clone(),
        ));
        println!("{}", chip8.program.last().unwrap().to_string());
    }
    chip8.print_display();
}
