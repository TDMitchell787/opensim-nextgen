use anyhow::{anyhow, Result};
use std::collections::HashMap;

use super::opcodes::*;
use crate::scripting::{LSLRotation, LSLValue, LSLVector};

const DEFAULT_MAX_INSTRUCTIONS: u64 = 100_000;
const MAX_STACK_SIZE: usize = 4096;
const MAX_CALL_DEPTH: usize = 128;

#[derive(Debug)]
struct CallFrame {
    return_ip: usize,
    stack_base: usize,
    locals_base: usize,
    chunk_idx: usize,
}

pub enum VmResult {
    Complete(LSLValue),
    StateChange(String),
    Yield,
    Halt,
}

pub struct Vm {
    stack: Vec<LSLValue>,
    locals: Vec<LSLValue>,
    globals: HashMap<String, LSLValue>,
    call_stack: Vec<CallFrame>,
    ip: usize,
    instructions_executed: u64,
    max_instructions: u64,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            locals: Vec::with_capacity(64),
            globals: HashMap::new(),
            call_stack: Vec::new(),
            ip: 0,
            instructions_executed: 0,
            max_instructions: DEFAULT_MAX_INSTRUCTIONS,
        }
    }

    pub fn with_max_instructions(mut self, max: u64) -> Self {
        self.max_instructions = max;
        self
    }

    pub fn set_globals(&mut self, globals: HashMap<String, LSLValue>) {
        self.globals = globals;
    }

    pub fn get_globals(&self) -> &HashMap<String, LSLValue> {
        &self.globals
    }

    pub fn take_globals(self) -> HashMap<String, LSLValue> {
        self.globals
    }

    pub fn run_initializers(&mut self, chunk: &Chunk) -> Result<()> {
        self.ip = 0;
        self.stack.clear();
        self.locals.clear();

        while self.ip < chunk.code.len() {
            let op = &chunk.code[self.ip];
            self.ip += 1;

            match op {
                OpCode::Push(val) => {
                    self.stack.push(val.clone());
                }
                OpCode::StoreGlobal(idx) => {
                    let val = self.pop()?;
                    let name = chunk
                        .global_names
                        .get(*idx as usize)
                        .ok_or_else(|| anyhow!("Invalid global index {}", idx))?
                        .clone();
                    self.globals.insert(name, val);
                }
                OpCode::Add => self.binary_add()?,
                OpCode::Sub => self.binary_sub()?,
                OpCode::Mul => self.binary_mul()?,
                OpCode::Div => self.binary_div()?,
                OpCode::Neg => self.unary_neg()?,
                OpCode::MakeVector => self.make_vector()?,
                OpCode::MakeRotation => self.make_rotation()?,
                OpCode::MakeList(count) => self.make_list(*count)?,
                _ => {}
            }
        }

        Ok(())
    }

    pub fn execute(
        &mut self,
        chunk: &Chunk,
        event_args: &[LSLValue],
        program: &BytecodeProgram,
    ) -> Result<VmResult> {
        self.ip = 0;
        self.stack.clear();
        self.locals.clear();
        self.call_stack.clear();
        self.instructions_executed = 0;

        for arg in event_args {
            self.locals.push(arg.clone());
        }

        loop {
            if self.ip >= chunk.code.len() {
                let val = self.stack.pop().unwrap_or(LSLValue::Integer(0));
                return Ok(VmResult::Complete(val));
            }

            if self.instructions_executed >= self.max_instructions {
                return Ok(VmResult::Yield);
            }

            let op = chunk.code[self.ip].clone();
            self.ip += 1;
            self.instructions_executed += 1;

            match op {
                OpCode::Push(val) => {
                    if self.stack.len() >= MAX_STACK_SIZE {
                        return Err(anyhow!("Stack overflow"));
                    }
                    self.stack.push(val);
                }

                OpCode::Pop => {
                    self.stack.pop();
                }

                OpCode::Dup => {
                    let val = self.peek()?.clone();
                    self.stack.push(val);
                }

                OpCode::LoadGlobal(idx) => {
                    let name = chunk
                        .global_names
                        .get(idx as usize)
                        .ok_or_else(|| anyhow!("Invalid global index {}", idx))?;
                    let val = self
                        .globals
                        .get(name)
                        .cloned()
                        .unwrap_or(LSLValue::Integer(0));
                    self.stack.push(val);
                }

                OpCode::StoreGlobal(idx) => {
                    let val = self.pop()?;
                    let name = chunk
                        .global_names
                        .get(idx as usize)
                        .ok_or_else(|| anyhow!("Invalid global index {}", idx))?
                        .clone();
                    self.globals.insert(name, val);
                }

                OpCode::LoadLocal(idx) => {
                    let base = self.call_stack.last().map(|f| f.locals_base).unwrap_or(0);
                    let actual = base + idx as usize;
                    let val = self
                        .locals
                        .get(actual)
                        .cloned()
                        .unwrap_or(LSLValue::Integer(0));
                    self.stack.push(val);
                }

                OpCode::StoreLocal(idx) => {
                    let val = self.pop()?;
                    let base = self.call_stack.last().map(|f| f.locals_base).unwrap_or(0);
                    let actual = base + idx as usize;
                    while self.locals.len() <= actual {
                        self.locals.push(LSLValue::Integer(0));
                    }
                    self.locals[actual] = val;
                }

                OpCode::Add => self.binary_add()?,
                OpCode::Sub => self.binary_sub()?,
                OpCode::Mul => self.binary_mul()?,
                OpCode::Div => self.binary_div()?,
                OpCode::Mod => self.binary_mod()?,
                OpCode::Neg => self.unary_neg()?,

                OpCode::BitAnd => {
                    let b = self.pop()?.to_integer();
                    let a = self.pop()?.to_integer();
                    self.stack.push(LSLValue::Integer(a & b));
                }
                OpCode::BitOr => {
                    let b = self.pop()?.to_integer();
                    let a = self.pop()?.to_integer();
                    self.stack.push(LSLValue::Integer(a | b));
                }
                OpCode::BitXor => {
                    let b = self.pop()?.to_integer();
                    let a = self.pop()?.to_integer();
                    self.stack.push(LSLValue::Integer(a ^ b));
                }
                OpCode::BitNot => {
                    let a = self.pop()?.to_integer();
                    self.stack.push(LSLValue::Integer(!a));
                }
                OpCode::Shl => {
                    let b = self.pop()?.to_integer();
                    let a = self.pop()?.to_integer();
                    self.stack.push(LSLValue::Integer(a << (b & 31)));
                }
                OpCode::Shr => {
                    let b = self.pop()?.to_integer();
                    let a = self.pop()?.to_integer();
                    self.stack.push(LSLValue::Integer(a >> (b & 31)));
                }

                OpCode::Eq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack
                        .push(LSLValue::Integer(if self.values_equal(&a, &b) {
                            1
                        } else {
                            0
                        }));
                }
                OpCode::Ne => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack
                        .push(LSLValue::Integer(if !self.values_equal(&a, &b) {
                            1
                        } else {
                            0
                        }));
                }
                OpCode::Lt => {
                    let b = self.pop()?.to_float();
                    let a = self.pop()?.to_float();
                    self.stack
                        .push(LSLValue::Integer(if a < b { 1 } else { 0 }));
                }
                OpCode::Gt => {
                    let b = self.pop()?.to_float();
                    let a = self.pop()?.to_float();
                    self.stack
                        .push(LSLValue::Integer(if a > b { 1 } else { 0 }));
                }
                OpCode::Le => {
                    let b = self.pop()?.to_float();
                    let a = self.pop()?.to_float();
                    self.stack
                        .push(LSLValue::Integer(if a <= b { 1 } else { 0 }));
                }
                OpCode::Ge => {
                    let b = self.pop()?.to_float();
                    let a = self.pop()?.to_float();
                    self.stack
                        .push(LSLValue::Integer(if a >= b { 1 } else { 0 }));
                }

                OpCode::LogicalAnd => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack
                        .push(LSLValue::Integer(if a.is_true() && b.is_true() {
                            1
                        } else {
                            0
                        }));
                }
                OpCode::LogicalOr => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack
                        .push(LSLValue::Integer(if a.is_true() || b.is_true() {
                            1
                        } else {
                            0
                        }));
                }
                OpCode::LogicalNot => {
                    let a = self.pop()?;
                    self.stack
                        .push(LSLValue::Integer(if a.is_true() { 0 } else { 1 }));
                }

                OpCode::CastInteger => {
                    let val = self.pop()?;
                    self.stack.push(val.coerce("integer"));
                }
                OpCode::CastFloat => {
                    let val = self.pop()?;
                    self.stack.push(val.coerce("float"));
                }
                OpCode::CastString => {
                    let val = self.pop()?;
                    self.stack.push(val.coerce("string"));
                }
                OpCode::CastKey => {
                    let val = self.pop()?;
                    self.stack.push(val.coerce("key"));
                }
                OpCode::CastVector => {
                    let val = self.pop()?;
                    self.stack.push(val.coerce("vector"));
                }
                OpCode::CastRotation => {
                    let val = self.pop()?;
                    self.stack.push(val.coerce("rotation"));
                }
                OpCode::CastList => {
                    let val = self.pop()?;
                    self.stack.push(val.coerce("list"));
                }

                OpCode::MakeVector => self.make_vector()?,
                OpCode::MakeRotation => self.make_rotation()?,
                OpCode::MakeList(count) => self.make_list(count)?,

                OpCode::MemberGet(member) => {
                    let val = self.pop()?;
                    let result = match &val {
                        LSLValue::Vector(v) => match member {
                            MEMBER_X => LSLValue::Float(v.x),
                            MEMBER_Y => LSLValue::Float(v.y),
                            MEMBER_Z => LSLValue::Float(v.z),
                            _ => return Err(anyhow!("Invalid vector member")),
                        },
                        LSLValue::Rotation(r) => match member {
                            MEMBER_X => LSLValue::Float(r.x),
                            MEMBER_Y => LSLValue::Float(r.y),
                            MEMBER_Z => LSLValue::Float(r.z),
                            MEMBER_S => LSLValue::Float(r.s),
                            _ => return Err(anyhow!("Invalid rotation member")),
                        },
                        _ => return Err(anyhow!("Type has no members")),
                    };
                    self.stack.push(result);
                }

                OpCode::MemberSet(member) => {
                    // stack: [new_value, composite]
                    let composite = self.pop()?;
                    let new_val = self.pop()?.to_float();
                    let result = match composite {
                        LSLValue::Vector(mut v) => {
                            match member {
                                MEMBER_X => v.x = new_val,
                                MEMBER_Y => v.y = new_val,
                                MEMBER_Z => v.z = new_val,
                                _ => return Err(anyhow!("Invalid vector member")),
                            }
                            LSLValue::Vector(v)
                        }
                        LSLValue::Rotation(mut r) => {
                            match member {
                                MEMBER_X => r.x = new_val,
                                MEMBER_Y => r.y = new_val,
                                MEMBER_Z => r.z = new_val,
                                MEMBER_S => r.s = new_val,
                                _ => return Err(anyhow!("Invalid rotation member")),
                            }
                            LSLValue::Rotation(r)
                        }
                        _ => return Err(anyhow!("Type has no members")),
                    };
                    self.stack.push(result);
                }

                OpCode::Jump(target) => {
                    self.ip = target as usize;
                }

                OpCode::JumpIfFalse(target) => {
                    let val = self.peek()?;
                    if !val.is_true() {
                        self.ip = target as usize;
                    }
                }

                OpCode::JumpIfTrue(target) => {
                    let val = self.peek()?;
                    if val.is_true() {
                        self.ip = target as usize;
                    }
                }

                OpCode::Call(func_idx) => {
                    let func_name = chunk
                        .function_names
                        .get(func_idx as usize)
                        .ok_or_else(|| anyhow!("Invalid function index {}", func_idx))?
                        .clone();

                    if let Some(func) = program.find_function(&func_name) {
                        if self.call_stack.len() >= MAX_CALL_DEPTH {
                            return Err(anyhow!("Call depth limit exceeded"));
                        }

                        let arity = func.arity as usize;
                        let locals_base = self.locals.len();

                        // Move args from stack to locals
                        let stack_len = self.stack.len();
                        if stack_len < arity {
                            return Err(anyhow!("Not enough arguments for {}", func_name));
                        }
                        for i in 0..arity {
                            let arg = self.stack[stack_len - arity + i].clone();
                            self.locals.push(arg);
                        }
                        self.stack.truncate(stack_len - arity);

                        self.call_stack.push(CallFrame {
                            return_ip: self.ip,
                            stack_base: self.stack.len(),
                            locals_base,
                            chunk_idx: 0,
                        });

                        // Execute function body inline using its chunk
                        let result = self.execute_function_chunk(&func.chunk, program)?;
                        self.stack.push(result);
                    } else {
                        // Builtin or unknown — pop args, push default
                        let arity = self.stack.len().min(8);
                        for _ in 0..arity {
                            // We don't know exact arity for builtins here
                        }
                        self.stack.push(LSLValue::Integer(0));
                    }
                }

                OpCode::CallBuiltin(_func_idx) => {
                    self.stack.push(LSLValue::Integer(0));
                }

                OpCode::Return => {
                    let val = self.stack.pop().unwrap_or(LSLValue::Integer(0));

                    if let Some(frame) = self.call_stack.pop() {
                        self.locals.truncate(frame.locals_base);
                        self.ip = frame.return_ip;
                        self.stack.push(val);
                    } else {
                        return Ok(VmResult::Complete(val));
                    }
                }

                OpCode::PushFrame => {}
                OpCode::PopFrame => {}

                OpCode::StateChange(idx) => {
                    let name = match &chunk.constants[idx as usize] {
                        LSLValue::String(s) => s.clone(),
                        _ => return Err(anyhow!("Invalid state change constant")),
                    };
                    return Ok(VmResult::StateChange(name));
                }

                OpCode::Halt => {
                    let val = self.stack.pop().unwrap_or(LSLValue::Integer(0));
                    return Ok(VmResult::Complete(val));
                }

                OpCode::Yield => {
                    return Ok(VmResult::Yield);
                }
            }
        }
    }

    fn execute_function_chunk(
        &mut self,
        chunk: &Chunk,
        program: &BytecodeProgram,
    ) -> Result<LSLValue> {
        let saved_ip = self.ip;
        self.ip = 0;

        loop {
            if self.ip >= chunk.code.len() {
                self.ip = saved_ip;
                return Ok(self.stack.pop().unwrap_or(LSLValue::Integer(0)));
            }

            if self.instructions_executed >= self.max_instructions {
                self.ip = saved_ip;
                return Ok(LSLValue::Integer(0));
            }

            let op = chunk.code[self.ip].clone();
            self.ip += 1;
            self.instructions_executed += 1;

            match op {
                OpCode::Push(val) => {
                    self.stack.push(val);
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::Dup => {
                    let val = self.peek()?.clone();
                    self.stack.push(val);
                }

                OpCode::LoadGlobal(idx) => {
                    let name = chunk
                        .global_names
                        .get(idx as usize)
                        .ok_or_else(|| anyhow!("Invalid global index"))?;
                    let val = self
                        .globals
                        .get(name)
                        .cloned()
                        .unwrap_or(LSLValue::Integer(0));
                    self.stack.push(val);
                }
                OpCode::StoreGlobal(idx) => {
                    let val = self.pop()?;
                    let name = chunk
                        .global_names
                        .get(idx as usize)
                        .ok_or_else(|| anyhow!("Invalid global index"))?
                        .clone();
                    self.globals.insert(name, val);
                }
                OpCode::LoadLocal(idx) => {
                    let base = self.call_stack.last().map(|f| f.locals_base).unwrap_or(0);
                    let actual = base + idx as usize;
                    let val = self
                        .locals
                        .get(actual)
                        .cloned()
                        .unwrap_or(LSLValue::Integer(0));
                    self.stack.push(val);
                }
                OpCode::StoreLocal(idx) => {
                    let val = self.pop()?;
                    let base = self.call_stack.last().map(|f| f.locals_base).unwrap_or(0);
                    let actual = base + idx as usize;
                    while self.locals.len() <= actual {
                        self.locals.push(LSLValue::Integer(0));
                    }
                    self.locals[actual] = val;
                }

                OpCode::Add => self.binary_add()?,
                OpCode::Sub => self.binary_sub()?,
                OpCode::Mul => self.binary_mul()?,
                OpCode::Div => self.binary_div()?,
                OpCode::Mod => self.binary_mod()?,
                OpCode::Neg => self.unary_neg()?,

                OpCode::BitAnd => {
                    let b = self.pop()?.to_integer();
                    let a = self.pop()?.to_integer();
                    self.stack.push(LSLValue::Integer(a & b));
                }
                OpCode::BitOr => {
                    let b = self.pop()?.to_integer();
                    let a = self.pop()?.to_integer();
                    self.stack.push(LSLValue::Integer(a | b));
                }
                OpCode::BitXor => {
                    let b = self.pop()?.to_integer();
                    let a = self.pop()?.to_integer();
                    self.stack.push(LSLValue::Integer(a ^ b));
                }
                OpCode::BitNot => {
                    let a = self.pop()?.to_integer();
                    self.stack.push(LSLValue::Integer(!a));
                }
                OpCode::Shl => {
                    let b = self.pop()?.to_integer();
                    let a = self.pop()?.to_integer();
                    self.stack.push(LSLValue::Integer(a << (b & 31)));
                }
                OpCode::Shr => {
                    let b = self.pop()?.to_integer();
                    let a = self.pop()?.to_integer();
                    self.stack.push(LSLValue::Integer(a >> (b & 31)));
                }

                OpCode::Eq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack
                        .push(LSLValue::Integer(if self.values_equal(&a, &b) {
                            1
                        } else {
                            0
                        }));
                }
                OpCode::Ne => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack
                        .push(LSLValue::Integer(if !self.values_equal(&a, &b) {
                            1
                        } else {
                            0
                        }));
                }
                OpCode::Lt => {
                    let b = self.pop()?.to_float();
                    let a = self.pop()?.to_float();
                    self.stack
                        .push(LSLValue::Integer(if a < b { 1 } else { 0 }));
                }
                OpCode::Gt => {
                    let b = self.pop()?.to_float();
                    let a = self.pop()?.to_float();
                    self.stack
                        .push(LSLValue::Integer(if a > b { 1 } else { 0 }));
                }
                OpCode::Le => {
                    let b = self.pop()?.to_float();
                    let a = self.pop()?.to_float();
                    self.stack
                        .push(LSLValue::Integer(if a <= b { 1 } else { 0 }));
                }
                OpCode::Ge => {
                    let b = self.pop()?.to_float();
                    let a = self.pop()?.to_float();
                    self.stack
                        .push(LSLValue::Integer(if a >= b { 1 } else { 0 }));
                }

                OpCode::LogicalAnd => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack
                        .push(LSLValue::Integer(if a.is_true() && b.is_true() {
                            1
                        } else {
                            0
                        }));
                }
                OpCode::LogicalOr => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack
                        .push(LSLValue::Integer(if a.is_true() || b.is_true() {
                            1
                        } else {
                            0
                        }));
                }
                OpCode::LogicalNot => {
                    let a = self.pop()?;
                    self.stack
                        .push(LSLValue::Integer(if a.is_true() { 0 } else { 1 }));
                }

                OpCode::CastInteger => {
                    let v = self.pop()?;
                    self.stack.push(v.coerce("integer"));
                }
                OpCode::CastFloat => {
                    let v = self.pop()?;
                    self.stack.push(v.coerce("float"));
                }
                OpCode::CastString => {
                    let v = self.pop()?;
                    self.stack.push(v.coerce("string"));
                }
                OpCode::CastKey => {
                    let v = self.pop()?;
                    self.stack.push(v.coerce("key"));
                }
                OpCode::CastVector => {
                    let v = self.pop()?;
                    self.stack.push(v.coerce("vector"));
                }
                OpCode::CastRotation => {
                    let v = self.pop()?;
                    self.stack.push(v.coerce("rotation"));
                }
                OpCode::CastList => {
                    let v = self.pop()?;
                    self.stack.push(v.coerce("list"));
                }

                OpCode::MakeVector => self.make_vector()?,
                OpCode::MakeRotation => self.make_rotation()?,
                OpCode::MakeList(count) => self.make_list(count)?,

                OpCode::MemberGet(m) => {
                    let val = self.pop()?;
                    let result = match &val {
                        LSLValue::Vector(v) => match m {
                            MEMBER_X => LSLValue::Float(v.x),
                            MEMBER_Y => LSLValue::Float(v.y),
                            MEMBER_Z => LSLValue::Float(v.z),
                            _ => return Err(anyhow!("Invalid member")),
                        },
                        LSLValue::Rotation(r) => match m {
                            MEMBER_X => LSLValue::Float(r.x),
                            MEMBER_Y => LSLValue::Float(r.y),
                            MEMBER_Z => LSLValue::Float(r.z),
                            MEMBER_S => LSLValue::Float(r.s),
                            _ => return Err(anyhow!("Invalid member")),
                        },
                        _ => return Err(anyhow!("Type has no members")),
                    };
                    self.stack.push(result);
                }
                OpCode::MemberSet(m) => {
                    let composite = self.pop()?;
                    let new_val = self.pop()?.to_float();
                    let result = match composite {
                        LSLValue::Vector(mut v) => {
                            match m {
                                MEMBER_X => v.x = new_val,
                                MEMBER_Y => v.y = new_val,
                                MEMBER_Z => v.z = new_val,
                                _ => return Err(anyhow!("Invalid member")),
                            }
                            LSLValue::Vector(v)
                        }
                        LSLValue::Rotation(mut r) => {
                            match m {
                                MEMBER_X => r.x = new_val,
                                MEMBER_Y => r.y = new_val,
                                MEMBER_Z => r.z = new_val,
                                MEMBER_S => r.s = new_val,
                                _ => return Err(anyhow!("Invalid member")),
                            }
                            LSLValue::Rotation(r)
                        }
                        _ => return Err(anyhow!("Type has no members")),
                    };
                    self.stack.push(result);
                }

                OpCode::Jump(target) => {
                    self.ip = target as usize;
                }
                OpCode::JumpIfFalse(target) => {
                    let val = self.peek()?;
                    if !val.is_true() {
                        self.ip = target as usize;
                    }
                }
                OpCode::JumpIfTrue(target) => {
                    let val = self.peek()?;
                    if val.is_true() {
                        self.ip = target as usize;
                    }
                }

                OpCode::Call(func_idx) => {
                    let func_name = chunk
                        .function_names
                        .get(func_idx as usize)
                        .ok_or_else(|| anyhow!("Invalid function index"))?
                        .clone();

                    if let Some(func) = program.find_function(&func_name) {
                        let arity = func.arity as usize;
                        let locals_base = self.locals.len();
                        let stack_len = self.stack.len();
                        if stack_len < arity {
                            return Err(anyhow!("Not enough arguments"));
                        }
                        for i in 0..arity {
                            let arg = self.stack[stack_len - arity + i].clone();
                            self.locals.push(arg);
                        }
                        self.stack.truncate(stack_len - arity);

                        self.call_stack.push(CallFrame {
                            return_ip: self.ip,
                            stack_base: self.stack.len(),
                            locals_base,
                            chunk_idx: 0,
                        });

                        let result = self.execute_function_chunk(&func.chunk, program)?;
                        self.stack.push(result);
                    } else {
                        self.stack.push(LSLValue::Integer(0));
                    }
                }

                OpCode::CallBuiltin(_) => {
                    self.stack.push(LSLValue::Integer(0));
                }

                OpCode::Return => {
                    let val = self.stack.pop().unwrap_or(LSLValue::Integer(0));
                    if let Some(frame) = self.call_stack.pop() {
                        self.locals.truncate(frame.locals_base);
                        self.ip = frame.return_ip;
                        // Restore IP to saved_ip since we're returning from recursive call
                        self.ip = saved_ip;
                        return Ok(val);
                    } else {
                        self.ip = saved_ip;
                        return Ok(val);
                    }
                }

                OpCode::PushFrame | OpCode::PopFrame => {}

                OpCode::StateChange(idx) => {
                    self.ip = saved_ip;
                    return Err(anyhow!("State change in function"));
                }

                OpCode::Halt => {
                    self.ip = saved_ip;
                    return Ok(self.stack.pop().unwrap_or(LSLValue::Integer(0)));
                }

                OpCode::Yield => {
                    self.ip = saved_ip;
                    return Ok(LSLValue::Integer(0));
                }
            }
        }
    }

    fn pop(&mut self) -> Result<LSLValue> {
        self.stack.pop().ok_or_else(|| anyhow!("Stack underflow"))
    }

    fn peek(&self) -> Result<&LSLValue> {
        self.stack.last().ok_or_else(|| anyhow!("Stack underflow"))
    }

    fn binary_add(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (LSLValue::Integer(x), LSLValue::Integer(y)) => LSLValue::Integer(x.wrapping_add(*y)),
            (LSLValue::Float(x), LSLValue::Float(y)) => LSLValue::Float(x + y),
            (LSLValue::Integer(x), LSLValue::Float(y)) => LSLValue::Float(*x as f32 + y),
            (LSLValue::Float(x), LSLValue::Integer(y)) => LSLValue::Float(x + *y as f32),
            (LSLValue::String(x), LSLValue::String(y)) => LSLValue::String(format!("{}{}", x, y)),
            (LSLValue::Vector(x), LSLValue::Vector(y)) => LSLValue::Vector(*x + *y),
            (LSLValue::List(x), LSLValue::List(y)) => {
                let mut r = x.clone();
                r.extend(y.iter().cloned());
                LSLValue::List(r)
            }
            _ => LSLValue::Integer(0),
        };
        self.stack.push(result);
        Ok(())
    }

    fn binary_sub(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (LSLValue::Integer(x), LSLValue::Integer(y)) => LSLValue::Integer(x.wrapping_sub(*y)),
            (LSLValue::Float(x), LSLValue::Float(y)) => LSLValue::Float(x - y),
            (LSLValue::Integer(x), LSLValue::Float(y)) => LSLValue::Float(*x as f32 - y),
            (LSLValue::Float(x), LSLValue::Integer(y)) => LSLValue::Float(x - *y as f32),
            (LSLValue::Vector(x), LSLValue::Vector(y)) => LSLValue::Vector(*x - *y),
            _ => LSLValue::Integer(0),
        };
        self.stack.push(result);
        Ok(())
    }

    fn binary_mul(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (LSLValue::Integer(x), LSLValue::Integer(y)) => LSLValue::Integer(x.wrapping_mul(*y)),
            (LSLValue::Float(x), LSLValue::Float(y)) => LSLValue::Float(x * y),
            (LSLValue::Integer(x), LSLValue::Float(y)) => LSLValue::Float(*x as f32 * y),
            (LSLValue::Float(x), LSLValue::Integer(y)) => LSLValue::Float(x * *y as f32),
            (LSLValue::Vector(x), LSLValue::Float(y)) => LSLValue::Vector(*x * *y),
            (LSLValue::Float(x), LSLValue::Vector(y)) => LSLValue::Vector(*y * *x),
            (LSLValue::Rotation(x), LSLValue::Rotation(y)) => LSLValue::Rotation(*x * *y),
            _ => LSLValue::Integer(0),
        };
        self.stack.push(result);
        Ok(())
    }

    fn binary_div(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (LSLValue::Integer(x), LSLValue::Integer(y)) => {
                if *y == 0 {
                    LSLValue::Integer(0)
                } else {
                    LSLValue::Integer(x / y)
                }
            }
            (LSLValue::Float(x), LSLValue::Float(y)) => {
                if *y == 0.0 {
                    LSLValue::Float(0.0)
                } else {
                    LSLValue::Float(x / y)
                }
            }
            (LSLValue::Integer(x), LSLValue::Float(y)) => {
                if *y == 0.0 {
                    LSLValue::Float(0.0)
                } else {
                    LSLValue::Float(*x as f32 / y)
                }
            }
            (LSLValue::Float(x), LSLValue::Integer(y)) => {
                if *y == 0 {
                    LSLValue::Float(0.0)
                } else {
                    LSLValue::Float(x / *y as f32)
                }
            }
            (LSLValue::Vector(x), LSLValue::Float(y)) => {
                if *y == 0.0 {
                    LSLValue::Vector(LSLVector::zero())
                } else {
                    LSLValue::Vector(*x / *y)
                }
            }
            _ => LSLValue::Integer(0),
        };
        self.stack.push(result);
        Ok(())
    }

    fn binary_mod(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (LSLValue::Integer(x), LSLValue::Integer(y)) => {
                if *y == 0 {
                    LSLValue::Integer(0)
                } else {
                    LSLValue::Integer(x % y)
                }
            }
            _ => LSLValue::Integer(0),
        };
        self.stack.push(result);
        Ok(())
    }

    fn unary_neg(&mut self) -> Result<()> {
        let a = self.pop()?;
        let result = match a {
            LSLValue::Integer(i) => LSLValue::Integer(-i),
            LSLValue::Float(f) => LSLValue::Float(-f),
            LSLValue::Vector(v) => LSLValue::Vector(LSLVector {
                x: -v.x,
                y: -v.y,
                z: -v.z,
            }),
            LSLValue::Rotation(r) => LSLValue::Rotation(LSLRotation {
                x: -r.x,
                y: -r.y,
                z: -r.z,
                s: -r.s,
            }),
            _ => LSLValue::Integer(0),
        };
        self.stack.push(result);
        Ok(())
    }

    fn make_vector(&mut self) -> Result<()> {
        let z = self.pop()?.to_float();
        let y = self.pop()?.to_float();
        let x = self.pop()?.to_float();
        self.stack.push(LSLValue::Vector(LSLVector { x, y, z }));
        Ok(())
    }

    fn make_rotation(&mut self) -> Result<()> {
        let s = self.pop()?.to_float();
        let z = self.pop()?.to_float();
        let y = self.pop()?.to_float();
        let x = self.pop()?.to_float();
        self.stack
            .push(LSLValue::Rotation(LSLRotation { x, y, z, s }));
        Ok(())
    }

    fn make_list(&mut self, count: u16) -> Result<()> {
        let count = count as usize;
        let stack_len = self.stack.len();
        if stack_len < count {
            return Err(anyhow!("Not enough values for list"));
        }
        let items: Vec<LSLValue> = self.stack.drain(stack_len - count..).collect();
        self.stack.push(LSLValue::List(items));
        Ok(())
    }

    fn values_equal(&self, a: &LSLValue, b: &LSLValue) -> bool {
        match (a, b) {
            (LSLValue::Integer(x), LSLValue::Integer(y)) => x == y,
            (LSLValue::Float(x), LSLValue::Float(y)) => (x - y).abs() < f32::EPSILON,
            (LSLValue::String(x), LSLValue::String(y)) => x == y,
            (LSLValue::Key(x), LSLValue::Key(y)) => x == y,
            (LSLValue::Vector(x), LSLValue::Vector(y)) => x == y,
            (LSLValue::Rotation(x), LSLValue::Rotation(y)) => x == y,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_arithmetic() -> Result<()> {
        let mut chunk = Chunk::new();
        chunk.emit(OpCode::Push(LSLValue::Integer(10)));
        chunk.emit(OpCode::Push(LSLValue::Integer(32)));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::Halt);

        let program = BytecodeProgram::new();
        let mut vm = Vm::new();
        let result = vm.execute(&chunk, &[], &program)?;

        match result {
            VmResult::Complete(LSLValue::Integer(42)) => {}
            _ => panic!("Expected Integer(42)"),
        }
        Ok(())
    }

    #[test]
    fn test_vm_globals() -> Result<()> {
        let mut chunk = Chunk::new();
        let g_idx = chunk.add_global("x");
        chunk.emit(OpCode::Push(LSLValue::Integer(100)));
        chunk.emit(OpCode::StoreGlobal(g_idx));
        chunk.emit(OpCode::LoadGlobal(g_idx));
        chunk.emit(OpCode::Push(LSLValue::Integer(5)));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::Halt);

        let program = BytecodeProgram::new();
        let mut vm = Vm::new();
        let result = vm.execute(&chunk, &[], &program)?;

        match result {
            VmResult::Complete(LSLValue::Integer(105)) => {}
            _ => panic!("Expected Integer(105)"),
        }
        Ok(())
    }

    #[test]
    fn test_vm_vector() -> Result<()> {
        let mut chunk = Chunk::new();
        chunk.emit(OpCode::Push(LSLValue::Float(1.0)));
        chunk.emit(OpCode::Push(LSLValue::Float(2.0)));
        chunk.emit(OpCode::Push(LSLValue::Float(3.0)));
        chunk.emit(OpCode::MakeVector);
        chunk.emit(OpCode::MemberGet(MEMBER_Y));
        chunk.emit(OpCode::Halt);

        let program = BytecodeProgram::new();
        let mut vm = Vm::new();
        let result = vm.execute(&chunk, &[], &program)?;

        match result {
            VmResult::Complete(LSLValue::Float(f)) if (f - 2.0).abs() < f32::EPSILON => {}
            _ => panic!("Expected Float(2.0)"),
        }
        Ok(())
    }

    #[test]
    fn test_vm_conditional() -> Result<()> {
        let mut chunk = Chunk::new();
        chunk.emit(OpCode::Push(LSLValue::Integer(1)));
        let false_jump = chunk.emit(OpCode::JumpIfFalse(0));
        chunk.emit(OpCode::Pop);
        chunk.emit(OpCode::Push(LSLValue::Integer(42)));
        let end_jump = chunk.emit(OpCode::Jump(0));
        let false_pos = chunk.current_pos() as i32;
        chunk.patch_jump(false_jump, false_pos);
        chunk.emit(OpCode::Pop);
        chunk.emit(OpCode::Push(LSLValue::Integer(99)));
        let end_pos = chunk.current_pos() as i32;
        chunk.patch_jump(end_jump, end_pos);
        chunk.emit(OpCode::Halt);

        let program = BytecodeProgram::new();
        let mut vm = Vm::new();
        let result = vm.execute(&chunk, &[], &program)?;

        match result {
            VmResult::Complete(LSLValue::Integer(42)) => {}
            _ => panic!("Expected Integer(42)"),
        }
        Ok(())
    }

    #[test]
    fn test_vm_loop() -> Result<()> {
        let mut chunk = Chunk::new();
        let g_idx = chunk.add_global("count");
        chunk.emit(OpCode::Push(LSLValue::Integer(0)));
        chunk.emit(OpCode::StoreGlobal(g_idx));

        let loop_start = chunk.current_pos() as i32;
        chunk.emit(OpCode::LoadGlobal(g_idx));
        chunk.emit(OpCode::Push(LSLValue::Integer(5)));
        chunk.emit(OpCode::Lt);
        let exit_jump = chunk.emit(OpCode::JumpIfFalse(0));
        chunk.emit(OpCode::Pop);

        chunk.emit(OpCode::LoadGlobal(g_idx));
        chunk.emit(OpCode::Push(LSLValue::Integer(1)));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::StoreGlobal(g_idx));
        chunk.emit(OpCode::Jump(loop_start));

        let end_pos = chunk.current_pos() as i32;
        chunk.patch_jump(exit_jump, end_pos);
        chunk.emit(OpCode::Pop);

        chunk.emit(OpCode::LoadGlobal(g_idx));
        chunk.emit(OpCode::Halt);

        let program = BytecodeProgram::new();
        let mut vm = Vm::new();
        let result = vm.execute(&chunk, &[], &program)?;

        match result {
            VmResult::Complete(LSLValue::Integer(5)) => {}
            _ => panic!("Expected Integer(5)"),
        }
        Ok(())
    }

    #[test]
    fn test_vm_instruction_limit() -> Result<()> {
        let mut chunk = Chunk::new();
        let loop_start = chunk.current_pos() as i32;
        chunk.emit(OpCode::Push(LSLValue::Integer(1)));
        chunk.emit(OpCode::Pop);
        chunk.emit(OpCode::Jump(loop_start));

        let program = BytecodeProgram::new();
        let mut vm = Vm::new().with_max_instructions(10);
        let result = vm.execute(&chunk, &[], &program)?;

        assert!(matches!(result, VmResult::Yield));
        Ok(())
    }
}
