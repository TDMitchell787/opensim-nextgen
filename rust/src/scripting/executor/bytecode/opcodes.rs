use crate::scripting::LSLValue;

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    // Stack manipulation
    Push(LSLValue),
    Pop,
    Dup,

    // Variable access
    LoadGlobal(u16),
    StoreGlobal(u16),
    LoadLocal(u16),
    StoreLocal(u16),

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    Shl,
    Shr,

    // Comparison
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,

    // Logical
    LogicalAnd,
    LogicalOr,
    LogicalNot,

    // Type casts
    CastInteger,
    CastFloat,
    CastString,
    CastKey,
    CastVector,
    CastRotation,
    CastList,

    // Composite types
    MakeVector,
    MakeRotation,
    MakeList(u16),
    MemberGet(u8),
    MemberSet(u8),

    // Control flow
    Jump(i32),
    JumpIfFalse(i32),
    JumpIfTrue(i32),

    // Functions
    Call(u16),
    CallBuiltin(u16),
    Return,

    // Scope
    PushFrame,
    PopFrame,

    // Script
    StateChange(u16),
    Halt,
    Yield,
}

pub const MEMBER_X: u8 = 0;
pub const MEMBER_Y: u8 = 1;
pub const MEMBER_Z: u8 = 2;
pub const MEMBER_S: u8 = 3;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<LSLValue>,
    pub global_names: Vec<String>,
    pub function_names: Vec<String>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            global_names: Vec::new(),
            function_names: Vec::new(),
        }
    }

    pub fn emit(&mut self, op: OpCode) -> usize {
        let pos = self.code.len();
        self.code.push(op);
        pos
    }

    pub fn add_constant(&mut self, value: LSLValue) -> u16 {
        for (i, c) in self.constants.iter().enumerate() {
            if Self::values_match(c, &value) {
                return i as u16;
            }
        }
        let idx = self.constants.len();
        self.constants.push(value);
        idx as u16
    }

    fn values_match(a: &LSLValue, b: &LSLValue) -> bool {
        match (a, b) {
            (LSLValue::Integer(x), LSLValue::Integer(y)) => x == y,
            (LSLValue::Float(x), LSLValue::Float(y)) => x.to_bits() == y.to_bits(),
            (LSLValue::String(x), LSLValue::String(y)) => x == y,
            _ => false,
        }
    }

    pub fn add_global(&mut self, name: &str) -> u16 {
        for (i, n) in self.global_names.iter().enumerate() {
            if n == name {
                return i as u16;
            }
        }
        let idx = self.global_names.len();
        self.global_names.push(name.to_string());
        idx as u16
    }

    pub fn add_function(&mut self, name: &str) -> u16 {
        for (i, n) in self.function_names.iter().enumerate() {
            if n == name {
                return i as u16;
            }
        }
        let idx = self.function_names.len();
        self.function_names.push(name.to_string());
        idx as u16
    }

    pub fn patch_jump(&mut self, pos: usize, target: i32) {
        match &mut self.code[pos] {
            OpCode::Jump(ref mut t) => *t = target,
            OpCode::JumpIfFalse(ref mut t) => *t = target,
            OpCode::JumpIfTrue(ref mut t) => *t = target,
            _ => {}
        }
    }

    pub fn current_pos(&self) -> usize {
        self.code.len()
    }
}

#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub name: String,
    pub arity: u16,
    pub param_names: Vec<String>,
    pub param_types: Vec<String>,
    pub return_type: String,
    pub chunk: Chunk,
}

#[derive(Debug, Clone)]
pub struct CompiledEvent {
    pub name: String,
    pub param_names: Vec<String>,
    pub param_types: Vec<String>,
    pub chunk: Chunk,
}

#[derive(Debug, Clone)]
pub struct BytecodeProgram {
    pub globals: Vec<(String, String)>,
    pub global_initializers: Chunk,
    pub functions: Vec<CompiledFunction>,
    pub states: Vec<(String, Vec<CompiledEvent>)>,
}

impl BytecodeProgram {
    pub fn new() -> Self {
        Self {
            globals: Vec::new(),
            global_initializers: Chunk::new(),
            functions: Vec::new(),
            states: Vec::new(),
        }
    }

    pub fn find_function(&self, name: &str) -> Option<&CompiledFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    pub fn find_event(&self, state: &str, event: &str) -> Option<&CompiledEvent> {
        self.states.iter()
            .find(|(s, _)| s == state)
            .and_then(|(_, events)| events.iter().find(|e| e.name == event))
    }
}
