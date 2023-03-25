use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
};

#[derive(Clone, Copy, Debug)]
pub enum Value {
    Lit(f64),
    Reg(Reg),
}

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Push(Value),
    Pop(Reg),
    Add(Reg, Value),
    Sub(Reg, Value),
    Mul(Reg, Value),
    Div(Reg, Value),
    Mov(Reg, Value),
    Cmp(Value, Value),
    Mark(&'static str),
    Goto(&'static str),
    GotoEq(&'static str, Value),
}

#[derive(Clone, Copy, Debug)]
pub enum Reg {
    AX,
    BX,
    CX,
    Cmp,
    OpPtr,
}

#[derive(Default, Debug)]
pub struct Regs {
    ax: f64,
    bx: f64,
    cx: f64,
    cmp: f64,
    opptr: usize,
}

impl Regs {
    fn resolve(&self, reg: Reg) -> &f64 {
        match reg {
            Reg::AX => &self.ax,
            Reg::BX => &self.bx,
            Reg::CX => &self.cx,
            Reg::Cmp => &self.cmp,
            Reg::OpPtr => panic!("ты шо дурак"),
        }
    }

    fn resolve_mut(&mut self, reg: Reg) -> &mut f64 {
        match reg {
            Reg::AX => &mut self.ax,
            Reg::BX => &mut self.bx,
            Reg::CX => &mut self.cx,
            Reg::Cmp => &mut self.cmp,
            Reg::OpPtr => panic!("ты шо дурак"),
        }
    }
}

#[derive(Default, Debug)]
pub struct VM {
    code: VecDeque<Op>,
    stack: VecDeque<f64>,
    stack_size: Option<usize>,
    marks: HashMap<&'static str, usize>,
    regs: Regs,
}

impl VM {
    pub fn with_sized_stack(stack_size: usize) -> Self {
        Self {
            stack_size: Some(stack_size),
            ..Default::default()
        }
    }

    pub fn load<I: IntoIterator<Item = Op>>(&mut self, code: I) {
        self.code.extend(code);
    }

    pub fn exec(&mut self) -> VMResult {
        loop {
            let op = if let Some(val) = self.code.get(self.regs.opptr).cloned() {
                val
            } else {
                return Ok(());
            };
            match op {
                Op::Push(val) => {
                    if let Some(stack_size) = self.stack_size {
                        if self.stack.len() == stack_size {
                            Err(ExecutionError::StackOverflow)?;
                        }
                    }
                    self.stack.push_back(self.retrieve_value(val));
                }
                Op::Pop(reg) => {
                    if let Some(val) = self.stack.pop_back() {
                        *self.regs.resolve_mut(reg) = val;
                    } else {
                        Err(ExecutionError::EmptyStack)?
                    }
                }
                Op::Add(reg, val) => *self.regs.resolve_mut(reg) += self.retrieve_value(val),
                Op::Sub(reg, val) => *self.regs.resolve_mut(reg) -= self.retrieve_value(val),
                Op::Mul(reg, val) => *self.regs.resolve_mut(reg) *= self.retrieve_value(val),
                Op::Div(reg, val) => {
                    let x = self.retrieve_value(val);
                    if x == 0. {
                        Err(ExecutionError::ZeroDivisionError)?;
                    }
                    *self.regs.resolve_mut(reg) /= x;
                }
                Op::Mov(reg, val) => *self.regs.resolve_mut(reg) = self.retrieve_value(val),
                Op::Cmp(val1, val2) => {
                    let ord = self
                        .retrieve_value(val1)
                        .total_cmp(&self.retrieve_value(val2));
                    self.regs.cmp = match ord {
                        Ordering::Less => -1.,
                        Ordering::Equal => 0.,
                        Ordering::Greater => 1.,
                    }
                }
                Op::Mark(id) => drop(self.marks.insert(id, self.regs.opptr)),
                Op::Goto(id) => self.goto(id)?,
                Op::GotoEq(id, val) => {
                    if self.retrieve_value(val) == self.regs.cmp {
                        self.goto(id)?
                    }
                }
            }
            self.regs.opptr += 1;
        }
    }

    pub fn regs(&self) -> &Regs {
        &self.regs
    }

    pub fn stack(&self) -> &VecDeque<f64> {
        &self.stack
    }

    fn goto(&mut self, id: &'static str) -> VMResult {
        if let Some(index) = self.marks.get(id).cloned() {
            self.regs.opptr = index;
            Ok(())
        } else {
            Err(ExecutionError::NoSuchMark)?
        }
    }

    fn retrieve_value(&self, val: Value) -> f64 {
        match val {
            Value::Lit(lit) => lit,
            Value::Reg(reg) => *self.regs.resolve(reg),
        }
    }
}

pub type VMResult = Result<(), ExecutionError>;

#[derive(Clone, Copy, Debug)]
pub enum ExecutionError {
    EmptyStack,
    StackOverflow,
    ZeroDivisionError,
    NoSuchMark,
}
