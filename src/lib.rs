#![warn(missing_docs)]
//! Solutions to the [Synacor Challenge](https://challenge.synacor.com/).

use std::io::Read;

type Arg = u16;

#[derive(Debug)]
enum Op {
    Halt,
    Set(Arg, Arg),
    Push(Arg),
    Pop(Arg),
    Eq(Arg, Arg, Arg),
    Gt(Arg, Arg, Arg),
    Jmp(Arg),
    Jt(Arg, Arg),
    Jf(Arg, Arg),
    Add(Arg, Arg, Arg),
    Mult(Arg, Arg, Arg),
    Mod(Arg, Arg, Arg),
    And(Arg, Arg, Arg),
    Or(Arg, Arg, Arg),
    Not(Arg, Arg),
    Rmem(Arg, Arg),
    Wmem(Arg, Arg),
    Call(Arg),
    Ret,
    Out(Arg),
    In(Arg),
    Noop,
    Invalid(Arg),
}

/// Represents a virtual machine used in the Synacor Challenge.
#[derive(Debug)]
pub struct Machine {
    memory: Vec<u16>,
    registers: Vec<u16>,
    stack: Vec<u16>,
    ip: u16,
}

impl Machine {
    /// Constructs a new, pristine `Machine`.
    pub fn new() -> Machine {
        Machine {
            memory: vec![0; 32768],
            registers: vec![0; 8],
            stack: vec![],
            ip: 0,
        }
    }

    /// Loads a program from the filesystem into memory.
    ///
    /// If successful, this function will return the total number of
    /// bytes read. Otherwise it returns an `std::io::Error`.
    pub fn load(&mut self, path: &str) -> std::io::Result<usize> {
        use std::io::prelude::*;
        use std::fs::File;

        let mut file = try!(File::open(path));
        let mut buf = vec![];
        let read = try!(file.read_to_end(&mut buf));
        let mut i = 0;
        while i < buf.len() - 1 {
            self.memory[i / 2] = (buf[i] as u16) | ((buf[i + 1] as u16) << 8);
            i += 2;
        }

        Ok(read)
    }

    /// Reads from memory at the IP and increment it.
    fn next(&mut self) -> u16 {
        let ip = self.ip as usize;
        let ret = self.memory[ip];
        self.ip = (ip + 1) as u16;
        ret
    }

    fn get_register(&self, reg: Arg) -> u16 {
        self.registers[(reg - 32768) as usize]
    }

    fn set_register(&mut self, reg: Arg, val: u16) {
        self.registers[(reg - 32768) as usize] = val;
    }

    /// Decodes an instruction argument, fetching from a register if necessary.
    fn value(&self, arg: Arg) -> u16 {
        if arg <= 32767 {
            arg
        } else if arg <= 32775 {
            self.get_register(arg)
        } else {
            panic!("Invalid argument")
        }
    }

    /// Decodes the next instruction, incrementing the IP as appropriate.
    fn decode(&mut self) -> Op {
        match self.next() {
            0 => Op::Halt,
            1 => Op::Set(self.next(), self.next()),
            2 => Op::Push(self.next()),
            3 => Op::Pop(self.next()),
            4 => Op::Eq(self.next(), self.next(), self.next()),
            5 => Op::Gt(self.next(), self.next(), self.next()),
            6 => Op::Jmp(self.next()),
            7 => Op::Jt(self.next(), self.next()),
            8 => Op::Jf(self.next(), self.next()),
            9 => Op::Add(self.next(), self.next(), self.next()),
            10 => Op::Mult(self.next(), self.next(), self.next()),
            11 => Op::Mod(self.next(), self.next(), self.next()),
            12 => Op::And(self.next(), self.next(), self.next()),
            13 => Op::Or(self.next(), self.next(), self.next()),
            14 => Op::Not(self.next(), self.next()),
            15 => Op::Rmem(self.next(), self.next()),
            16 => Op::Wmem(self.next(), self.next()),
            17 => Op::Call(self.next()),
            18 => Op::Ret,
            19 => Op::Out(self.next()),
            20 => Op::In(self.next()),
            21 => Op::Noop,
            inv => Op::Invalid(inv),
        }
    }


