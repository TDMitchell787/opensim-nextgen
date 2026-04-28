use anyhow::{anyhow, Result};

use super::opcodes::*;
use crate::scripting::lsl_interpreter::{ASTNode, Token};
use crate::scripting::{LSLRotation, LSLValue, LSLVector};

pub struct BytecodeCompiler {
    locals: Vec<Vec<(String, u16)>>,
    next_local: u16,
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            next_local: 0,
        }
    }

    pub fn compile_program(&mut self, ast: &ASTNode) -> Result<BytecodeProgram> {
        let statements = match ast {
            ASTNode::Program(stmts) => stmts,
            _ => return Err(anyhow!("Expected Program node")),
        };

        let mut program = BytecodeProgram::new();

        // First pass: collect globals and functions
        for node in statements {
            match node {
                ASTNode::Variable { var_type, name, .. } => {
                    program.globals.push((name.clone(), var_type.clone()));
                }
                _ => {}
            }
        }

        // Second pass: compile global initializers
        for node in statements {
            if let ASTNode::Variable {
                var_type,
                name,
                value,
            } = node
            {
                let global_idx = program.global_initializers.add_global(name);
                if let Some(init_expr) = value {
                    self.compile_expr(&mut program.global_initializers, init_expr)?;
                } else {
                    let default_val = LSLValue::type_default(var_type);
                    program.global_initializers.emit(OpCode::Push(default_val));
                }
                program
                    .global_initializers
                    .emit(OpCode::StoreGlobal(global_idx));
            }
        }

        // Third pass: compile functions
        for node in statements {
            if let ASTNode::Function {
                name,
                return_type,
                parameters,
                body,
            } = node
            {
                let func =
                    self.compile_function(name, return_type, parameters, body, &program.globals)?;
                program.functions.push(func);
            }
        }

        // Fourth pass: compile states
        for node in statements {
            if let ASTNode::State { name, body } = node {
                let mut events = Vec::new();
                for event_node in body {
                    if let ASTNode::Event {
                        name: event_name,
                        parameters,
                        body: event_body,
                    } = event_node
                    {
                        let compiled = self.compile_event(
                            event_name,
                            parameters,
                            event_body,
                            &program.globals,
                        )?;
                        events.push(compiled);
                    }
                }
                program.states.push((name.clone(), events));
            }
        }

        Ok(program)
    }

    fn compile_function(
        &mut self,
        name: &str,
        return_type: &str,
        parameters: &[(String, String)],
        body: &[ASTNode],
        globals: &[(String, String)],
    ) -> Result<CompiledFunction> {
        let mut chunk = Chunk::new();

        for (gname, _) in globals {
            chunk.add_global(gname);
        }

        self.begin_scope();
        for (pname, _ptype) in parameters {
            self.define_local(pname);
        }

        self.compile_block(&mut chunk, body)?;

        if return_type == "void" || return_type.is_empty() {
            chunk.emit(OpCode::Push(LSLValue::Integer(0)));
        }
        chunk.emit(OpCode::Return);

        self.end_scope();

        Ok(CompiledFunction {
            name: name.to_string(),
            arity: parameters.len() as u16,
            param_names: parameters.iter().map(|(n, _)| n.clone()).collect(),
            param_types: parameters.iter().map(|(_, t)| t.clone()).collect(),
            return_type: return_type.to_string(),
            chunk,
        })
    }

    fn compile_event(
        &mut self,
        name: &str,
        parameters: &[(String, String)],
        body: &[ASTNode],
        globals: &[(String, String)],
    ) -> Result<CompiledEvent> {
        let mut chunk = Chunk::new();

        for (gname, _) in globals {
            chunk.add_global(gname);
        }

        self.begin_scope();
        for (pname, _ptype) in parameters {
            self.define_local(pname);
        }

        self.compile_block(&mut chunk, body)?;
        chunk.emit(OpCode::Halt);

        self.end_scope();

        Ok(CompiledEvent {
            name: name.to_string(),
            param_names: parameters.iter().map(|(n, _)| n.clone()).collect(),
            param_types: parameters.iter().map(|(_, t)| t.clone()).collect(),
            chunk,
        })
    }

    fn compile_block(&mut self, chunk: &mut Chunk, statements: &[ASTNode]) -> Result<()> {
        for stmt in statements {
            self.compile_node(chunk, stmt)?;
        }
        Ok(())
    }

    fn compile_node(&mut self, chunk: &mut Chunk, node: &ASTNode) -> Result<()> {
        match node {
            ASTNode::Literal(val) => {
                chunk.emit(OpCode::Push(val.clone()));
            }

            ASTNode::Identifier(name) => {
                if let Some(local_idx) = self.resolve_local(name) {
                    chunk.emit(OpCode::LoadLocal(local_idx));
                } else {
                    let global_idx = chunk.add_global(name);
                    chunk.emit(OpCode::LoadGlobal(global_idx));
                }
            }

            ASTNode::Variable {
                var_type,
                name,
                value,
            } => {
                if let Some(init_expr) = value {
                    self.compile_expr(chunk, init_expr)?;
                } else {
                    let default_val = LSLValue::type_default(var_type);
                    chunk.emit(OpCode::Push(default_val));
                }
                let local_idx = self.define_local(name);
                chunk.emit(OpCode::StoreLocal(local_idx));
            }

            ASTNode::Assignment { target, value } => {
                self.compile_expr(chunk, value)?;
                chunk.emit(OpCode::Dup);
                self.compile_store(chunk, target)?;
            }

            ASTNode::CompoundAssignment {
                target,
                operator,
                value,
            } => {
                self.compile_expr(chunk, target)?;
                self.compile_expr(chunk, value)?;
                match operator {
                    Token::PlusAssign => chunk.emit(OpCode::Add),
                    Token::MinusAssign => chunk.emit(OpCode::Sub),
                    Token::MultiplyAssign => chunk.emit(OpCode::Mul),
                    Token::DivideAssign => chunk.emit(OpCode::Div),
                    Token::ModuloAssign => chunk.emit(OpCode::Mod),
                    _ => return Err(anyhow!("Invalid compound assignment operator")),
                };
                chunk.emit(OpCode::Dup);
                self.compile_store(chunk, target)?;
            }

            ASTNode::BinaryOp {
                left,
                operator,
                right,
            } => {
                // Short-circuit for logical AND/OR
                match operator {
                    Token::And => {
                        self.compile_expr(chunk, left)?;
                        let jump_pos = chunk.emit(OpCode::JumpIfFalse(0));
                        chunk.emit(OpCode::Pop);
                        self.compile_expr(chunk, right)?;
                        let end_pos = chunk.current_pos() as i32;
                        chunk.patch_jump(jump_pos, end_pos);
                    }
                    Token::Or => {
                        self.compile_expr(chunk, left)?;
                        let jump_pos = chunk.emit(OpCode::JumpIfTrue(0));
                        chunk.emit(OpCode::Pop);
                        self.compile_expr(chunk, right)?;
                        let end_pos = chunk.current_pos() as i32;
                        chunk.patch_jump(jump_pos, end_pos);
                    }
                    _ => {
                        self.compile_expr(chunk, left)?;
                        self.compile_expr(chunk, right)?;
                        match operator {
                            Token::Plus => chunk.emit(OpCode::Add),
                            Token::Minus => chunk.emit(OpCode::Sub),
                            Token::Multiply => chunk.emit(OpCode::Mul),
                            Token::Divide => chunk.emit(OpCode::Div),
                            Token::Modulo => chunk.emit(OpCode::Mod),
                            Token::Equal => chunk.emit(OpCode::Eq),
                            Token::NotEqual => chunk.emit(OpCode::Ne),
                            Token::Less => chunk.emit(OpCode::Lt),
                            Token::Greater => chunk.emit(OpCode::Gt),
                            Token::LessEqual => chunk.emit(OpCode::Le),
                            Token::GreaterEqual => chunk.emit(OpCode::Ge),
                            Token::BitAnd => chunk.emit(OpCode::BitAnd),
                            Token::BitOr => chunk.emit(OpCode::BitOr),
                            Token::BitXor => chunk.emit(OpCode::BitXor),
                            Token::ShiftLeft => chunk.emit(OpCode::Shl),
                            Token::ShiftRight => chunk.emit(OpCode::Shr),
                            _ => {
                                return Err(anyhow!("Unsupported binary operator: {:?}", operator))
                            }
                        };
                    }
                }
            }

            ASTNode::UnaryOp { operator, operand } => {
                self.compile_expr(chunk, operand)?;
                match operator {
                    Token::Minus => chunk.emit(OpCode::Neg),
                    Token::Not => chunk.emit(OpCode::LogicalNot),
                    Token::BitNot => chunk.emit(OpCode::BitNot),
                    _ => return Err(anyhow!("Unsupported unary operator: {:?}", operator)),
                };
            }

            ASTNode::PreIncrement(expr) => {
                self.compile_expr(chunk, expr)?;
                chunk.emit(OpCode::Push(LSLValue::Integer(1)));
                chunk.emit(OpCode::Add);
                chunk.emit(OpCode::Dup);
                self.compile_store(chunk, expr)?;
            }

            ASTNode::PreDecrement(expr) => {
                self.compile_expr(chunk, expr)?;
                chunk.emit(OpCode::Push(LSLValue::Integer(1)));
                chunk.emit(OpCode::Sub);
                chunk.emit(OpCode::Dup);
                self.compile_store(chunk, expr)?;
            }

            ASTNode::PostIncrement(expr) => {
                self.compile_expr(chunk, expr)?;
                chunk.emit(OpCode::Dup);
                chunk.emit(OpCode::Push(LSLValue::Integer(1)));
                chunk.emit(OpCode::Add);
                self.compile_store(chunk, expr)?;
            }

            ASTNode::PostDecrement(expr) => {
                self.compile_expr(chunk, expr)?;
                chunk.emit(OpCode::Dup);
                chunk.emit(OpCode::Push(LSLValue::Integer(1)));
                chunk.emit(OpCode::Sub);
                self.compile_store(chunk, expr)?;
            }

            ASTNode::TypeCast { target_type, expr } => {
                self.compile_expr(chunk, expr)?;
                match target_type.as_str() {
                    "integer" => chunk.emit(OpCode::CastInteger),
                    "float" => chunk.emit(OpCode::CastFloat),
                    "string" => chunk.emit(OpCode::CastString),
                    "key" => chunk.emit(OpCode::CastKey),
                    "vector" => chunk.emit(OpCode::CastVector),
                    "rotation" => chunk.emit(OpCode::CastRotation),
                    "list" => chunk.emit(OpCode::CastList),
                    _ => return Err(anyhow!("Unknown cast type: {}", target_type)),
                };
            }

            ASTNode::MemberAccess { object, member } => {
                self.compile_expr(chunk, object)?;
                let member_id = match member.as_str() {
                    "x" => MEMBER_X,
                    "y" => MEMBER_Y,
                    "z" => MEMBER_Z,
                    "s" => MEMBER_S,
                    _ => return Err(anyhow!("Unknown member: {}", member)),
                };
                chunk.emit(OpCode::MemberGet(member_id));
            }

            ASTNode::VectorLiteral { x, y, z } => {
                self.compile_expr(chunk, x)?;
                self.compile_expr(chunk, y)?;
                self.compile_expr(chunk, z)?;
                chunk.emit(OpCode::MakeVector);
            }

            ASTNode::RotationLiteral { x, y, z, s } => {
                self.compile_expr(chunk, x)?;
                self.compile_expr(chunk, y)?;
                self.compile_expr(chunk, z)?;
                self.compile_expr(chunk, s)?;
                chunk.emit(OpCode::MakeRotation);
            }

            ASTNode::ListLiteral(elements) => {
                for elem in elements {
                    self.compile_expr(chunk, elem)?;
                }
                chunk.emit(OpCode::MakeList(elements.len() as u16));
            }

            ASTNode::FunctionCall { name, arguments } => {
                for arg in arguments {
                    self.compile_expr(chunk, arg)?;
                }
                let func_idx = chunk.add_function(name);
                chunk.emit(OpCode::Call(func_idx));
            }

            ASTNode::If {
                condition,
                then_body,
                else_body,
            } => {
                self.compile_expr(chunk, condition)?;
                let else_jump = chunk.emit(OpCode::JumpIfFalse(0));
                chunk.emit(OpCode::Pop);

                chunk.emit(OpCode::PushFrame);
                self.begin_scope();
                self.compile_block(chunk, then_body)?;
                self.end_scope();
                chunk.emit(OpCode::PopFrame);

                if let Some(else_stmts) = else_body {
                    let end_jump = chunk.emit(OpCode::Jump(0));
                    let else_pos = chunk.current_pos() as i32;
                    chunk.patch_jump(else_jump, else_pos);
                    chunk.emit(OpCode::Pop);

                    chunk.emit(OpCode::PushFrame);
                    self.begin_scope();
                    self.compile_block(chunk, else_stmts)?;
                    self.end_scope();
                    chunk.emit(OpCode::PopFrame);

                    let end_pos = chunk.current_pos() as i32;
                    chunk.patch_jump(end_jump, end_pos);
                } else {
                    let end_pos = chunk.current_pos() as i32;
                    chunk.patch_jump(else_jump, end_pos);
                    chunk.emit(OpCode::Pop);
                }
            }

            ASTNode::While { condition, body } => {
                let loop_start = chunk.current_pos() as i32;
                self.compile_expr(chunk, condition)?;
                let exit_jump = chunk.emit(OpCode::JumpIfFalse(0));
                chunk.emit(OpCode::Pop);

                chunk.emit(OpCode::PushFrame);
                self.begin_scope();
                self.compile_block(chunk, body)?;
                self.end_scope();
                chunk.emit(OpCode::PopFrame);

                chunk.emit(OpCode::Jump(loop_start));
                let end_pos = chunk.current_pos() as i32;
                chunk.patch_jump(exit_jump, end_pos);
                chunk.emit(OpCode::Pop);
            }

            ASTNode::DoWhile { body, condition } => {
                let loop_start = chunk.current_pos() as i32;

                chunk.emit(OpCode::PushFrame);
                self.begin_scope();
                self.compile_block(chunk, body)?;
                self.end_scope();
                chunk.emit(OpCode::PopFrame);

                self.compile_expr(chunk, condition)?;
                chunk.emit(OpCode::JumpIfTrue(loop_start));
                chunk.emit(OpCode::Pop);
            }

            ASTNode::For {
                init,
                condition,
                increment,
                body,
            } => {
                chunk.emit(OpCode::PushFrame);
                self.begin_scope();

                if let Some(init_node) = init {
                    self.compile_node(chunk, init_node)?;
                }

                let loop_start = chunk.current_pos() as i32;

                let exit_jump = if let Some(cond) = condition {
                    self.compile_expr(chunk, cond)?;
                    let j = chunk.emit(OpCode::JumpIfFalse(0));
                    chunk.emit(OpCode::Pop);
                    Some(j)
                } else {
                    None
                };

                chunk.emit(OpCode::PushFrame);
                self.begin_scope();
                self.compile_block(chunk, body)?;
                self.end_scope();
                chunk.emit(OpCode::PopFrame);

                if let Some(inc) = increment {
                    self.compile_expr(chunk, inc)?;
                    chunk.emit(OpCode::Pop);
                }

                chunk.emit(OpCode::Jump(loop_start));

                if let Some(ej) = exit_jump {
                    let end_pos = chunk.current_pos() as i32;
                    chunk.patch_jump(ej, end_pos);
                    chunk.emit(OpCode::Pop);
                }

                self.end_scope();
                chunk.emit(OpCode::PopFrame);
            }

            ASTNode::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expr(chunk, e)?;
                } else {
                    chunk.emit(OpCode::Push(LSLValue::Integer(0)));
                }
                chunk.emit(OpCode::Return);
            }

            ASTNode::StateChange(name) => {
                let state_idx = chunk.add_constant(LSLValue::String(name.clone()));
                chunk.emit(OpCode::StateChange(state_idx));
            }

            ASTNode::Label(_) => {
                // Labels are compile-time only, no runtime effect
            }

            ASTNode::Jump(label) => {
                // Jump to label — simplified: emit a Halt (labels are rare in LSL)
                chunk.emit(OpCode::Push(LSLValue::String(label.clone())));
                chunk.emit(OpCode::Pop);
            }

            ASTNode::Block(statements) => {
                chunk.emit(OpCode::PushFrame);
                self.begin_scope();
                self.compile_block(chunk, statements)?;
                self.end_scope();
                chunk.emit(OpCode::PopFrame);
            }

            _ => {}
        }

        Ok(())
    }

    fn compile_expr(&mut self, chunk: &mut Chunk, node: &ASTNode) -> Result<()> {
        self.compile_node(chunk, node)
    }

    fn compile_store(&mut self, chunk: &mut Chunk, target: &ASTNode) -> Result<()> {
        match target {
            ASTNode::Identifier(name) => {
                if let Some(local_idx) = self.resolve_local(name) {
                    chunk.emit(OpCode::StoreLocal(local_idx));
                } else {
                    let global_idx = chunk.add_global(name);
                    chunk.emit(OpCode::StoreGlobal(global_idx));
                }
            }
            ASTNode::MemberAccess { object, member } => {
                if let ASTNode::Identifier(name) = object.as_ref() {
                    let member_id = match member.as_str() {
                        "x" => MEMBER_X,
                        "y" => MEMBER_Y,
                        "z" => MEMBER_Z,
                        "s" => MEMBER_S,
                        _ => return Err(anyhow!("Unknown member: {}", member)),
                    };
                    if let Some(local_idx) = self.resolve_local(name) {
                        chunk.emit(OpCode::LoadLocal(local_idx));
                    } else {
                        let global_idx = chunk.add_global(name);
                        chunk.emit(OpCode::LoadGlobal(global_idx));
                    }
                    // stack: [value_to_set, vec/rot]
                    chunk.emit(OpCode::MemberSet(member_id));
                    // store back
                    if let Some(local_idx) = self.resolve_local(name) {
                        chunk.emit(OpCode::StoreLocal(local_idx));
                    } else {
                        let global_idx = chunk.add_global(name);
                        chunk.emit(OpCode::StoreGlobal(global_idx));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.locals.push(Vec::new());
    }

    fn end_scope(&mut self) {
        if let Some(scope) = self.locals.pop() {
            self.next_local = self.next_local.saturating_sub(scope.len() as u16);
        }
    }

    fn define_local(&mut self, name: &str) -> u16 {
        let idx = self.next_local;
        self.next_local += 1;
        if let Some(scope) = self.locals.last_mut() {
            scope.push((name.to_string(), idx));
        }
        idx
    }

    fn resolve_local(&self, name: &str) -> Option<u16> {
        for scope in self.locals.iter().rev() {
            for (local_name, idx) in scope.iter().rev() {
                if local_name == name {
                    return Some(*idx);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_program() -> Result<()> {
        let ast = ASTNode::Program(vec![
            ASTNode::Variable {
                var_type: "integer".to_string(),
                name: "x".to_string(),
                value: Some(Box::new(ASTNode::Literal(LSLValue::Integer(42)))),
            },
            ASTNode::State {
                name: "default".to_string(),
                body: vec![ASTNode::Event {
                    name: "state_entry".to_string(),
                    parameters: vec![],
                    body: vec![ASTNode::Assignment {
                        target: Box::new(ASTNode::Identifier("x".to_string())),
                        value: Box::new(ASTNode::BinaryOp {
                            left: Box::new(ASTNode::Identifier("x".to_string())),
                            operator: Token::Plus,
                            right: Box::new(ASTNode::Literal(LSLValue::Integer(1))),
                        }),
                    }],
                }],
            },
        ]);

        let mut compiler = BytecodeCompiler::new();
        let program = compiler.compile_program(&ast)?;

        assert_eq!(program.globals.len(), 1);
        assert_eq!(program.globals[0].0, "x");
        assert_eq!(program.states.len(), 1);
        assert_eq!(program.states[0].0, "default");
        assert_eq!(program.states[0].1.len(), 1);
        assert_eq!(program.states[0].1[0].name, "state_entry");
        Ok(())
    }

    #[test]
    fn test_compile_while_loop() -> Result<()> {
        let ast = ASTNode::Program(vec![ASTNode::State {
            name: "default".to_string(),
            body: vec![ASTNode::Event {
                name: "state_entry".to_string(),
                parameters: vec![],
                body: vec![ASTNode::While {
                    condition: Box::new(ASTNode::BinaryOp {
                        left: Box::new(ASTNode::Identifier("i".to_string())),
                        operator: Token::Less,
                        right: Box::new(ASTNode::Literal(LSLValue::Integer(5))),
                    }),
                    body: vec![ASTNode::PostIncrement(Box::new(ASTNode::Identifier(
                        "i".to_string(),
                    )))],
                }],
            }],
        }]);

        let mut compiler = BytecodeCompiler::new();
        let program = compiler.compile_program(&ast)?;
        assert!(!program.states[0].1[0].chunk.code.is_empty());
        Ok(())
    }
}
