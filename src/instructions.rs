use log::trace;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(PartialEq, Eq)]
enum InstType {
    Store = 6,
    Add = 7,
    StoreMemAddr = 0xA,
    Draw = 0xD,
    SetDelayTimer = 0xF15,
}
impl InstType {
    fn from_u8(i: u8) -> InstType {
        match i {
            6 => InstType::Store,
            7 => InstType::Add,
            0xA => InstType::StoreMemAddr,
            0xD => InstType::Draw,
            _ => panic!("{}", format!("Unknown Instruction {}", i)),
        }
    }
}

pub trait Instruction {
    fn execute(&mut self) -> ();
    fn to_string(&self) -> String;
}

struct StoreInstruction {
    target: u8,
    val: u8,
    register_ref: Rc<RefCell<[u8; 16]>>,
}

impl<'a> StoreInstruction {
    fn store(&mut self) {
        let mut registers = self.register_ref.borrow_mut();
        registers[self.target as usize] = self.val;
    }
    fn to_string(&self) -> String {
        format!("Instr Store to Reg {} Val {}", self.target, self.val)
    }
}

impl Instruction for StoreInstruction {
    fn execute(&mut self) -> () {
        self.store();
    }
    fn to_string(&self) -> String {
        self.to_string()
    }
}

struct AddInstruction {
    target: u8,
    val: u8,
    register_ref: Rc<RefCell<[u8; 16]>>,
}

impl AddInstruction {
    fn add(&mut self) {
        let mut registers = self.register_ref.borrow_mut();
        registers[self.target as usize] += self.val;
    }
    fn to_string(&self) -> String {
        format!("Instr Add to Reg {} Val {}", self.target, self.val)
    }
}
impl Instruction for AddInstruction {
    fn execute(&mut self) -> () {
        self.add()
    }
    fn to_string(&self) -> String {
        self.to_string()
    }
}

struct StoreMemAddrInstruction {
    i_ref: Rc<RefCell<u16>>,
    val: u16,
}
impl StoreMemAddrInstruction {
    fn store_mem_addr(&mut self) {
        let mut mem_register = self.i_ref.borrow_mut();
        *mem_register = self.val;
    }
    fn to_string(&self) -> String {
        format!("Instr Store to MemAddr Val {}", self.val)
    }
}
impl Instruction for StoreMemAddrInstruction {
    fn execute(&mut self) -> () {
        self.store_mem_addr();
    }
    fn to_string(&self) -> String {
        self.to_string()
    }
}

struct DrawInstruction {
    sprite_bytes: u8,
    x: u8,
    y: u8,
    register_ref: Rc<RefCell<[u8; 16]>>,
    mem_register: Rc<RefCell<u16>>,
    screen: Rc<RefCell<[[bool; 63]; 31]>>,
    ram: Rc<RefCell<[u8; 4096]>>,
}

fn u8_to_8_bools(input: u8) -> [bool; 8] {
    let mut ret = [false; 8];
    for i in 0..8 {
        let extracted = input & (1 << i);
        ret[i] = extracted != 0;
    }
    ret.reverse();
    ret
}
impl DrawInstruction {
    fn draw(&mut self) {
        // TODO: wrap around display behavior and VF flag
        let ram: [u8; 4096] = *self.ram.borrow();
        let sprite_len: usize = self.sprite_bytes.into();
        let mem_pos: usize = self.mem_register.borrow().to_owned().into();
        let sprite = &ram[mem_pos..mem_pos + sprite_len];
        let mut screen = self.screen.borrow_mut();
        for elem in sprite {
            let mut y_shift: usize = 0;
            for pixel in u8_to_8_bools(*elem as u8) {
                // pixels are added to the screen with XOR
                screen[self.x as usize][self.y as usize + y_shift] ^= pixel;
                y_shift += 1;
            }
        }
    }
}

impl Instruction for DrawInstruction {
    fn execute(&mut self) -> () {
        self.draw()
    }
    fn to_string(&self) -> String {
        let mem_register = self.mem_register.borrow();
        format!(
            "Draw from pos x {}, y {}, {} many bytes from memory, starting from mempos {}",
            self.x, self.y, self.sprite_bytes, *mem_register,
        )
    }
}

struct SetTimer {
    timer: Rc<RefCell<u8>>,
    val: u8,
}

impl SetTimer {
    fn set(&mut self) {
        let mut timer = *self.timer.borrow_mut();
        timer = self.val;
    }
}
impl Instruction for SetTimer {
    fn execute(&mut self) -> () {
        self.set();
    }
    fn to_string(&self) -> String {
        format!("Setting val: {} to timer", self.val)
    }
}

pub fn instruction_parser<'a>(
    raw_data: (u8, u8),
    register_ref: Rc<RefCell<[u8; 16]>>,
    mem_register: Rc<RefCell<u16>>,
    screen: Rc<RefCell<[[bool; 63]; 31]>>,
    ram: Rc<RefCell<[u8; 4096]>>,
    timer: Rc<RefCell<u8>>,
) -> Box<dyn Instruction + 'a> {
    let first_nibble = (raw_data.0 & 0xF0) >> 4;
    trace!("First nibble with trace {:x}", first_nibble);
    let second_nibble = raw_data.0 & 0x0F;
    let third_nibble = (raw_data.1 & 0xF0) >> 4;
    let fourth_nibble = raw_data.1 & 0x0F;
    match (first_nibble, second_nibble, third_nibble, fourth_nibble) {
        (0x6, _, _, _) => Box::new(StoreInstruction {
            target: second_nibble,
            val: third_nibble + fourth_nibble,
            register_ref,
        }) as Box<dyn Instruction + 'a>,
        (0x07, _, _, _) => Box::new(AddInstruction {
            target: second_nibble,
            val: third_nibble + fourth_nibble,
            register_ref,
        }) as Box<dyn Instruction + 'a>,
        (0xA, _, _, _) => Box::new(StoreMemAddrInstruction {
            i_ref: mem_register,
            val: ((third_nibble << 4) + fourth_nibble).try_into().unwrap(),
        }),
        (0xD, _, _, _) => Box::new(DrawInstruction {
            x: second_nibble,
            y: third_nibble,
            sprite_bytes: fourth_nibble,
            register_ref,
            mem_register,
            screen,
            ram,
        }) as Box<dyn Instruction + 'a>,
        (0xF, _, 0x1, 0x5) => {
            let reg = register_ref.borrow();
            let val = reg[second_nibble as usize];
            return Box::new(SetTimer { val, timer }) as Box<dyn Instruction + 'a>;
        }
        (_, _, _, _) => panic!("IDK"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u8_to_bool_checks() {
        let t1 = 1;
        let res = u8_to_8_bools(t1);
        assert_eq!(res, [false, false, false, false, false, false, false, true]);

        let t2 = 3;
        let res = u8_to_8_bools(t2);
        assert_eq!(res, [false, false, false, false, false, false, true, true]);
        let t3 = 255;
        let res = u8_to_8_bools(t3);
        assert_eq!(res, [true, true, true, true, true, true, true, true]);
    }
}