    /// Adds two values modulo 32768.
    fn add(&self, a: u16, b: u16) -> u16 {
        let res = ((a as u32) + (b as u32)) % 32768;
        res as u16
    }

    /// Multiplies two values modulo 32768.
    fn mult(&self, a: u16, b: u16) -> u16 {
        let res = ((a as u32) * (b as u32)) % 32768;
        res as u16
    }

    /// Decodes and runs the next instruction.
    ///
    /// Returns `false` if a `HALT` or invalid instruction was encountered,
    /// true otherwise.
    ///
    /// # Panics
    ///
    /// Panics on stack underflow, an invalid instruction argument,
    /// or an EOF on stdin.
    pub fn tick(&mut self) -> bool {
        match self.decode() {
            Op::Halt => {
                println!("Got HALT, stopping.");
                return false;
            }
            Op::Set(a, b) => {
                let val = self.value(b);
                self.set_register(a, val);
            }
            Op::Push(a) => {
                let val = self.value(a);
                self.stack.push(val);
            }
            Op::Pop(a) => {
                let top = self.stack.pop().expect("Stack underflow");
                self.set_register(a, top);
            }
            Op::Eq(a, b, c) => {
                if self.value(b) == self.value(c) {
                    self.set_register(a, 1);
                } else {
                    self.set_register(a, 0);
                }
            }
            Op::Gt(a, b, c) => {
                if self.value(b) > self.value(c) {
                    self.set_register(a, 1);
                } else {
                    self.set_register(a, 0);
                }
            }
            Op::Jmp(a) => {
                self.ip = self.value(a);
            }
            Op::Jt(a, b) => {
                if self.value(a) != 0 {
                    self.ip = self.value(b);
                }
            }
            Op::Jf(a, b) => {
                if self.value(a) == 0 {
                    self.ip = self.value(b);
                }
            }
            Op::Add(a, b, c) => {
                let sum = self.add(self.value(b), self.value(c));
                self.set_register(a, sum);
            }
            Op::Mult(a, b, c) => {
                let res = self.mult(self.value(b), self.value(c));
                self.set_register(a, res);
            }
            Op::Mod(a, b, c) => {
                let res = self.value(b) % self.value(c);
                self.set_register(a, res);
            }
            Op::And(a, b, c) => {
                let and = self.value(b) & self.value(c);
                self.set_register(a, and);
            }
            Op::Or(a, b, c) => {
                let or = self.value(b) | self.value(c);
                self.set_register(a, or);
            }
            Op::Not(a, b) => {
                let not = (!self.value(b)) & 32767;
                self.set_register(a, not);
            }
            Op::Rmem(a, b) => {
                let val = self.memory[self.value(b) as usize];
                self.set_register(a, val);
            }
            Op::Wmem(a, b) => {
                let val = self.value(b);
                let dest = self.value(a);
                self.memory[dest as usize] = val;
            }
            Op::Call(a) => {
                self.stack.push(self.ip);
                self.ip = self.value(a);
            }
            Op::Ret => {
                match self.stack.pop() {
                    None => {
                        println!("Empty stack on RET. Halting.");
                        return false;
                    }
                    Some(addr) => {
                        self.ip = addr;
                    }
                }
            }
            Op::Out(a) => {
                print!("{}", (self.value(a) as u8) as char);
            }
            Op::In(a) => {
                let c: u8 = std::io::stdin()
                                .bytes()
                                .nth(0)
                                .expect("EOF")
                                .expect("EOF");
                self.set_register(a, c as u16);
            }
            Op::Noop => {}
            invalid => {
                println!("Not handling: {:?}", invalid);
                return false;
            }
        }

        true
    }

    /// Runs the machine until HALT or an invalid instruction.
    pub fn run(&mut self) {
        while self.tick() {
        }
    }
}
