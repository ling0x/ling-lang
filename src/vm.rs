#[derive(Clone, Debug)]
pub enum Op {
    Push(u8),
    Add,
    Sub,
    Mul,
    Print,
    Dup,
    Halt,
}

pub struct Vm {
    stack: Vec<i32>,
}

impl Vm {
    pub fn new() -> Self {
        Vm { stack: Vec::new() }
    }

    pub fn run(&mut self, ops: &[Op]) {
        for op in ops {
            match op {
                Op::Push(n) => self.stack.push(*n as i32),
                Op::Add => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    self.stack.push(a + b);
                }
                Op::Sub => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    self.stack.push(a - b);
                }
                Op::Mul => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    self.stack.push(a * b);
                }
                Op::Dup => {
                    let v = self.stack.last().copied().unwrap_or(0);
                    self.stack.push(v);
                }
                Op::Print => {
                    let v = self.stack.pop().unwrap_or(0);
                    println!("{}", v);
                }
                Op::Halt => break,
            }
        }
    }
}
