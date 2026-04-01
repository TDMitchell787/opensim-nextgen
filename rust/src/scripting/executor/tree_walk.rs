use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use uuid::Uuid;

use super::{
    CompiledScript, EventHandler, ExecutionResult, ScriptExecutor, ScriptInstance,
    StateDefinition, UserFunction,
};
use crate::scripting::lsl_interpreter::{ASTNode, LSLLexer, LSLParser, Token};
use crate::scripting::{LSLValue, LSLVector, LSLRotation};
use crate::scripting::lsl_constants::build_constant_map;

pub static LSL_CONSTANTS: std::sync::LazyLock<HashMap<String, LSLValue>> = std::sync::LazyLock::new(|| {
    build_constant_map().into_iter().map(|(k, v)| (k.to_string(), v)).collect()
});

const MAX_LOOP_ITERATIONS: u32 = 65536;
const MAX_CALL_DEPTH: u32 = 128;

enum ControlFlow {
    Normal(LSLValue),
    Return(LSLValue),
    StateChange(String),
    Jump(String),
}

pub struct TreeWalkExecutor;

impl TreeWalkExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_script_parts(ast: &ASTNode) -> Result<(
        Vec<(String, String, Option<ASTNode>)>,
        HashMap<String, UserFunction>,
        HashMap<String, StateDefinition>,
    )> {
        let statements = match ast {
            ASTNode::Program(stmts) => stmts,
            _ => return Err(anyhow!("Expected Program node")),
        };

        let mut globals = Vec::new();
        let mut functions = HashMap::new();
        let mut states = HashMap::new();

        for node in statements {
            match node {
                ASTNode::Variable { var_type, name, value } => {
                    globals.push((name.clone(), var_type.clone(), value.as_ref().map(|v| *v.clone())));
                }
                ASTNode::Function { name, return_type, parameters, body } => {
                    functions.insert(name.clone(), UserFunction {
                        name: name.clone(),
                        return_type: return_type.clone(),
                        parameters: parameters.clone(),
                        body: body.clone(),
                    });
                }
                ASTNode::State { name, body } => {
                    let mut events = HashMap::new();
                    for event_node in body {
                        if let ASTNode::Event { name: event_name, parameters, body: event_body } = event_node {
                            events.insert(event_name.clone(), EventHandler {
                                name: event_name.clone(),
                                parameters: parameters.clone(),
                                body: event_body.clone(),
                            });
                        }
                    }
                    states.insert(name.clone(), StateDefinition {
                        name: name.clone(),
                        events,
                    });
                }
                _ => {}
            }
        }

        Ok((globals, functions, states))
    }
}

impl ScriptExecutor for TreeWalkExecutor {
    fn name(&self) -> &'static str {
        "TreeWalk"
    }

    fn compile(&self, source: &str, script_id: Uuid) -> Result<CompiledScript> {
        let trimmed = source.trim();
        let effective = if trimmed.starts_with('{') {
            format!("default {}", trimmed)
        } else {
            source.to_string()
        };
        let mut lexer = LSLLexer::new(effective);
        let tokens = lexer.tokenize()?;
        let mut parser = LSLParser::new(tokens);
        let ast = parser.parse()?;

        let (globals, functions, states) = Self::extract_script_parts(&ast)?;

        Ok(CompiledScript {
            script_id,
            globals,
            functions,
            states,
            ast,
        })
    }

    fn execute_event(
        &self,
        instance: &mut ScriptInstance,
        event: &str,
        args: &[LSLValue],
    ) -> Result<ExecutionResult> {
        let state_def = instance.compiled.states.get(&instance.current_state)
            .ok_or_else(|| anyhow!("State '{}' not found", instance.current_state))?
            .clone();

        let handler = match state_def.events.get(event) {
            Some(h) => h.clone(),
            None => return Ok(ExecutionResult::Complete(LSLValue::Integer(0))),
        };

        let mut env = Environment::new();

        env.push_scope();
        for (key, val) in &instance.global_vars {
            env.set(key.clone(), val.clone());
        }

        env.push_scope();
        for (i, (param_name, param_type)) in handler.parameters.iter().enumerate() {
            let val = args.get(i)
                .cloned()
                .unwrap_or_else(|| LSLValue::type_default(param_type));
            env.set(param_name.clone(), val);
        }

        let functions = instance.compiled.functions.clone();
        instance.pending_actions.clear();
        let mut ctx = ExecContext {
            env,
            functions,
            call_depth: 0,
            instructions: 0,
            actions: Vec::new(),
            ctx: instance.context.clone(),
            script_id: instance.script_id,
        };

        let result = ctx.execute_block(&handler.body);

        instance.pending_actions = ctx.actions;

        let global_scope = ctx.env.scopes.first().cloned().unwrap_or_default();
        for (key, val) in global_scope {
            instance.global_vars.insert(key, val);
        }

        match result {
            Ok(ControlFlow::Normal(v)) | Ok(ControlFlow::Return(v)) => {
                Ok(ExecutionResult::Complete(v))
            }
            Ok(ControlFlow::StateChange(name)) => {
                Ok(ExecutionResult::StateChange(name))
            }
            Ok(ControlFlow::Jump(_)) => {
                Ok(ExecutionResult::Complete(LSLValue::Integer(0)))
            }
            Err(e) => Ok(ExecutionResult::Error(e.to_string())),
        }
    }
}

struct Environment {
    scopes: Vec<HashMap<String, LSLValue>>,
}

impl Environment {
    fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn get(&self, name: &str) -> Option<LSLValue> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }

    fn set(&mut self, name: String, value: LSLValue) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name) {
                scope.insert(name, value);
                return;
            }
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    fn define(&mut self, name: String, value: LSLValue) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }
}

struct ExecContext {
    env: Environment,
    functions: HashMap<String, UserFunction>,
    call_depth: u32,
    instructions: u64,
    actions: Vec<super::ScriptAction>,
    ctx: super::ObjectContext,
    script_id: Uuid,
}

impl ExecContext {
    fn execute_block(&mut self, statements: &[ASTNode]) -> Result<ControlFlow> {
        let mut result = LSLValue::Integer(0);
        for stmt in statements {
            match self.execute_node(stmt)? {
                ControlFlow::Normal(v) => { result = v; }
                flow => return Ok(flow),
            }
        }
        Ok(ControlFlow::Normal(result))
    }

    fn execute_node(&mut self, node: &ASTNode) -> Result<ControlFlow> {
        self.instructions += 1;

        match node {
            ASTNode::Literal(val) => Ok(ControlFlow::Normal(val.clone())),

            ASTNode::Identifier(name) => {
                let val = self.env.get(name)
                    .or_else(|| LSL_CONSTANTS.get(name).cloned())
                    .ok_or_else(|| anyhow!("Undefined variable: {}", name))?;
                Ok(ControlFlow::Normal(val))
            }

            ASTNode::Variable { var_type, name, value } => {
                let val = if let Some(expr) = value {
                    self.eval(expr)?
                } else {
                    LSLValue::type_default(var_type)
                };
                self.env.define(name.clone(), val);
                Ok(ControlFlow::Normal(LSLValue::Integer(0)))
            }

            ASTNode::Assignment { target, value } => {
                let val = self.eval(value)?;
                self.assign_to(target, val.clone())?;
                Ok(ControlFlow::Normal(val))
            }

            ASTNode::CompoundAssignment { target, operator, value } => {
                let current = self.eval(target)?;
                let rhs = self.eval(value)?;
                let result = match operator {
                    Token::PlusAssign => self.binary_op(&current, &Token::Plus, &rhs)?,
                    Token::MinusAssign => self.binary_op(&current, &Token::Minus, &rhs)?,
                    Token::MultiplyAssign => self.binary_op(&current, &Token::Multiply, &rhs)?,
                    Token::DivideAssign => self.binary_op(&current, &Token::Divide, &rhs)?,
                    Token::ModuloAssign => self.binary_op(&current, &Token::Modulo, &rhs)?,
                    _ => return Err(anyhow!("Invalid compound assignment operator")),
                };
                self.assign_to(target, result.clone())?;
                Ok(ControlFlow::Normal(result))
            }

            ASTNode::BinaryOp { left, operator, right } => {
                let lv = self.eval(left)?;
                let rv = self.eval(right)?;
                let result = self.binary_op(&lv, operator, &rv)?;
                Ok(ControlFlow::Normal(result))
            }

            ASTNode::UnaryOp { operator, operand } => {
                let val = self.eval(operand)?;
                let result = self.unary_op(operator, &val)?;
                Ok(ControlFlow::Normal(result))
            }

            ASTNode::PreIncrement(expr) => {
                let val = self.eval(expr)?;
                let result = self.binary_op(&val, &Token::Plus, &LSLValue::Integer(1))?;
                self.assign_to(expr, result.clone())?;
                Ok(ControlFlow::Normal(result))
            }

            ASTNode::PreDecrement(expr) => {
                let val = self.eval(expr)?;
                let result = self.binary_op(&val, &Token::Minus, &LSLValue::Integer(1))?;
                self.assign_to(expr, result.clone())?;
                Ok(ControlFlow::Normal(result))
            }

            ASTNode::PostIncrement(expr) => {
                let val = self.eval(expr)?;
                let result = self.binary_op(&val, &Token::Plus, &LSLValue::Integer(1))?;
                self.assign_to(expr, result)?;
                Ok(ControlFlow::Normal(val))
            }

            ASTNode::PostDecrement(expr) => {
                let val = self.eval(expr)?;
                let result = self.binary_op(&val, &Token::Minus, &LSLValue::Integer(1))?;
                self.assign_to(expr, result)?;
                Ok(ControlFlow::Normal(val))
            }

            ASTNode::TypeCast { target_type, expr } => {
                let val = self.eval(expr)?;
                Ok(ControlFlow::Normal(val.coerce(target_type)))
            }

            ASTNode::MemberAccess { object, member } => {
                let val = self.eval(object)?;
                let result = self.access_member(&val, member)?;
                Ok(ControlFlow::Normal(result))
            }

            ASTNode::VectorLiteral { x, y, z } => {
                let xv = self.eval(x)?.to_float();
                let yv = self.eval(y)?.to_float();
                let zv = self.eval(z)?.to_float();
                Ok(ControlFlow::Normal(LSLValue::Vector(LSLVector { x: xv, y: yv, z: zv })))
            }

            ASTNode::RotationLiteral { x, y, z, s } => {
                let xv = self.eval(x)?.to_float();
                let yv = self.eval(y)?.to_float();
                let zv = self.eval(z)?.to_float();
                let sv = self.eval(s)?.to_float();
                Ok(ControlFlow::Normal(LSLValue::Rotation(LSLRotation { x: xv, y: yv, z: zv, s: sv })))
            }

            ASTNode::ListLiteral(elements) => {
                let mut list = Vec::new();
                for elem in elements {
                    list.push(self.eval(elem)?);
                }
                Ok(ControlFlow::Normal(LSLValue::List(list)))
            }

            ASTNode::FunctionCall { name, arguments } => {
                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.eval(arg)?);
                }
                let result = self.call_function(name, &args)?;
                Ok(ControlFlow::Normal(result))
            }

            ASTNode::If { condition, then_body, else_body } => {
                let cond = self.eval(condition)?;
                if cond.is_true() {
                    self.env.push_scope();
                    let result = self.execute_block(then_body);
                    self.env.pop_scope();
                    result
                } else if let Some(else_stmts) = else_body {
                    self.env.push_scope();
                    let result = self.execute_block(else_stmts);
                    self.env.pop_scope();
                    result
                } else {
                    Ok(ControlFlow::Normal(LSLValue::Integer(0)))
                }
            }

            ASTNode::While { condition, body } => {
                let mut iterations = 0u32;
                loop {
                    let cond = self.eval(condition)?;
                    if !cond.is_true() { break; }

                    iterations += 1;
                    if iterations > MAX_LOOP_ITERATIONS {
                        return Err(anyhow!("Loop iteration limit exceeded"));
                    }

                    self.env.push_scope();
                    match self.execute_block(body)? {
                        ControlFlow::Normal(_) => {}
                        flow => { self.env.pop_scope(); return Ok(flow); }
                    }
                    self.env.pop_scope();
                }
                Ok(ControlFlow::Normal(LSLValue::Integer(0)))
            }

            ASTNode::DoWhile { body, condition } => {
                let mut iterations = 0u32;
                loop {
                    iterations += 1;
                    if iterations > MAX_LOOP_ITERATIONS {
                        return Err(anyhow!("Loop iteration limit exceeded"));
                    }

                    self.env.push_scope();
                    match self.execute_block(body)? {
                        ControlFlow::Normal(_) => {}
                        flow => { self.env.pop_scope(); return Ok(flow); }
                    }
                    self.env.pop_scope();

                    let cond = self.eval(condition)?;
                    if !cond.is_true() { break; }
                }
                Ok(ControlFlow::Normal(LSLValue::Integer(0)))
            }

            ASTNode::For { init, condition, increment, body } => {
                self.env.push_scope();

                if let Some(init_expr) = init {
                    self.execute_node(init_expr)?;
                }

                let mut iterations = 0u32;
                loop {
                    if let Some(cond) = condition {
                        let cv = self.eval(cond)?;
                        if !cv.is_true() { break; }
                    }

                    iterations += 1;
                    if iterations > MAX_LOOP_ITERATIONS {
                        self.env.pop_scope();
                        return Err(anyhow!("Loop iteration limit exceeded"));
                    }

                    self.env.push_scope();
                    match self.execute_block(body)? {
                        ControlFlow::Normal(_) => {}
                        flow => {
                            self.env.pop_scope();
                            self.env.pop_scope();
                            return Ok(flow);
                        }
                    }
                    self.env.pop_scope();

                    if let Some(inc) = increment {
                        self.eval(inc)?;
                    }
                }

                self.env.pop_scope();
                Ok(ControlFlow::Normal(LSLValue::Integer(0)))
            }

            ASTNode::Return(expr) => {
                let val = if let Some(e) = expr {
                    self.eval(e)?
                } else {
                    LSLValue::Integer(0)
                };
                Ok(ControlFlow::Return(val))
            }

            ASTNode::StateChange(name) => {
                Ok(ControlFlow::StateChange(name.clone()))
            }

            ASTNode::Label(_) => {
                Ok(ControlFlow::Normal(LSLValue::Integer(0)))
            }

            ASTNode::Jump(label) => {
                Ok(ControlFlow::Jump(label.clone()))
            }

            ASTNode::Block(statements) => {
                self.env.push_scope();
                let result = self.execute_block(statements);
                self.env.pop_scope();
                result
            }

            _ => Ok(ControlFlow::Normal(LSLValue::Integer(0))),
        }
    }

    fn eval(&mut self, node: &ASTNode) -> Result<LSLValue> {
        match self.execute_node(node)? {
            ControlFlow::Normal(v) => Ok(v),
            ControlFlow::Return(v) => Ok(v),
            ControlFlow::StateChange(s) => Err(anyhow!("Unexpected state change to '{}'", s)),
            ControlFlow::Jump(l) => Err(anyhow!("Unhandled jump to '{}'", l)),
        }
    }

    fn assign_to(&mut self, target: &ASTNode, value: LSLValue) -> Result<()> {
        match target {
            ASTNode::Identifier(name) => {
                self.env.set(name.clone(), value);
                Ok(())
            }
            ASTNode::MemberAccess { object, member } => {
                if let ASTNode::Identifier(name) = object.as_ref() {
                    let mut val = self.env.get(name)
                        .ok_or_else(|| anyhow!("Undefined variable: {}", name))?;
                    self.set_member(&mut val, member, value.to_float())?;
                    self.env.set(name.clone(), val);
                    Ok(())
                } else {
                    Err(anyhow!("Cannot assign to computed member access"))
                }
            }
            _ => Err(anyhow!("Invalid assignment target")),
        }
    }

    fn access_member(&self, val: &LSLValue, member: &str) -> Result<LSLValue> {
        match val {
            LSLValue::Vector(v) => match member {
                "x" => Ok(LSLValue::Float(v.x)),
                "y" => Ok(LSLValue::Float(v.y)),
                "z" => Ok(LSLValue::Float(v.z)),
                _ => Err(anyhow!("Vector has no member '{}'", member)),
            },
            LSLValue::Rotation(r) => match member {
                "x" => Ok(LSLValue::Float(r.x)),
                "y" => Ok(LSLValue::Float(r.y)),
                "z" => Ok(LSLValue::Float(r.z)),
                "s" => Ok(LSLValue::Float(r.s)),
                _ => Err(anyhow!("Rotation has no member '{}'", member)),
            },
            _ => Err(anyhow!("Type has no members")),
        }
    }

    fn set_member(&self, val: &mut LSLValue, member: &str, new_val: f32) -> Result<()> {
        match val {
            LSLValue::Vector(v) => match member {
                "x" => { v.x = new_val; Ok(()) }
                "y" => { v.y = new_val; Ok(()) }
                "z" => { v.z = new_val; Ok(()) }
                _ => Err(anyhow!("Vector has no member '{}'", member)),
            },
            LSLValue::Rotation(r) => match member {
                "x" => { r.x = new_val; Ok(()) }
                "y" => { r.y = new_val; Ok(()) }
                "z" => { r.z = new_val; Ok(()) }
                "s" => { r.s = new_val; Ok(()) }
                _ => Err(anyhow!("Rotation has no member '{}'", member)),
            },
            _ => Err(anyhow!("Type has no members")),
        }
    }

    fn call_function(&mut self, name: &str, args: &[LSLValue]) -> Result<LSLValue> {
        self.call_depth += 1;
        if self.call_depth > MAX_CALL_DEPTH {
            self.call_depth -= 1;
            return Err(anyhow!("Call depth limit exceeded"));
        }

        let func = self.functions.get(name).cloned();

        let result = if let Some(func) = func {
            self.env.push_scope();
            for (i, (param_name, param_type)) in func.parameters.iter().enumerate() {
                let val = args.get(i)
                    .cloned()
                    .unwrap_or_else(|| LSLValue::type_default(param_type));
                self.env.define(param_name.clone(), val);
            }

            let result = self.execute_block(&func.body);
            self.env.pop_scope();

            match result? {
                ControlFlow::Normal(v) | ControlFlow::Return(v) => Ok(v),
                ControlFlow::StateChange(s) => Err(anyhow!("State change in function not allowed")),
                ControlFlow::Jump(l) => Err(anyhow!("Jump out of function not allowed")),
            }
        } else if name.starts_with("ll") || name.starts_with("os") {
            self.call_builtin(name, args)
        } else {
            Ok(LSLValue::Integer(0))
        };

        self.call_depth -= 1;
        result
    }

    fn call_builtin(&mut self, name: &str, args: &[LSLValue]) -> Result<LSLValue> {
        use super::ScriptAction;
        match name {
            // ==================== CHAT ====================
            "llSay" => {
                let ch = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let msg = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::Say { channel: ch, message: msg, object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llShout" => {
                let ch = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let msg = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::Shout { channel: ch, message: msg, object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llWhisper" => {
                let ch = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let msg = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::Whisper { channel: ch, message: msg, object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llOwnerSay" => {
                let msg = args.first().map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::OwnerSay { message: msg, object_id: self.ctx.object_id, owner_id: self.ctx.owner_id });
                Ok(LSLValue::Integer(0))
            }
            "llRegionSay" => {
                let ch = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let msg = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                if ch == 0 { return Ok(LSLValue::Integer(0)); }
                self.actions.push(ScriptAction::RegionSay { channel: ch, message: msg, object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llRegionSayTo" => {
                let target = args.first().map(|a| a.to_key()).unwrap_or_default();
                let ch = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let msg = args.get(2).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::RegionSayTo { target_id: target, channel: ch, message: msg, object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llInstantMessage" => {
                let target = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let msg = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::InstantMessage {
                    target_id: target, message: msg,
                    object_id: self.ctx.object_id, object_name: self.ctx.object_name.clone(),
                });
                Ok(LSLValue::Integer(0))
            }

            // ==================== TIMERS & LISTENERS ====================
            "llSetTimerEvent" => {
                let interval = args.first().map(|a| a.to_float() as f64).unwrap_or(0.0);
                self.actions.push(ScriptAction::SetTimerEvent { interval });
                Ok(LSLValue::Integer(0))
            }
            "llListen" => {
                let ch = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let nm = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                let id = args.get(2).map(|a| a.to_string()).unwrap_or_default();
                let msg = args.get(3).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::Listen { channel: ch, name: nm, id, msg });
                Ok(LSLValue::Integer(1))
            }
            "llListenRemove" => {
                let handle = args.first().map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::ListenRemove { handle });
                Ok(LSLValue::Integer(0))
            }
            "llListenControl" => {
                Ok(LSLValue::Integer(0))
            }

            // ==================== CONTEXT-AWARE QUERIES ====================
            "llGetPos" | "llGetLocalPos" => Ok(LSLValue::Vector(self.ctx.position)),
            "llGetRot" | "llGetLocalRot" => Ok(LSLValue::Rotation(self.ctx.rotation)),
            "llGetScale" => Ok(LSLValue::Vector(self.ctx.scale)),
            "llGetOwner" => Ok(LSLValue::Key(self.ctx.owner_id)),
            "llGetKey" => Ok(LSLValue::Key(self.ctx.object_id)),
            "llGetObjectName" => Ok(LSLValue::String(self.ctx.object_name.clone())),
            "llGetRegionName" => Ok(LSLValue::String(self.ctx.region_name.clone())),
            "llGetScriptName" => Ok(LSLValue::String("Script".to_string())),
            "llGetCreator" => Ok(LSLValue::Key(self.ctx.owner_id)),
            "llGetNumberOfPrims" => Ok(LSLValue::Integer(self.ctx.link_count.max(1))),
            "llGetLinkNumber" => Ok(LSLValue::Integer(self.ctx.link_num)),
            "llGetObjectDesc" => Ok(LSLValue::String(String::new())),

            // ==================== DETECTED PARAMS ====================
            "llDetectedKey" => {
                let idx = args.first().map(|a| a.to_integer()).unwrap_or(0) as usize;
                Ok(LSLValue::Key(self.ctx.detect_params.get(idx).map(|d| d.key).unwrap_or(Uuid::nil())))
            }
            "llDetectedName" => {
                let idx = args.first().map(|a| a.to_integer()).unwrap_or(0) as usize;
                Ok(LSLValue::String(self.ctx.detect_params.get(idx).map(|d| d.name.clone()).unwrap_or_default()))
            }
            "llDetectedOwner" => {
                let idx = args.first().map(|a| a.to_integer()).unwrap_or(0) as usize;
                Ok(LSLValue::Key(self.ctx.detect_params.get(idx).map(|d| d.owner).unwrap_or(Uuid::nil())))
            }
            "llDetectedPos" => {
                let idx = args.first().map(|a| a.to_integer()).unwrap_or(0) as usize;
                Ok(LSLValue::Vector(self.ctx.detect_params.get(idx).map(|d| d.position).unwrap_or(LSLVector::zero())))
            }
            "llDetectedRot" => {
                let idx = args.first().map(|a| a.to_integer()).unwrap_or(0) as usize;
                Ok(LSLValue::Rotation(self.ctx.detect_params.get(idx).map(|d| d.rotation).unwrap_or(LSLRotation::identity())))
            }
            "llDetectedVel" => {
                let idx = args.first().map(|a| a.to_integer()).unwrap_or(0) as usize;
                Ok(LSLValue::Vector(self.ctx.detect_params.get(idx).map(|d| d.velocity).unwrap_or(LSLVector::zero())))
            }
            "llDetectedType" => {
                let idx = args.first().map(|a| a.to_integer()).unwrap_or(0) as usize;
                Ok(LSLValue::Integer(self.ctx.detect_params.get(idx).map(|d| d.det_type).unwrap_or(0)))
            }
            "llDetectedLinkNumber" => {
                let idx = args.first().map(|a| a.to_integer()).unwrap_or(0) as usize;
                Ok(LSLValue::Integer(self.ctx.detect_params.get(idx).map(|d| d.link_num).unwrap_or(0)))
            }
            "llDetectedGroup" | "llDetectedGrab" | "llDetectedTouchNormal"
            | "llDetectedTouchBinormal" | "llDetectedTouchPos" => {
                Ok(LSLValue::Vector(LSLVector::zero()))
            }
            "llDetectedTouchUV" | "llDetectedTouchST" => {
                Ok(LSLValue::Vector(LSLVector::zero()))
            }
            "llDetectedTouchFace" => Ok(LSLValue::Integer(-1)),

            // ==================== SCRIPT CONTROL ====================
            "llResetScript" => {
                self.actions.push(ScriptAction::ResetScript);
                Ok(LSLValue::Integer(0))
            }
            "llSleep" => {
                let secs = args.first().map(|a| a.to_float() as f64).unwrap_or(0.0);
                self.actions.push(ScriptAction::Sleep { seconds: secs });
                Ok(LSLValue::Integer(0))
            }
            "llGetTime" => Ok(LSLValue::Float(0.0)),
            "llResetTime" => Ok(LSLValue::Integer(0)),
            "llGetEnergy" => Ok(LSLValue::Float(1.0)),
            "llGetFreeMemory" | "llGetUsedMemory" => Ok(LSLValue::Integer(65536)),
            "llGetMemoryLimit" => Ok(LSLValue::Integer(65536)),
            "llGetFreeURLs" => Ok(LSLValue::Integer(10)),

            // ==================== DIALOG ====================
            "llDialog" => {
                let avatar = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let msg = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                let buttons = args.get(2).map(|a| {
                    if let LSLValue::List(l) = a { l.iter().map(|v| v.to_string()).collect() } else { vec![] }
                }).unwrap_or_else(|| vec!["OK".to_string()]);
                let ch = args.get(3).map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::Dialog {
                    avatar_id: avatar, object_name: self.ctx.object_name.clone(),
                    message: msg, buttons, channel: ch, object_id: self.ctx.object_id,
                });
                Ok(LSLValue::Integer(0))
            }

            // ==================== LINK MESSAGES ====================
            "llMessageLinked" => {
                let link = args.first().map(|a| a.to_integer()).unwrap_or(-1);
                let num = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let str_val = args.get(2).map(|a| a.to_string()).unwrap_or_default();
                let id = args.get(3).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::MessageLinked { link_num: link, num, str_val, id });
                Ok(LSLValue::Integer(0))
            }

            // ==================== STRING FUNCTIONS ====================
            "llStringLength" => {
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                Ok(LSLValue::Integer(s.len() as i32))
            }
            "llGetSubString" => {
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                let start = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let end = args.get(2).map(|a| a.to_integer()).unwrap_or(-1);
                let len = s.len() as i32;
                if len == 0 { return Ok(LSLValue::String(String::new())); }
                let s_idx = if start < 0 { (len + start).max(0) as usize } else { (start as usize).min(len as usize) };
                let e_idx = if end < 0 { (len + end).max(0) as usize } else { (end as usize).min(len as usize - 1) };
                if s_idx <= e_idx {
                    Ok(LSLValue::String(s.chars().skip(s_idx).take(e_idx - s_idx + 1).collect()))
                } else {
                    let head: String = s.chars().take(e_idx + 1).collect();
                    let tail: String = s.chars().skip(s_idx).collect();
                    Ok(LSLValue::String(format!("{}{}", tail, head)))
                }
            }
            "llDeleteSubString" => {
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                let start = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let end = args.get(2).map(|a| a.to_integer()).unwrap_or(-1);
                let len = s.len() as i32;
                if len == 0 { return Ok(LSLValue::String(String::new())); }
                let s_idx = if start < 0 { (len + start).max(0) as usize } else { (start as usize).min(len as usize) };
                let e_idx = if end < 0 { (len + end).max(0) as usize } else { (end as usize).min(len as usize - 1) };
                if s_idx <= e_idx {
                    let head: String = s.chars().take(s_idx).collect();
                    let tail: String = s.chars().skip(e_idx + 1).collect();
                    Ok(LSLValue::String(format!("{}{}", head, tail)))
                } else {
                    Ok(LSLValue::String(s.chars().skip(e_idx + 1).take(s_idx - e_idx - 1).collect()))
                }
            }
            "llSubStringIndex" => {
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                let pat = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                Ok(LSLValue::Integer(s.find(&pat).map(|i| i as i32).unwrap_or(-1)))
            }
            "llToUpper" => {
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                Ok(LSLValue::String(s.to_uppercase()))
            }
            "llToLower" => {
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                Ok(LSLValue::String(s.to_lowercase()))
            }
            "llStringTrim" => {
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                let mode = args.get(1).map(|a| a.to_integer()).unwrap_or(3);
                Ok(LSLValue::String(match mode {
                    1 => s.trim_start().to_string(),
                    2 => s.trim_end().to_string(),
                    _ => s.trim().to_string(),
                }))
            }
            "llStringToBase64" => {
                use base64::Engine;
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                Ok(LSLValue::String(base64::engine::general_purpose::STANDARD.encode(s.as_bytes())))
            }
            "llBase64ToString" => {
                use base64::Engine;
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                let decoded = base64::engine::general_purpose::STANDARD.decode(s.as_bytes()).unwrap_or_default();
                Ok(LSLValue::String(String::from_utf8_lossy(&decoded).to_string()))
            }
            "llInsertString" => {
                let dst = args.first().map(|a| a.to_string()).unwrap_or_default();
                let pos = args.get(1).map(|a| a.to_integer()).unwrap_or(0).max(0) as usize;
                let src = args.get(2).map(|a| a.to_string()).unwrap_or_default();
                let pos = pos.min(dst.len());
                Ok(LSLValue::String(format!("{}{}{}", &dst[..pos], src, &dst[pos..])))
            }
            "llGetNotecardLine" | "llGetNumberOfNotecardLines" => {
                Ok(LSLValue::Key(Uuid::nil()))
            }

            // ==================== LIST FUNCTIONS ====================
            "llGetListLength" => {
                if let Some(LSLValue::List(l)) = args.first() {
                    Ok(LSLValue::Integer(l.len() as i32))
                } else {
                    Ok(LSLValue::Integer(0))
                }
            }
            "llList2String" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let idx = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let i = if idx < 0 { (list.len() as i32 + idx).max(0) as usize } else { idx as usize };
                Ok(LSLValue::String(list.get(i).map(|v| v.to_string()).unwrap_or_default()))
            }
            "llList2Integer" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let idx = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let i = if idx < 0 { (list.len() as i32 + idx).max(0) as usize } else { idx as usize };
                Ok(LSLValue::Integer(list.get(i).map(|v| v.to_integer()).unwrap_or(0)))
            }
            "llList2Float" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let idx = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let i = if idx < 0 { (list.len() as i32 + idx).max(0) as usize } else { idx as usize };
                Ok(LSLValue::Float(list.get(i).map(|v| v.to_float()).unwrap_or(0.0)))
            }
            "llList2Key" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let idx = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let i = if idx < 0 { (list.len() as i32 + idx).max(0) as usize } else { idx as usize };
                Ok(LSLValue::Key(list.get(i).map(|v| v.to_key()).unwrap_or(Uuid::nil())))
            }
            "llList2Vector" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let idx = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let i = if idx < 0 { (list.len() as i32 + idx).max(0) as usize } else { idx as usize };
                Ok(LSLValue::Vector(list.get(i).map(|v| v.to_vector()).unwrap_or(LSLVector::zero())))
            }
            "llList2Rot" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let idx = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let i = if idx < 0 { (list.len() as i32 + idx).max(0) as usize } else { idx as usize };
                Ok(LSLValue::Rotation(list.get(i).map(|v| v.to_rotation()).unwrap_or(LSLRotation::identity())))
            }
            "llDeleteSubList" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let start = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let end = args.get(2).map(|a| a.to_integer()).unwrap_or(-1);
                let len = list.len() as i32;
                if len == 0 { return Ok(LSLValue::List(vec![])); }
                let s = if start < 0 { (len + start).max(0) as usize } else { (start as usize).min(len as usize) };
                let e = if end < 0 { (len + end).max(0) as usize } else { (end as usize).min(len as usize - 1) };
                if s <= e {
                    let mut r = list[..s].to_vec();
                    if e + 1 < list.len() { r.extend_from_slice(&list[e + 1..]); }
                    Ok(LSLValue::List(r))
                } else {
                    Ok(LSLValue::List(list[e + 1..s].to_vec()))
                }
            }
            "llListInsertList" => {
                let dst = args.first().map(|a| a.to_list()).unwrap_or_default();
                let src = args.get(1).map(|a| a.to_list()).unwrap_or_default();
                let pos = args.get(2).map(|a| a.to_integer()).unwrap_or(0).max(0) as usize;
                let pos = pos.min(dst.len());
                let mut r = dst[..pos].to_vec();
                r.extend(src);
                r.extend_from_slice(&dst[pos..]);
                Ok(LSLValue::List(r))
            }
            "llListSort" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let stride = args.get(1).map(|a| a.to_integer()).unwrap_or(1).max(1) as usize;
                let ascending = args.get(2).map(|a| a.to_integer()).unwrap_or(1) != 0;
                if stride >= list.len() || stride == 0 {
                    return Ok(LSLValue::List(list));
                }
                let mut chunks: Vec<Vec<LSLValue>> = list.chunks(stride).map(|c| c.to_vec()).collect();
                chunks.sort_by(|a, b| {
                    let cmp = a.first().map(|v| v.to_string()).unwrap_or_default()
                        .cmp(&b.first().map(|v| v.to_string()).unwrap_or_default());
                    if ascending { cmp } else { cmp.reverse() }
                });
                Ok(LSLValue::List(chunks.into_iter().flatten().collect()))
            }
            "llListReplaceList" => {
                let dst = args.first().map(|a| a.to_list()).unwrap_or_default();
                let src = args.get(1).map(|a| a.to_list()).unwrap_or_default();
                let start = args.get(2).map(|a| a.to_integer()).unwrap_or(0);
                let end = args.get(3).map(|a| a.to_integer()).unwrap_or(-1);
                let len = dst.len() as i32;
                let s = if start < 0 { (len + start).max(0) as usize } else { (start as usize).min(len as usize) };
                let e = if end < 0 { (len + end).max(0) as usize } else { (end as usize).min(if len > 0 { len as usize - 1 } else { 0 }) };
                let mut r = dst[..s].to_vec();
                r.extend(src);
                if e + 1 < dst.len() { r.extend_from_slice(&dst[e + 1..]); }
                Ok(LSLValue::List(r))
            }
            "llListFindList" => {
                let haystack = args.first().map(|a| a.to_list()).unwrap_or_default();
                let needle = args.get(1).map(|a| a.to_list()).unwrap_or_default();
                if needle.is_empty() || needle.len() > haystack.len() {
                    return Ok(LSLValue::Integer(-1));
                }
                for i in 0..=(haystack.len() - needle.len()) {
                    let mut found = true;
                    for j in 0..needle.len() {
                        if haystack[i + j].to_string() != needle[j].to_string() {
                            found = false;
                            break;
                        }
                    }
                    if found { return Ok(LSLValue::Integer(i as i32)); }
                }
                Ok(LSLValue::Integer(-1))
            }
            "llGetListEntryType" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let idx = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let i = if idx < 0 { (list.len() as i32 + idx).max(0) as usize } else { idx as usize };
                Ok(LSLValue::Integer(list.get(i).map(|v| match v {
                    LSLValue::Integer(_) => 1,
                    LSLValue::Float(_) => 2,
                    LSLValue::String(_) => 3,
                    LSLValue::Key(_) => 4,
                    LSLValue::Vector(_) => 5,
                    LSLValue::Rotation(_) => 6,
                    LSLValue::List(_) => 0,
                }).unwrap_or(0)))
            }
            "llDumpList2String" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let sep = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                let strs: Vec<String> = list.iter().map(|v| v.to_string()).collect();
                Ok(LSLValue::String(strs.join(&sep)))
            }
            "llParseString2List" => {
                let src = args.first().map(|a| a.to_string()).unwrap_or_default();
                let separators = args.get(1).map(|a| a.to_list()).unwrap_or_default();
                let _spacers = args.get(2).map(|a| a.to_list()).unwrap_or_default();
                let seps: Vec<String> = separators.iter().map(|v| v.to_string()).collect();
                if seps.is_empty() {
                    return Ok(LSLValue::List(vec![LSLValue::String(src)]));
                }
                let mut result = Vec::new();
                let mut remaining = src.as_str();
                while !remaining.is_empty() {
                    let mut earliest_pos = remaining.len();
                    let mut earliest_len = 0;
                    for sep in &seps {
                        if sep.is_empty() { continue; }
                        if let Some(pos) = remaining.find(sep.as_str()) {
                            if pos < earliest_pos {
                                earliest_pos = pos;
                                earliest_len = sep.len();
                            }
                        }
                    }
                    if earliest_pos == remaining.len() {
                        if !remaining.is_empty() { result.push(LSLValue::String(remaining.to_string())); }
                        break;
                    }
                    if earliest_pos > 0 {
                        result.push(LSLValue::String(remaining[..earliest_pos].to_string()));
                    }
                    remaining = &remaining[earliest_pos + earliest_len..];
                }
                Ok(LSLValue::List(result))
            }
            "llCSV2List" => {
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                let items: Vec<LSLValue> = s.split(',').map(|p| LSLValue::String(p.trim().to_string())).collect();
                Ok(LSLValue::List(items))
            }
            "llList2CSV" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let strs: Vec<String> = list.iter().map(|v| v.to_string()).collect();
                Ok(LSLValue::String(strs.join(", ")))
            }
            "llList2ListStrided" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let start = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let end = args.get(2).map(|a| a.to_integer()).unwrap_or(-1);
                let stride = args.get(3).map(|a| a.to_integer()).unwrap_or(1).max(1) as usize;
                let len = list.len() as i32;
                let s = if start < 0 { (len + start).max(0) as usize } else { (start as usize).min(len as usize) };
                let e = if end < 0 { (len + end).max(0) as usize } else { (end as usize).min(if len > 0 { len as usize - 1 } else { 0 }) };
                let mut result = Vec::new();
                let mut i = s;
                while i <= e && i < list.len() {
                    result.push(list[i].clone());
                    i += stride;
                }
                Ok(LSLValue::List(result))
            }

            // ==================== MATH FUNCTIONS ====================
            "llFrand" => {
                let mag = args.first().map(|a| a.to_float()).unwrap_or(1.0);
                let r: f32 = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos() as f32) / 4_294_967_296.0;
                Ok(LSLValue::Float(r * mag))
            }
            "llAbs" => {
                let v = args.first().map(|a| a.to_integer()).unwrap_or(0);
                Ok(LSLValue::Integer(v.abs()))
            }
            "llFabs" => {
                let v = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Float(v.abs()))
            }
            "llFloor" => {
                let v = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Integer(v.floor() as i32))
            }
            "llCeil" => {
                let v = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Integer(v.ceil() as i32))
            }
            "llRound" => {
                let v = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Integer(v.round() as i32))
            }
            "llSqrt" => {
                let v = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Float(if v >= 0.0 { v.sqrt() } else { 0.0 }))
            }
            "llPow" => {
                let base = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                let exp = args.get(1).map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Float(base.powf(exp)))
            }
            "llLog" => {
                let v = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Float(if v > 0.0 { v.ln() } else { 0.0 }))
            }
            "llLog10" => {
                let v = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Float(if v > 0.0 { v.log10() } else { 0.0 }))
            }
            "llSin" => {
                let v = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Float(v.sin()))
            }
            "llCos" => {
                let v = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Float(v.cos()))
            }
            "llTan" => {
                let v = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Float(v.tan()))
            }
            "llAsin" => {
                let v = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Float(v.clamp(-1.0, 1.0).asin()))
            }
            "llAcos" => {
                let v = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Float(v.clamp(-1.0, 1.0).acos()))
            }
            "llAtan2" => {
                let y = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                let x = args.get(1).map(|a| a.to_float()).unwrap_or(0.0);
                Ok(LSLValue::Float(y.atan2(x)))
            }

            // ==================== VECTOR / ROTATION ====================
            "llVecDist" => {
                let a = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let b = args.get(1).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                Ok(LSLValue::Float(a.distance(&b)))
            }
            "llVecMag" => {
                let v = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                Ok(LSLValue::Float(v.magnitude()))
            }
            "llVecNorm" => {
                let v = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                Ok(LSLValue::Vector(v.normalize()))
            }
            "llEuler2Rot" => {
                let v = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                Ok(LSLValue::Rotation(LSLRotation::from_euler(v.x, v.y, v.z)))
            }
            "llRot2Euler" => {
                let r = args.first().map(|a| a.to_rotation()).unwrap_or(LSLRotation::identity());
                let (roll, pitch, yaw) = r.to_euler();
                Ok(LSLValue::Vector(LSLVector::new(roll, pitch, yaw)))
            }
            "llAxes2Rot" | "llRot2Fwd" | "llRot2Left" | "llRot2Up" => {
                Ok(LSLValue::Vector(LSLVector::new(1.0, 0.0, 0.0)))
            }
            "llRotBetween" => {
                Ok(LSLValue::Rotation(LSLRotation::identity()))
            }
            "llAngleBetween" => {
                let a = args.first().map(|v| v.to_rotation()).unwrap_or(LSLRotation::identity());
                let b = args.get(1).map(|v| v.to_rotation()).unwrap_or(LSLRotation::identity());
                let dot = (a.x * b.x + a.y * b.y + a.z * b.z + a.s * b.s).clamp(-1.0, 1.0);
                Ok(LSLValue::Float(2.0 * dot.abs().acos()))
            }
            "llAxisAngle2Rot" => {
                let axis = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::new(0.0, 0.0, 1.0));
                let angle = args.get(1).map(|a| a.to_float()).unwrap_or(0.0);
                let mag = (axis.x * axis.x + axis.y * axis.y + axis.z * axis.z).sqrt();
                if mag < 1e-6 {
                    Ok(LSLValue::Rotation(LSLRotation::identity()))
                } else {
                    let half = angle / 2.0;
                    let s = half.sin() / mag;
                    Ok(LSLValue::Rotation(LSLRotation { x: axis.x * s, y: axis.y * s, z: axis.z * s, s: half.cos() }))
                }
            }
            "llRot2Axis" => {
                let r = args.first().map(|a| a.to_rotation()).unwrap_or(LSLRotation::identity());
                let mag = (r.x * r.x + r.y * r.y + r.z * r.z).sqrt();
                if mag < 1e-6 {
                    Ok(LSLValue::Vector(LSLVector::new(0.0, 0.0, 1.0)))
                } else {
                    Ok(LSLValue::Vector(LSLVector::new(r.x / mag, r.y / mag, r.z / mag)))
                }
            }
            "llRot2Angle" => {
                let r = args.first().map(|a| a.to_rotation()).unwrap_or(LSLRotation::identity());
                let mag = (r.x * r.x + r.y * r.y + r.z * r.z).sqrt();
                Ok(LSLValue::Float(2.0 * mag.atan2(r.s)))
            }

            // ==================== MISC UTILITY ====================
            "llGetUnixTime" | "llGetGMTclock" => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                if name == "llGetUnixTime" {
                    Ok(LSLValue::Integer(now.as_secs() as i32))
                } else {
                    Ok(LSLValue::Float((now.as_secs() % 86400) as f32))
                }
            }
            "llGetTimestamp" => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                let secs = now.as_secs();
                let hours = (secs / 3600) % 24;
                let mins = (secs / 60) % 60;
                let s = secs % 60;
                Ok(LSLValue::String(format!("2026-01-01T{:02}:{:02}:{:02}.000000Z", hours, mins, s)))
            }
            "llGenerateKey" => {
                Ok(LSLValue::Key(Uuid::new_v4()))
            }
            "llSHA1String" => {
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                Ok(LSLValue::String(format!("{:x}", md5::compute(s.as_bytes()))))
            }
            "llMD5String" => {
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                let nonce = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let input = format!("{}:{}", s, nonce);
                Ok(LSLValue::String(format!("{:x}", md5::compute(input.as_bytes()))))
            }

            // ==================== TYPE CONVERSION ====================
            "llList2List" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let start = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let end = args.get(2).map(|a| a.to_integer()).unwrap_or(-1);
                let len = list.len() as i32;
                if len == 0 { return Ok(LSLValue::List(vec![])); }
                let s = if start < 0 { (len + start).max(0) as usize } else { (start as usize).min(len as usize) };
                let e = if end < 0 { (len + end).max(0) as usize } else { (end as usize).min(len as usize - 1) };
                if s <= e {
                    Ok(LSLValue::List(list[s..=e].to_vec()))
                } else {
                    let mut r = list[..=e].to_vec();
                    r.extend_from_slice(&list[s..]);
                    Ok(LSLValue::List(r))
                }
            }
            "llGetListEntryType" => {
                Ok(LSLValue::Integer(0))
            }

            // ==================== PERMISSIONS / INVENTORY ====================
            "llRequestPermissions" => {
                let avatar = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let perms = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::RequestPermissions {
                    script_id: self.script_id,
                    object_id: self.ctx.object_id,
                    object_name: self.ctx.object_name.clone(),
                    avatar_id: avatar,
                    permissions: perms as u32,
                });
                Ok(LSLValue::Integer(0))
            }
            "llGetPermissions" => Ok(LSLValue::Integer(self.ctx.granted_perms as i32)),
            "llGetPermissionsKey" => Ok(LSLValue::Key(self.ctx.perm_granter)),
            "llGetInventoryNumber" => {
                let inv_type = args.first().map(|a| a.to_integer()).unwrap_or(-1);
                let count = self.ctx.inventory.iter()
                    .filter(|item| inv_type < 0 || item.asset_type == inv_type)
                    .count();
                Ok(LSLValue::Integer(count as i32))
            }
            "llGetInventoryName" => {
                let inv_type = args.first().map(|a| a.to_integer()).unwrap_or(-1);
                let index = args.get(1).map(|a| a.to_integer()).unwrap_or(0) as usize;
                let items: Vec<_> = self.ctx.inventory.iter()
                    .filter(|item| inv_type < 0 || item.asset_type == inv_type)
                    .collect();
                if index < items.len() {
                    Ok(LSLValue::String(items[index].name.clone()))
                } else {
                    Ok(LSLValue::String(String::new()))
                }
            }
            "llGetInventoryType" => {
                let name = args.first().map(|a| a.to_string()).unwrap_or_default();
                let inv_type = self.ctx.inventory.iter()
                    .find(|item| item.name == name)
                    .map(|item| item.inv_type)
                    .unwrap_or(-1);
                Ok(LSLValue::Integer(inv_type))
            }
            "llGetInventoryKey" => {
                let name = args.first().map(|a| a.to_string()).unwrap_or_default();
                let key = self.ctx.inventory.iter()
                    .find(|item| item.name == name)
                    .map(|item| item.asset_id)
                    .unwrap_or(Uuid::nil());
                Ok(LSLValue::Key(key))
            }
            "llGetInventoryCreator" => Ok(LSLValue::Key(Uuid::nil())),
            "llGetInventoryPermMask" => {
                let item_name = args.first().map(|a| a.to_string()).unwrap_or_default();
                let mask_type = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                if let Some(inv_item) = self.ctx.inventory.iter().find(|i| i.name == item_name) {
                    let perms = inv_item.permissions;
                    let result = match mask_type {
                        0 => perms,
                        1 => perms,
                        2 => 0,
                        3 => 0,
                        4 => self.ctx.next_owner_mask,
                        _ => 0,
                    };
                    Ok(LSLValue::Integer(result as i32))
                } else {
                    Ok(LSLValue::Integer(-1))
                }
            }
            "llGiveInventory" => {
                let destination = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let item_name = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                if !destination.is_nil() && !item_name.is_empty() {
                    self.actions.push(ScriptAction::GiveInventory {
                        prim_id: self.ctx.object_id,
                        destination_id: destination,
                        item_name,
                        owner_id: self.ctx.owner_id,
                    });
                }
                Ok(LSLValue::Integer(0))
            }
            "llRezObject" | "llRezAtRoot" => {
                let item_name = args.first().map(|a| a.to_string()).unwrap_or_default();
                let pos = args.get(1).map(|a| a.to_vector()).unwrap_or(super::super::LSLVector::zero());
                let vel = args.get(2).map(|a| a.to_vector()).unwrap_or(super::super::LSLVector::zero());
                let rot = args.get(3).map(|a| a.to_rotation()).unwrap_or(super::super::LSLRotation::identity());
                let param = args.get(4).map(|a| a.to_integer()).unwrap_or(0);
                let at_root = name == "llRezAtRoot";
                if !item_name.is_empty() {
                    self.actions.push(ScriptAction::RezObject {
                        prim_id: self.ctx.object_id,
                        item_name,
                        position: [pos.x, pos.y, pos.z],
                        velocity: [vel.x, vel.y, vel.z],
                        rotation: [rot.x, rot.y, rot.z, rot.s],
                        start_param: param,
                        at_root,
                        owner_id: self.ctx.owner_id,
                    });
                }
                Ok(LSLValue::Integer(0))
            }
            "llRemoveInventory" => {
                let item_name = args.first().map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::RemoveInventory { object_id: self.ctx.object_id, script_id: self.script_id, item_name });
                Ok(LSLValue::Integer(0))
            }
            "llGetAttached" => Ok(LSLValue::Integer(0)),
            "llGetObjectPrimCount" | "llGetAgentSize" => Ok(LSLValue::Integer(1)),

            // ==================== AGENT / AVATAR FUNCTIONS ====================
            "llGetAgentInfo" => Ok(LSLValue::Integer(0)),
            "llGetDisplayName" | "llGetUsername" | "llKey2Name" => {
                Ok(LSLValue::String(String::new())) // resolved in action consumer
            }
            "llRequestAgentData" => {
                let agent = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let data = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let qid = Uuid::new_v4().to_string();
                self.actions.push(ScriptAction::RequestUserData { script_id: self.script_id, agent_id: agent, query_id: qid.clone(), data_type: data });
                Ok(LSLValue::Key(Uuid::parse_str(&qid).unwrap_or(Uuid::nil())))
            }
            "llRequestDisplayName" | "llRequestUsername" => {
                let agent = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let qid = Uuid::new_v4().to_string();
                let dt = if name == "llRequestDisplayName" { 8 } else { 9 };
                self.actions.push(ScriptAction::RequestUserData { script_id: self.script_id, agent_id: agent, query_id: qid.clone(), data_type: dt });
                Ok(LSLValue::Key(Uuid::parse_str(&qid).unwrap_or(Uuid::nil())))
            }
            "llSameGroup" => Ok(LSLValue::Integer(0)),
            "llGetAgentLanguage" => Ok(LSLValue::String("en-us".to_string())),

            // ==================== TEXTURE / PRIM PARAMS ====================
            "llGetTexture" | "llGetColor" => Ok(LSLValue::String(String::new())),
            "llGetAlpha" => Ok(LSLValue::Float(1.0)),
            "llGetTextureOffset" | "llGetTextureScale" => Ok(LSLValue::Vector(LSLVector::zero())),
            "llGetTextureRot" => Ok(LSLValue::Float(0.0)),
            "llSetAlpha" => {
                let alpha = args.first().map(|a| a.to_float()).unwrap_or(1.0);
                let face = args.get(1).map(|a| a.to_integer()).unwrap_or(-1);
                self.actions.push(ScriptAction::SetAlpha { object_id: self.ctx.object_id, alpha, face });
                Ok(LSLValue::Integer(0))
            }
            "llSetSitText" => {
                let text = args.first().map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::SetSitText { object_id: self.ctx.object_id, text });
                Ok(LSLValue::Integer(0))
            }
            "llSetTouchText" => {
                let text = args.first().map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::SetTouchText { object_id: self.ctx.object_id, text });
                Ok(LSLValue::Integer(0))
            }
            "llSitTarget" => {
                let pos = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let rot = args.get(1).map(|a| a.to_rotation()).unwrap_or(LSLRotation::identity());
                self.actions.push(ScriptAction::SetSitTarget {
                    object_id: self.ctx.object_id,
                    position: [pos.x, pos.y, pos.z],
                    rotation: [rot.x, rot.y, rot.z, rot.s],
                });
                Ok(LSLValue::Integer(0))
            }
            "llAvatarOnSitTarget" => {
                Ok(LSLValue::Key(self.ctx.sitting_avatar_id))
            }
            "llUnSit" => {
                let target = args.first().map(|a| a.to_key()).unwrap_or(self.ctx.sitting_avatar_id);
                if !target.is_nil() {
                    self.actions.push(ScriptAction::UnSit { avatar_id: target });
                }
                Ok(LSLValue::Integer(0))
            }
            "llSetCameraParams" => {
                let list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let mut params = Vec::new();
                let mut i = 0;
                while i + 1 < list.len() {
                    let ptype = list[i].to_integer();
                    match ptype {
                        1 | 13 | 17 => {
                            let vec = list.get(i + 1).map(|v| v.to_vector()).unwrap_or(crate::scripting::lsl_types::LSLVector::new(0.0, 0.0, 0.0));
                            params.push((ptype, vec.x));
                            params.push((ptype + 1, vec.y));
                            params.push((ptype + 2, vec.z));
                            i += 2;
                        }
                        3 => {
                            let val = list.get(i + 1).map(|v| v.to_float()).unwrap_or(0.0);
                            params.push((16, val));
                            i += 2;
                        }
                        0 | 2 | 4..=12 | 14..=16 | 18..=22 => {
                            let val = list.get(i + 1).map(|v| v.to_float()).unwrap_or(0.0);
                            params.push((ptype, val));
                            i += 2;
                        }
                        _ => { i += 2; }
                    }
                }
                if !params.is_empty() {
                    self.actions.push(ScriptAction::SetCameraParams { avatar_id: self.ctx.perm_granter, object_id: self.ctx.object_id, params });
                }
                Ok(LSLValue::Integer(0))
            }
            "llSetPrimitiveParams" | "llSetLinkPrimitiveParams" | "llSetLinkPrimitiveParamsFast" => {
                let (link_num, rules_list) = if name == "llSetPrimitiveParams" {
                    (self.ctx.link_num, args.first().map(|a| a.to_list()).unwrap_or_default())
                } else {
                    let ln = args.first().map(|a| a.to_integer()).unwrap_or(0);
                    (ln, args.get(1).map(|a| a.to_list()).unwrap_or_default())
                };
                let mut rules = Vec::new();
                let mut i = 0;
                while i < rules_list.len() {
                    let rule = rules_list[i].to_integer();
                    match rule {
                        29 => { // PRIM_PHANTOM
                            let val = rules_list.get(i + 1).map(|v| v.to_integer() != 0).unwrap_or(false);
                            rules.push((29, val));
                            i += 2;
                        }
                        45 => { // PRIM_PHYSICS
                            let val = rules_list.get(i + 1).map(|v| v.to_integer() != 0).unwrap_or(false);
                            rules.push((45, val));
                            i += 2;
                        }
                        _ => { i += 1; }
                    }
                }
                if !rules.is_empty() {
                    self.actions.push(ScriptAction::SetLinkPrimParams { object_id: self.ctx.object_id, link_num, rules });
                }
                Ok(LSLValue::Integer(0))
            }
            "llSetColor" => {
                let color = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::new(1.0, 1.0, 1.0));
                let face = args.get(1).map(|a| a.to_integer()).unwrap_or(-1);
                self.actions.push(ScriptAction::SetColor { object_id: self.ctx.object_id, color: [color.x, color.y, color.z], face });
                Ok(LSLValue::Integer(0))
            }
            "llSetTexture" => {
                let tex = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let face = args.get(1).map(|a| a.to_integer()).unwrap_or(-1);
                self.actions.push(ScriptAction::SetTexture { object_id: self.ctx.object_id, texture_id: tex, face });
                Ok(LSLValue::Integer(0))
            }
            "llSetTextureAnim" => {
                let mode = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let face = args.get(1).map(|a| a.to_integer()).unwrap_or(-1);
                let sx = args.get(2).map(|a| a.to_integer()).unwrap_or(1);
                let sy = args.get(3).map(|a| a.to_integer()).unwrap_or(1);
                let start = args.get(4).map(|a| a.to_float()).unwrap_or(0.0);
                let length = args.get(5).map(|a| a.to_float()).unwrap_or(0.0);
                let rate = args.get(6).map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::SetTextureAnim { object_id: self.ctx.object_id, mode, face, size_x: sx, size_y: sy, start, length, rate });
                Ok(LSLValue::Integer(0))
            }
            "llSetLinkColor" => {
                let link = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let color = args.get(1).map(|a| a.to_vector()).unwrap_or(LSLVector::new(1.0, 1.0, 1.0));
                let face = args.get(2).map(|a| a.to_integer()).unwrap_or(-1);
                self.actions.push(ScriptAction::SetLinkColor { object_id: self.ctx.object_id, link_num: link, color: [color.x, color.y, color.z], face });
                Ok(LSLValue::Integer(0))
            }
            "llSetLinkAlpha" => {
                let link = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let alpha = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                let face = args.get(2).map(|a| a.to_integer()).unwrap_or(-1);
                self.actions.push(ScriptAction::SetLinkAlpha { object_id: self.ctx.object_id, link_num: link, alpha, face });
                Ok(LSLValue::Integer(0))
            }
            "llSetLinkTexture" => {
                let link = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let tex = args.get(1).map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let face = args.get(2).map(|a| a.to_integer()).unwrap_or(-1);
                self.actions.push(ScriptAction::SetLinkTexture { object_id: self.ctx.object_id, link_num: link, face, texture_id: tex });
                Ok(LSLValue::Integer(0))
            }
            "llSetObjectName" => {
                let name = args.first().map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::SetObjectName { object_id: self.ctx.object_id, name });
                Ok(LSLValue::Integer(0))
            }
            "llSetObjectDesc" => {
                let desc = args.first().map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::SetObjectDesc { object_id: self.ctx.object_id, desc });
                Ok(LSLValue::Integer(0))
            }
            "llTargetOmega" => {
                let axis = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let spinrate = args.get(1).map(|a| a.to_float()).unwrap_or(0.0);
                let gain = args.get(2).map(|a| a.to_float()).unwrap_or(0.0);
                self.actions.push(ScriptAction::TargetOmega { object_id: self.ctx.object_id, axis: [axis.x, axis.y, axis.z], spinrate, gain });
                Ok(LSLValue::Integer(0))
            }
            "llSetCameraEyeOffset" => {
                let v = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                self.actions.push(ScriptAction::SetCameraEyeOffset { object_id: self.ctx.object_id, offset: [v.x, v.y, v.z] });
                Ok(LSLValue::Integer(0))
            }
            "llSetCameraAtOffset" => {
                let v = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                self.actions.push(ScriptAction::SetCameraAtOffset { object_id: self.ctx.object_id, offset: [v.x, v.y, v.z] });
                Ok(LSLValue::Integer(0))
            }
            "llGetPrimitiveParams" | "llGetLinkPrimitiveParams" => {
                let (link, param_list) = if name == "llGetPrimitiveParams" {
                    (self.ctx.link_num, args.first().map(|a| a.to_list()).unwrap_or_default())
                } else {
                    (args.first().map(|a| a.to_integer()).unwrap_or(0),
                     args.get(1).map(|a| a.to_list()).unwrap_or_default())
                };
                let mut result = Vec::new();
                for param in &param_list {
                    let code = param.to_integer();
                    if code == 7 {
                        // PRIM_SIZE = 7
                        if link == 0 || link == self.ctx.link_num {
                            result.push(LSLValue::Vector(self.ctx.scale));
                        } else if let Some((_, scale)) = self.ctx.link_scales.iter().find(|(num, _)| *num == link) {
                            result.push(LSLValue::Vector(*scale));
                        } else {
                            result.push(LSLValue::Vector(LSLVector::new(1.0, 1.0, 1.0)));
                        }
                    }
                }
                Ok(LSLValue::List(result))
            }
            "llGetRegionCorner" => Ok(LSLValue::Vector(LSLVector::zero())),
            "llGetRegionFlags" => Ok(LSLValue::Integer(0x0004_0000)),
            "llGetSimulatorHostname" => Ok(LSLValue::String("localhost".to_string())),
            "llGetEnv" => {
                let key = args.first().map(|a| a.to_string()).unwrap_or_default();
                Ok(LSLValue::String(match key.as_str() {
                    "sim_channel" => "OpenSim Next".to_string(),
                    "sim_version" => "0.9.3.0".to_string(),
                    "frame_number" => "0".to_string(),
                    "region_idle" => "0".to_string(),
                    "dynamic_pathfinding" => "disabled".to_string(),
                    "estate_id" => "1".to_string(),
                    "region_max_prims" => "15000".to_string(),
                    _ => String::new(),
                }))
            }
            "llGetRegionFPS" | "llGetRegionTimeDilation" => Ok(LSLValue::Float(45.0)),
            "llGetSunDirection" => Ok(LSLValue::Vector(LSLVector::new(0.0, 0.707, 0.707))),
            "llGround" => Ok(LSLValue::Float(0.0)),
            "llWater" => Ok(LSLValue::Float(20.0)),
            "llWind" => Ok(LSLValue::Vector(LSLVector::zero())),
            "llCloud" => Ok(LSLValue::Float(0.0)),

            // ==================== PARTICLE SYSTEM ====================
            "llParticleSystem" => {
                let rules = args.first().map(|a| a.to_list()).unwrap_or_default();
                let ps_bytes = Self::encode_particle_system_inline(&rules);
                self.actions.push(ScriptAction::ParticleSystem { object_id: self.ctx.object_id, rules: ps_bytes });
                Ok(LSLValue::Integer(0))
            }
            "llLinkParticleSystem" => {
                let _link = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let rules = args.get(1).map(|a| a.to_list()).unwrap_or_default();
                let ps_bytes = Self::encode_particle_system_inline(&rules);
                self.actions.push(ScriptAction::ParticleSystem { object_id: self.ctx.object_id, rules: ps_bytes });
                Ok(LSLValue::Integer(0))
            }

            // ==================== PHYSICS / MOVEMENT ====================
            "llSetStatus" => {
                let status = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let value = args.get(1).map(|a| a.to_integer() != 0).unwrap_or(false);
                self.actions.push(ScriptAction::SetStatus { object_id: self.ctx.object_id, status, value });
                Ok(LSLValue::Integer(0))
            }
            "llSetVehicleType" => {
                let vtype = args.first().map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::SetVehicleType { object_id: self.ctx.object_id, vehicle_type: vtype });
                Ok(LSLValue::Integer(0))
            }
            "llSetVehicleFloatParam" => {
                let param = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let val = args.get(1).map(|a| a.to_float() as f64).unwrap_or(0.0);
                self.actions.push(ScriptAction::SetVehicleFloatParam { object_id: self.ctx.object_id, param_id: param, value: val });
                Ok(LSLValue::Integer(0))
            }
            "llSetVehicleVectorParam" => {
                let param = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let v = args.get(1).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                self.actions.push(ScriptAction::SetVehicleVectorParam { object_id: self.ctx.object_id, param_id: param, value: [v.x, v.y, v.z] });
                Ok(LSLValue::Integer(0))
            }
            "llSetVehicleRotationParam" => {
                let param = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let r = args.get(1).map(|a| a.to_rotation()).unwrap_or(LSLRotation::identity());
                self.actions.push(ScriptAction::SetVehicleRotationParam { object_id: self.ctx.object_id, param_id: param, value: [r.x, r.y, r.z, r.s] });
                Ok(LSLValue::Integer(0))
            }
            "llSetVehicleFlags" => {
                let f = args.first().map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::SetVehicleFlags { object_id: self.ctx.object_id, flags: f });
                Ok(LSLValue::Integer(0))
            }
            "llRemoveVehicleFlags" => {
                let f = args.first().map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::RemoveVehicleFlags { object_id: self.ctx.object_id, flags: f });
                Ok(LSLValue::Integer(0))
            }
            "llTakeControls" => {
                let controls = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let accept = args.get(1).map(|a| a.to_integer() != 0).unwrap_or(true);
                let pass_on = args.get(2).map(|a| a.to_integer() != 0).unwrap_or(false);
                if self.ctx.granted_perms & 0x04 != 0 {
                    self.actions.push(ScriptAction::TakeControls {
                        script_id: self.script_id, object_id: self.ctx.object_id,
                        avatar_id: self.ctx.perm_granter, controls, accept, pass_on,
                    });
                }
                Ok(LSLValue::Integer(0))
            }
            "llReleaseControls" => {
                if self.ctx.granted_perms & 0x04 != 0 {
                    self.actions.push(ScriptAction::ReleaseControls {
                        script_id: self.script_id, object_id: self.ctx.object_id,
                        avatar_id: self.ctx.perm_granter,
                    });
                }
                Ok(LSLValue::Integer(0))
            }
            "llSetBuoyancy" => {
                let b = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                self.actions.push(ScriptAction::SetBuoyancy { object_id: self.ctx.object_id, buoyancy: b });
                Ok(LSLValue::Integer(0))
            }
            "llSetForce" => {
                let v = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let local = args.get(1).map(|a| a.to_integer() != 0).unwrap_or(false);
                self.actions.push(ScriptAction::SetForce { object_id: self.ctx.object_id, force: [v.x, v.y, v.z], local });
                Ok(LSLValue::Integer(0))
            }
            "llSetTorque" => {
                let v = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let local = args.get(1).map(|a| a.to_integer() != 0).unwrap_or(false);
                self.actions.push(ScriptAction::SetTorque { object_id: self.ctx.object_id, torque: [v.x, v.y, v.z], local });
                Ok(LSLValue::Integer(0))
            }
            "llSetVelocity" => {
                let v = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let local = args.get(1).map(|a| a.to_integer() != 0).unwrap_or(false);
                self.actions.push(ScriptAction::SetVelocity { object_id: self.ctx.object_id, velocity: [v.x, v.y, v.z], local });
                Ok(LSLValue::Integer(0))
            }
            "llSetAngularVelocity" => {
                let v = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let local = args.get(1).map(|a| a.to_integer() != 0).unwrap_or(false);
                self.actions.push(ScriptAction::SetAngularVelocity { object_id: self.ctx.object_id, velocity: [v.x, v.y, v.z], local });
                Ok(LSLValue::Integer(0))
            }
            "llMoveToTarget" => {
                let pos = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let tau = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::MoveToTarget { object_id: self.ctx.object_id, target: [pos.x, pos.y, pos.z], tau });
                Ok(LSLValue::Integer(0))
            }
            "llStopMoveToTarget" => {
                self.actions.push(ScriptAction::StopMoveToTarget { object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llLookAt" => {
                let target = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let str_val = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                let damp = args.get(2).map(|a| a.to_float()).unwrap_or(0.2);
                self.actions.push(ScriptAction::LookAt { object_id: self.ctx.object_id, target: [target.x, target.y, target.z], strength: str_val, damping: damp });
                Ok(LSLValue::Integer(0))
            }
            "llStopLookAt" => {
                self.actions.push(ScriptAction::StopLookAt { object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llRotLookAt" => {
                let rot = args.first().map(|a| a.to_rotation()).unwrap_or(LSLRotation::identity());
                let str_val = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                let damp = args.get(2).map(|a| a.to_float()).unwrap_or(0.2);
                self.actions.push(ScriptAction::RotLookAt { object_id: self.ctx.object_id, rotation: [rot.x, rot.y, rot.z, rot.s], strength: str_val, damping: damp });
                Ok(LSLValue::Integer(0))
            }
            "llSetPhysicsMaterial" => {
                let flags = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let grav = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                let rest = args.get(2).map(|a| a.to_float()).unwrap_or(0.5);
                let fric = args.get(3).map(|a| a.to_float()).unwrap_or(0.5);
                let dens = args.get(4).map(|a| a.to_float()).unwrap_or(1000.0);
                self.actions.push(ScriptAction::SetPhysicsMaterial { object_id: self.ctx.object_id, gravity: grav, restitution: rest, friction: fric, density: dens, flags });
                Ok(LSLValue::Integer(0))
            }
            "llSetHoverHeight" => {
                let h = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                let water = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let tau = args.get(2).map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::SetHoverHeight { object_id: self.ctx.object_id, height: h, water, tau });
                Ok(LSLValue::Integer(0))
            }
            "llStopHover" => {
                self.actions.push(ScriptAction::StopHover { object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llPushObject" => {
                let target = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let imp = args.get(1).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let ang = args.get(2).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let local = args.get(3).map(|a| a.to_integer() != 0).unwrap_or(false);
                self.actions.push(ScriptAction::PushObject { target_id: target, impulse: [imp.x, imp.y, imp.z], angular_impulse: [ang.x, ang.y, ang.z], local });
                Ok(LSLValue::Integer(0))
            }
            "llPassTouches" => {
                let pass = args.first().map(|a| a.to_integer() != 0).unwrap_or(false);
                self.actions.push(ScriptAction::SetPassTouches { object_id: self.ctx.object_id, pass });
                Ok(LSLValue::Integer(0))
            }
            "llPassCollisions" => {
                let pass = args.first().map(|a| a.to_integer() != 0).unwrap_or(false);
                self.actions.push(ScriptAction::SetPassCollisions { object_id: self.ctx.object_id, pass });
                Ok(LSLValue::Integer(0))
            }
            "llSetForceAndTorque" => {
                let force = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let torque = args.get(1).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let local = args.get(2).map(|a| a.to_integer() != 0).unwrap_or(false);
                self.actions.push(ScriptAction::SetForceAndTorque { object_id: self.ctx.object_id, force: [force.x, force.y, force.z], torque: [torque.x, torque.y, torque.z], local });
                Ok(LSLValue::Integer(0))
            }
            "llApplyRotationalImpulse" => {
                let imp = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let local = args.get(1).map(|a| a.to_integer() != 0).unwrap_or(false);
                self.actions.push(ScriptAction::ApplyRotationalImpulse { object_id: self.ctx.object_id, impulse: [imp.x, imp.y, imp.z], local });
                Ok(LSLValue::Integer(0))
            }
            "llGroundRepel" => {
                let h = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                let water = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let tau = args.get(2).map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::GroundRepel { object_id: self.ctx.object_id, height: h, water, tau });
                Ok(LSLValue::Integer(0))
            }
            "llGetStatus" => Ok(LSLValue::Integer(0)),
            "llGetVel" => Ok(LSLValue::Vector(self.ctx.velocity)),
            "llGetForce" | "llGetTorque" | "llGetAccel" | "llGetOmega" => {
                Ok(LSLValue::Vector(LSLVector::zero()))
            }
            "llGetMass" => Ok(LSLValue::Float(1.0)),
            "llGetBoundingBox" => Ok(LSLValue::List(vec![
                LSLValue::Vector(LSLVector::zero()),
                LSLValue::Vector(LSLVector::new(1.0, 1.0, 1.0)),
            ])),

            // ==================== SOUNDS ====================
            "llTriggerSound" => {
                let sound = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let vol = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::TriggerSound { object_id: self.ctx.object_id, sound_id: sound, volume: vol.clamp(0.0, 1.0) });
                Ok(LSLValue::Integer(0))
            }
            "llPlaySound" => {
                let sound = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let vol = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::PlaySound { object_id: self.ctx.object_id, sound_id: sound, volume: vol.clamp(0.0, 1.0) });
                Ok(LSLValue::Integer(0))
            }
            "llLoopSound" => {
                let sound = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let vol = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::LoopSound { object_id: self.ctx.object_id, sound_id: sound, volume: vol.clamp(0.0, 1.0) });
                Ok(LSLValue::Integer(0))
            }
            "llLoopSoundMaster" => {
                let sound = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let vol = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::LoopSoundMaster { object_id: self.ctx.object_id, sound_id: sound, volume: vol.clamp(0.0, 1.0) });
                Ok(LSLValue::Integer(0))
            }
            "llLoopSoundSlave" => {
                let sound = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let vol = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::LoopSoundSlave { object_id: self.ctx.object_id, sound_id: sound, volume: vol.clamp(0.0, 1.0) });
                Ok(LSLValue::Integer(0))
            }
            "llPlaySoundSlave" => {
                let sound = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let vol = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::PlaySoundSlave { object_id: self.ctx.object_id, sound_id: sound, volume: vol.clamp(0.0, 1.0) });
                Ok(LSLValue::Integer(0))
            }
            "llStopSound" => {
                self.actions.push(ScriptAction::StopSound { object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llPreloadSound" => {
                let sound = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::PreloadSound { object_id: self.ctx.object_id, sound_id: sound });
                Ok(LSLValue::Integer(0))
            }
            "llTriggerSoundLimited" => {
                let sound = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let vol = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                let tne = args.get(2).map(|a| a.to_vector()).unwrap_or(LSLVector::new(256.0, 256.0, 256.0));
                let bsw = args.get(3).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                self.actions.push(ScriptAction::TriggerSoundLimited { object_id: self.ctx.object_id, sound_id: sound, volume: vol.clamp(0.0, 1.0), top_ne: [tne.x, tne.y, tne.z], bot_sw: [bsw.x, bsw.y, bsw.z] });
                Ok(LSLValue::Integer(0))
            }
            "llAdjustSoundVolume" => {
                let vol = args.first().map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::AdjustSoundVolume { object_id: self.ctx.object_id, volume: vol.clamp(0.0, 1.0) });
                Ok(LSLValue::Integer(0))
            }
            "llSetSoundQueueing" => {
                let q = args.first().map(|a| a.to_integer() != 0).unwrap_or(false);
                self.actions.push(ScriptAction::SetSoundQueueing { object_id: self.ctx.object_id, queueing: q });
                Ok(LSLValue::Integer(0))
            }
            "llSetSoundRadius" => {
                let r = args.first().map(|a| a.to_float()).unwrap_or(0.0);
                self.actions.push(ScriptAction::SetSoundRadius { object_id: self.ctx.object_id, radius: r });
                Ok(LSLValue::Integer(0))
            }
            "llCollisionSound" => {
                let sound = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let vol = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::SetCollisionSound { object_id: self.ctx.object_id, sound_id: sound, volume: vol.clamp(0.0, 1.0) });
                Ok(LSLValue::Integer(0))
            }

            // ==================== LINKS ====================
            "llGetLinkKey" => Ok(LSLValue::Key(self.ctx.object_id)),
            "llGetLinkName" => {
                let link = args.first().map(|a| a.to_integer()).unwrap_or(0);
                if link == 0 || link == self.ctx.link_num {
                    Ok(LSLValue::String(self.ctx.object_name.clone()))
                } else if let Some((_, name)) = self.ctx.link_names.iter().find(|(num, _)| *num == link) {
                    Ok(LSLValue::String(name.clone()))
                } else {
                    Ok(LSLValue::String(String::new()))
                }
            }
            "llGetLinkNumberOfSides" => Ok(LSLValue::Integer(6)),
            "llGetNumberOfSides" => Ok(LSLValue::Integer(6)),

            // ==================== ANIMATION ====================
            "llStartAnimation" => {
                let anim = args.first().map(|a| a.to_string()).unwrap_or_default();
                if self.ctx.granted_perms & 0x10 != 0 {
                    let resolved = self.ctx.inventory.iter()
                        .find(|item| item.name == anim && item.asset_type == 20)
                        .map(|item| item.asset_id.to_string())
                        .unwrap_or(anim);
                    self.actions.push(ScriptAction::StartAnimation { avatar_id: self.ctx.perm_granter, anim_name: resolved });
                }
                Ok(LSLValue::Integer(0))
            }
            "llStopAnimation" => {
                let anim = args.first().map(|a| a.to_string()).unwrap_or_default();
                if self.ctx.granted_perms & 0x10 != 0 {
                    let resolved = self.ctx.inventory.iter()
                        .find(|item| item.name == anim && item.asset_type == 20)
                        .map(|item| item.asset_id.to_string())
                        .unwrap_or(anim);
                    self.actions.push(ScriptAction::StopAnimation { avatar_id: self.ctx.perm_granter, anim_name: resolved });
                }
                Ok(LSLValue::Integer(0))
            }
            "llGetAnimation" => {
                Ok(LSLValue::String("Standing".into()))
            }
            "llGetAnimationList" => {
                Ok(LSLValue::List(vec![]))
            }
            "llSetAnimationOverride" => {
                let anim_state = args.first().map(|a| a.to_string()).unwrap_or_default();
                let anim_name = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                if self.ctx.granted_perms & 0x8000 != 0 {
                    self.actions.push(ScriptAction::SetAnimationOverride { avatar_id: self.ctx.perm_granter, anim_state, anim_name });
                }
                Ok(LSLValue::Integer(0))
            }
            "llResetAnimationOverride" => {
                let anim_state = args.first().map(|a| a.to_string()).unwrap_or_default();
                if self.ctx.granted_perms & 0x8000 != 0 {
                    self.actions.push(ScriptAction::ResetAnimationOverride { avatar_id: self.ctx.perm_granter, anim_state });
                }
                Ok(LSLValue::Integer(0))
            }

            // ==================== HTTP ====================
            "llHTTPRequest" => {
                let url = args.first().map(|a| a.to_string()).unwrap_or_default();
                let params_list = args.get(1).map(|a| a.to_list()).unwrap_or_default();
                let body = args.get(2).map(|a| a.to_string()).unwrap_or_default();
                let mut params = Vec::new();
                let mut i = 0;
                while i + 1 < params_list.len() {
                    let k = params_list[i].to_string();
                    let v = params_list[i + 1].to_string();
                    params.push((k, v));
                    i += 2;
                }
                let request_id = Uuid::new_v4();
                self.actions.push(ScriptAction::HttpRequest { object_id: self.ctx.object_id, script_id: self.script_id, url, params, body });
                Ok(LSLValue::Key(request_id))
            }
            "llHTTPResponse" => {
                let request_id = args.first().map(|a| a.to_string()).unwrap_or_default();
                let status = args.get(1).map(|a| a.to_integer()).unwrap_or(200);
                let body = args.get(2).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::HTTPInResponse { request_id, status, body, content_type: "text/plain".into() });
                Ok(LSLValue::Integer(0))
            }
            "llSetContentType" => {
                Ok(LSLValue::Integer(0))
            }
            "llGetHTTPHeader" => Ok(LSLValue::String(String::new())),
            "llRequestURL" => {
                let request_id = Uuid::new_v4();
                self.actions.push(ScriptAction::RequestURL { object_id: self.ctx.object_id, script_id: self.script_id, request_id, secure: false });
                Ok(LSLValue::Key(request_id))
            }
            "llRequestSecureURL" => {
                let request_id = Uuid::new_v4();
                self.actions.push(ScriptAction::RequestURL { object_id: self.ctx.object_id, script_id: self.script_id, request_id, secure: true });
                Ok(LSLValue::Key(request_id))
            }
            "llReleaseURL" => {
                let url = args.first().map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::ReleaseURL { url });
                Ok(LSLValue::Integer(0))
            }
            "llGetFreeURLs" => Ok(LSLValue::Integer(100)),
            "llEscapeURL" => {
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                Ok(LSLValue::String(s))
            }
            "llUnescapeURL" => {
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                Ok(LSLValue::String(s))
            }

            // ==================== LAND / PARCEL ====================
            "llGetParcelFlags" | "llGetParcelMaxPrims" | "llGetParcelPrimCount"
            | "llGetParcelPrimOwners" => Ok(LSLValue::Integer(0)),
            "llGetParcelDetails" => Ok(LSLValue::List(vec![])),
            "llOverMyLand" => Ok(LSLValue::Integer(0)),

            // ==================== EXPERIENCE (stubs — no pathfinding/experience system) ====================
            "llGetExperienceDetails" | "llAgentInExperience" | "llGetExperienceErrorMessage" => {
                Ok(LSLValue::String(String::new()))
            }
            "llCreateCharacter" | "llDeleteCharacter" | "llUpdateCharacter"
            | "llExecCharacterCmd" | "llNavigateTo" | "llWanderWithin"
            | "llPatrolPoints" | "llPursue" | "llEvade" | "llFleeFrom" => {
                Ok(LSLValue::Integer(0))
            }
            "llGetClosestNavPoint" => {
                let pos = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                Ok(LSLValue::List(vec![LSLValue::Vector(pos)]))
            }
            "llGetStaticPath" => Ok(LSLValue::List(vec![])),

            // ==================== CAMERA (Wired) ====================
            "llClearCameraParams" => {
                if self.ctx.granted_perms & 0x800 != 0 {
                    self.actions.push(ScriptAction::ClearCameraParams { avatar_id: self.ctx.perm_granter });
                }
                Ok(LSLValue::Integer(0))
            }

            // ==================== MISC ACTIONS (Wired) ====================
            "llLoadURL" => {
                let avatar = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let msg = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                let url = args.get(2).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::LoadURL { avatar_id: avatar, message: msg, url, object_name: self.ctx.object_name.clone() });
                Ok(LSLValue::Integer(0))
            }
            "llMapDestination" => {
                let sim = args.first().map(|a| a.to_string()).unwrap_or_default();
                let pos = args.get(1).map(|a| a.to_vector()).unwrap_or(LSLVector::new(128.0, 128.0, 0.0));
                let look = args.get(2).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                self.actions.push(ScriptAction::MapDestination { avatar_id: self.ctx.perm_granter, sim_name: sim, position: [pos.x, pos.y, pos.z], look_at: [look.x, look.y, look.z] });
                Ok(LSLValue::Integer(0))
            }
            "llScaleByFactor" => {
                let factor = args.first().map(|a| a.to_float() as f64).unwrap_or(1.0);
                if factor > 0.0 {
                    self.actions.push(ScriptAction::ScaleByFactor { object_id: self.ctx.object_id, factor });
                }
                Ok(LSLValue::Float(if factor > 0.0 { 1.0 } else { 0.0 }))
            }
            "llKeyframedMotion" | "llSetKeyframedMotion" => {
                let kf_list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let opts_list = args.get(1).map(|a| a.to_list()).unwrap_or_default();
                let mut keyframes = Vec::new();
                for v in &kf_list { keyframes.push(v.to_float()); }
                let mode = opts_list.first().map(|a| a.to_integer()).unwrap_or(0);
                let data = opts_list.get(1).map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::KeyframedMotion { object_id: self.ctx.object_id, keyframes, mode, data });
                Ok(LSLValue::Integer(0))
            }
            "llEmail" => {
                let addr = args.first().map(|a| a.to_string()).unwrap_or_default();
                let subj = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                let body = args.get(2).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::Email { object_id: self.ctx.object_id, address: addr, subject: subj, message: body });
                Ok(LSLValue::Integer(0))
            }
            "llTeleportAgent" => {
                let agent = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let lm = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                let pos = args.get(2).map(|a| a.to_vector()).unwrap_or(LSLVector::new(128.0, 128.0, 21.0));
                let look = args.get(3).map(|a| a.to_vector()).unwrap_or(LSLVector::new(0.0, 1.0, 0.0));
                self.actions.push(ScriptAction::TeleportAgent { agent_id: agent, landmark: lm, position: [pos.x, pos.y, pos.z], look_at: [look.x, look.y, look.z] });
                Ok(LSLValue::Integer(0))
            }
            "llTeleportAgentHome" => {
                let agent = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::TeleportAgentHome { avatar_id: agent });
                Ok(LSLValue::Integer(0))
            }
            "llEjectFromLand" => {
                let agent = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::EjectFromLand { object_id: self.ctx.object_id, agent_id: agent });
                Ok(LSLValue::Integer(0))
            }
            "llCastRay" => {
                let start = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let end = args.get(1).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let opts = args.get(2).map(|a| a.to_list()).unwrap_or_default();
                let mut reject = 0; let mut max_hits = 1;
                let mut i = 0;
                while i + 1 < opts.len() {
                    match opts[i].to_integer() {
                        0 => { reject = opts[i+1].to_integer(); i += 2; }
                        2 => { max_hits = opts[i+1].to_integer(); i += 2; }
                        _ => { i += 2; }
                    }
                }
                self.actions.push(ScriptAction::CastRay { object_id: self.ctx.object_id, start: [start.x, start.y, start.z], end: [end.x, end.y, end.z], reject_types: reject, max_hits });
                Ok(LSLValue::List(vec![LSLValue::Integer(0)]))
            }
            "llResetOtherScript" => {
                let sn = args.first().map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::ResetOtherScript { object_id: self.ctx.object_id, script_name: sn });
                Ok(LSLValue::Integer(0))
            }
            "llSetScriptState" => {
                let sn = args.first().map(|a| a.to_string()).unwrap_or_default();
                let running = args.get(1).map(|a| a.to_integer() != 0).unwrap_or(true);
                self.actions.push(ScriptAction::SetScriptState { object_id: self.ctx.object_id, script_name: sn, running });
                Ok(LSLValue::Integer(0))
            }
            "llSetPayPrice" => {
                let price = args.first().map(|a| a.to_integer()).unwrap_or(-1);
                let buttons_list = args.get(1).map(|a| a.to_list()).unwrap_or_default();
                let mut prices = [price, -1, -1, -1, -1];
                for (j, v) in buttons_list.iter().take(4).enumerate() {
                    prices[j + 1] = v.to_integer();
                }
                self.actions.push(ScriptAction::SetPayPrice { object_id: self.ctx.object_id, prices });
                Ok(LSLValue::Integer(0))
            }
            "llCollisionSprite" => {
                self.actions.push(ScriptAction::CollisionSprite { object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llAttachToAvatar" => {
                let point = args.first().map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::AttachToAvatar { object_id: self.ctx.object_id, attach_point: point });
                Ok(LSLValue::Integer(0))
            }
            "llDetachFromAvatar" => {
                self.actions.push(ScriptAction::DetachFromAvatar { object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llGiveMoney" => {
                let dest = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let amount = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::GiveMoney { owner_id: self.ctx.owner_id, destination_id: dest, amount, object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llSetParcelMusicURL" => {
                let url = args.first().map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::SetParcelMusicURL { object_id: self.ctx.object_id, url });
                Ok(LSLValue::Integer(0))
            }
            "llAddToLandBanList" => {
                let agent = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let hours = args.get(1).map(|a| a.to_float()).unwrap_or(0.0);
                self.actions.push(ScriptAction::AddToLandBanList { object_id: self.ctx.object_id, agent_id: agent, hours, is_ban: true });
                Ok(LSLValue::Integer(0))
            }
            "llRemoveFromLandBanList" => {
                let agent = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::RemoveFromLandBanList { object_id: self.ctx.object_id, agent_id: agent, is_ban: true });
                Ok(LSLValue::Integer(0))
            }
            "llResetLandBanList" => {
                self.actions.push(ScriptAction::ResetLandBanList { object_id: self.ctx.object_id, is_ban: true });
                Ok(LSLValue::Integer(0))
            }
            "llAddToLandPassList" => {
                let agent = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let hours = args.get(1).map(|a| a.to_float()).unwrap_or(0.0);
                self.actions.push(ScriptAction::AddToLandBanList { object_id: self.ctx.object_id, agent_id: agent, hours, is_ban: false });
                Ok(LSLValue::Integer(0))
            }
            "llRemoveFromLandPassList" => {
                let agent = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::RemoveFromLandBanList { object_id: self.ctx.object_id, agent_id: agent, is_ban: false });
                Ok(LSLValue::Integer(0))
            }
            "llManageEstateAccess" => {
                let action = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let agent = args.get(1).map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::ManageEstateAccess { object_id: self.ctx.object_id, action, agent_id: agent });
                Ok(LSLValue::Integer(0))
            }
            "llGetObjectPermMask" => {
                let mask_type = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let result = match mask_type {
                    0 => self.ctx.base_mask,
                    1 => self.ctx.owner_mask,
                    2 => self.ctx.group_mask,
                    3 => self.ctx.everyone_mask,
                    4 => self.ctx.next_owner_mask,
                    _ => 0,
                };
                Ok(LSLValue::Integer(result as i32))
            }
            "llSetObjectPermMask" => {
                let mask_type = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let mask_value = args.get(1).map(|a| a.to_integer() as u32).unwrap_or(0);
                self.actions.push(ScriptAction::SetObjectPermMask { object_id: self.ctx.object_id, mask_type, mask_value });
                Ok(LSLValue::Integer(0))
            }
            "llSetInventoryPermMask" => {
                let item = args.first().map(|a| a.to_string()).unwrap_or_default();
                let mask_type = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let mask_value = args.get(2).map(|a| a.to_integer() as u32).unwrap_or(0);
                self.actions.push(ScriptAction::SetInventoryPermMask { object_id: self.ctx.object_id, item_name: item, mask_type, mask_value });
                Ok(LSLValue::Integer(0))
            }
            "llCreateLink" => {
                let target = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let parent = args.get(1).map(|a| a.to_integer() != 0).unwrap_or(true);
                self.actions.push(ScriptAction::CreateLink { object_id: self.ctx.object_id, target_id: target, parent });
                Ok(LSLValue::Integer(0))
            }
            "llBreakLink" => {
                let link = args.first().map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::BreakLink { object_id: self.ctx.object_id, link_num: link });
                Ok(LSLValue::Integer(0))
            }
            "llBreakAllLinks" => {
                self.actions.push(ScriptAction::BreakAllLinks { object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llReturnObjectsByID" => {
                let ids_list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let ids: Vec<Uuid> = ids_list.iter().map(|v| v.to_key()).collect();
                self.actions.push(ScriptAction::ReturnObjectsByID { object_id: self.ctx.object_id, ids });
                Ok(LSLValue::Integer(0))
            }
            "llReturnObjectsByOwner" => {
                let owner = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::ReturnObjectsByOwner { object_id: self.ctx.object_id, owner_id: owner });
                Ok(LSLValue::Integer(0))
            }
            "llSetPrimMediaParams" => {
                let face = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let pl = args.get(1).map(|a| a.to_list()).unwrap_or_default();
                let mut params = Vec::new();
                let mut i = 0;
                while i + 1 < pl.len() { params.push((pl[i].to_integer(), pl[i+1].to_string())); i += 2; }
                self.actions.push(ScriptAction::SetPrimMediaParams { object_id: self.ctx.object_id, face, params });
                Ok(LSLValue::Integer(0))
            }
            "llClearPrimMedia" => {
                let face = args.first().map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::ClearPrimMedia { object_id: self.ctx.object_id, face });
                Ok(LSLValue::Integer(0))
            }
            "llCollisionFilter" => {
                let name = args.first().map(|a| a.to_string()).unwrap_or_default();
                let id = args.get(1).map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let accept = args.get(2).map(|a| a.to_integer() != 0).unwrap_or(true);
                self.actions.push(ScriptAction::CollisionFilter { object_id: self.ctx.object_id, name, id, accept });
                Ok(LSLValue::Integer(0))
            }
            "llSetLinkSitFlags" => {
                let _link = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let flags = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::SetLinkSitFlags { object_id: self.ctx.object_id, flags });
                Ok(LSLValue::Integer(0))
            }
            "llLinksetDataReset" => {
                self.actions.push(ScriptAction::LinksetDataReset { object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }
            "llDerezObject" => {
                let id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::DerezObject { object_id: self.ctx.object_id, target_id: id });
                Ok(LSLValue::Integer(0))
            }
            "llGetAgentList" => {
                let scope = args.first().map(|a| a.to_integer()).unwrap_or(1);
                self.actions.push(ScriptAction::GetAgentList { object_id: self.ctx.object_id, scope });
                Ok(LSLValue::List(vec![]))
            }
            "llMakeExplosion" | "llMakeFire" | "llMakeFountain" | "llMakeSmoke" => {
                let particles = args.first().map(|a| a.to_integer()).unwrap_or(10);
                let scale = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                let flags: u32 = match name {
                    "llMakeExplosion" => 0x02 | 0x20,
                    "llMakeFire" => 0x02 | 0x10 | 0x100,
                    "llMakeFountain" => 0x02 | 0x80,
                    _ => 0x02 | 0x10,
                };
                let rules = vec![
                    LSLValue::Integer(0), LSLValue::Integer(flags as i32),
                    LSLValue::Integer(1), LSLValue::Integer(particles.min(100)),
                    LSLValue::Integer(2), LSLValue::Float(if name == "llMakeFire" { 3.0 } else { 1.0 }),
                    LSLValue::Integer(3), LSLValue::Float(0.5),
                    LSLValue::Integer(5), LSLValue::Float(scale),
                    LSLValue::Integer(7), LSLValue::Float(if name == "llMakeExplosion" { 2.0 } else { 0.5 }),
                ];
                let ps_bytes = Self::encode_particle_system_inline(&rules);
                self.actions.push(ScriptAction::ParticleSystem { object_id: self.ctx.object_id, rules: ps_bytes });
                Ok(LSLValue::Integer(0))
            }
            "llPointAt" | "llStopPointAt" | "llRemoteLoadScript"
            | "llOpenRemoteDataChannel" | "llCloseRemoteDataChannel"
            | "llSendRemoteData" | "llRemoteDataReply"
            | "llRefreshPrimURL" => {
                Ok(LSLValue::Integer(0))
            }
            "llListenControl" => {
                let handle = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let active = args.get(1).map(|a| a.to_integer() != 0).unwrap_or(true);
                self.actions.push(ScriptAction::ListenControl { object_id: self.ctx.object_id, handle, active });
                Ok(LSLValue::Integer(0))
            }
            "llSetLinkTextureAnim" => {
                let link = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let mode = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let face = args.get(2).map(|a| a.to_integer()).unwrap_or(-1);
                let sx = args.get(3).map(|a| a.to_integer()).unwrap_or(1);
                let sy = args.get(4).map(|a| a.to_integer()).unwrap_or(1);
                let start = args.get(5).map(|a| a.to_float()).unwrap_or(0.0);
                let length = args.get(6).map(|a| a.to_float()).unwrap_or(0.0);
                let rate = args.get(7).map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::SetLinkTextureAnim { object_id: self.ctx.object_id, link_num: link, mode, face, sizex: sx, sizey: sy, start, length, rate });
                Ok(LSLValue::Integer(0))
            }
            "llParcelMediaCommandList" => {
                let cmds_list = args.first().map(|a| a.to_list()).unwrap_or_default();
                let cmds: Vec<i32> = cmds_list.iter().map(|v| v.to_integer()).collect();
                self.actions.push(ScriptAction::ParcelMediaCommandList { object_id: self.ctx.object_id, commands: cmds });
                Ok(LSLValue::Integer(0))
            }

            // ==================== OSSL: NPC Management ====================
            "osNpcCreate" => {
                let first = args.first().map(|a| a.to_string()).unwrap_or_default();
                let last = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                let pos = args.get(2).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let nc = args.get(3).map(|a| a.to_string()).unwrap_or_default();
                let opts = args.get(4).map(|a| a.to_integer()).unwrap_or(0);
                let npc_id = Uuid::new_v4();
                self.actions.push(ScriptAction::NpcCreate { first_name: first, last_name: last, position: [pos.x as f32, pos.y as f32, pos.z as f32], notecard: nc, options: opts });
                Ok(LSLValue::Key(npc_id))
            }
            "osNpcRemove" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::NpcRemove { npc_id });
                Ok(LSLValue::Integer(0))
            }
            "osNpcSay" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let ch = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let msg = args.get(2).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::NpcSay { npc_id, channel: ch, message: msg });
                Ok(LSLValue::Integer(0))
            }
            "osNpcMoveTo" | "osNpcMoveToTarget" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let pos = args.get(1).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let opts = args.get(2).map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::NpcMoveTo { npc_id, position: [pos.x as f32, pos.y as f32, pos.z as f32], options: opts });
                Ok(LSLValue::Integer(0))
            }
            "osIsNpc" => {
                let _uuid = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                Ok(LSLValue::Integer(0))
            }
            "osNpcSetRot" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let rot = args.get(1).map(|a| a.to_rotation()).unwrap_or(LSLRotation::identity());
                self.actions.push(ScriptAction::NpcSetRot { npc_id, rotation: [rot.x as f32, rot.y as f32, rot.z as f32, rot.s as f32] });
                Ok(LSLValue::Integer(0))
            }
            "osNpcPlayAnimation" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let anim = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::NpcPlayAnimation { npc_id, anim_name: anim });
                Ok(LSLValue::Integer(0))
            }
            "osNpcStopAnimation" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let anim = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::NpcStopAnimation { npc_id, anim_name: anim });
                Ok(LSLValue::Integer(0))
            }
            "osNpcShout" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let ch = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let msg = args.get(2).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::NpcShout { npc_id, channel: ch, message: msg });
                Ok(LSLValue::Integer(0))
            }
            "osNpcWhisper" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let ch = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let msg = args.get(2).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::NpcWhisper { npc_id, channel: ch, message: msg });
                Ok(LSLValue::Integer(0))
            }
            "osNpcTouch" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let target = args.get(1).map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let link = args.get(2).map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::NpcTouch { npc_id, target_id: target, link_num: link });
                Ok(LSLValue::Integer(0))
            }
            "osNpcSit" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let target = args.get(1).map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::NpcSit { npc_id, target_id: target });
                Ok(LSLValue::Integer(0))
            }
            "osNpcStand" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::NpcStand { npc_id });
                Ok(LSLValue::Integer(0))
            }
            "osNpcLoadAppearance" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let nc = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::NpcLoadAppearance { npc_id, notecard: nc });
                Ok(LSLValue::Integer(0))
            }
            "osNpcSaveAppearance" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let nc = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::NpcSaveAppearance { npc_id, notecard: nc });
                Ok(LSLValue::Key(Uuid::new_v4()))
            }
            "osNpcSetProfileAbout" | "osNpcSetProfileImage" => Ok(LSLValue::Integer(0)),
            "osNpcGetPos" => {
                Ok(LSLValue::Vector(LSLVector::zero()))
            }
            "osNpcGetRot" => {
                Ok(LSLValue::Rotation(LSLRotation::identity()))
            }

            // ==================== OSSL: Statue Creation ====================
            "osCreateStatue" => {
                let anim_name = args.first().map(|a| a.to_string()).unwrap_or_default();
                let frame = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let total_frames = args.get(2).map(|a| a.to_integer()).unwrap_or(50);
                let pos = args.get(3).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let name = args.get(4).map(|a| a.to_string()).unwrap_or_else(|| format!("Statue_{}", anim_name));
                self.actions.push(ScriptAction::CreateStatue {
                    owner_id: self.ctx.owner_id,
                    anim_name,
                    frame,
                    total_frames,
                    position: [pos.x as f32, pos.y as f32, pos.z as f32],
                    name,
                });
                Ok(LSLValue::Key(Uuid::new_v4()))
            }

            "osCreateSnapshotStatue" => {
                let anim_name = args.first().map(|a| a.to_string()).unwrap_or_default();
                let frame = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let pos = args.get(2).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let name = args.get(3).map(|a| a.to_string()).unwrap_or_else(|| format!("Snapshot_{}", anim_name));
                self.actions.push(ScriptAction::CreateSnapshotStatue {
                    owner_id: self.ctx.owner_id,
                    anim_name,
                    frame,
                    position: [pos.x as f32, pos.y as f32, pos.z as f32],
                    name,
                });
                Ok(LSLValue::Key(Uuid::new_v4()))
            }

            // ==================== OSSL: Skill Engine ====================
            "osInvokeSkill" => {
                let domain = args.first().map(|a| a.to_string()).unwrap_or_default();
                let skill_id = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                let params_json = args.get(2).map(|a| a.to_string()).unwrap_or_else(|| "{}".to_string());
                self.actions.push(ScriptAction::InvokeSkill {
                    object_id: self.ctx.object_id,
                    script_id: self.script_id,
                    domain,
                    skill_id,
                    params_json,
                });
                Ok(LSLValue::String(String::new()))
            }

            // ==================== OSSL: Terrain & Environment ====================
            "osSetTerrainHeight" => {
                let x = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let y = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                let h = args.get(2).map(|a| a.to_float()).unwrap_or(21.0);
                self.actions.push(ScriptAction::SetTerrainHeight { x, y, height: h });
                Ok(LSLValue::Integer(0))
            }
            "osSetTerrainTexture" => {
                let corner = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let tex = args.get(1).map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::SetTerrainTexture { corner, texture_id: tex });
                Ok(LSLValue::Integer(0))
            }
            "osSetTerrainTextureHeight" => {
                let corner = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let lo = args.get(1).map(|a| a.to_float()).unwrap_or(10.0);
                let hi = args.get(2).map(|a| a.to_float()).unwrap_or(60.0);
                self.actions.push(ScriptAction::SetTerrainTextureHeight { corner, low: lo, high: hi });
                Ok(LSLValue::Integer(0))
            }
            "osSetRegionWaterHeight" => {
                let h = args.first().map(|a| a.to_float()).unwrap_or(20.0);
                self.actions.push(ScriptAction::SetRegionWaterHeight { height: h });
                Ok(LSLValue::Integer(0))
            }
            "osSetSunParam" => {
                let param = args.first().map(|a| a.to_string()).unwrap_or_default();
                let val = args.get(1).map(|a| a.to_float()).unwrap_or(0.0);
                self.actions.push(ScriptAction::SetSunParam { param, value: val });
                Ok(LSLValue::Integer(0))
            }
            "osSetWindParam" => {
                let param = args.first().map(|a| a.to_string()).unwrap_or_default();
                let val = args.get(1).map(|a| a.to_float()).unwrap_or(0.0);
                self.actions.push(ScriptAction::SetWindParam { param, value: val });
                Ok(LSLValue::Integer(0))
            }
            "osGetTerrainHeight" => {
                let _x = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let _y = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                Ok(LSLValue::Float(21.0))
            }
            "osGetCurrentSunHour" | "osGetSunParam" => Ok(LSLValue::Float(6.0)),

            // ==================== OSSL: Avatar Management ====================
            "osKickAvatar" => {
                let avatar = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let msg = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::KickAvatar { avatar_id: avatar, message: msg });
                Ok(LSLValue::Integer(0))
            }
            "osTeleportAgent" => {
                let avatar = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let sim = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                let pos = args.get(2).map(|a| a.to_vector()).unwrap_or(LSLVector::new(128.0, 128.0, 21.0));
                let lookat = args.get(3).map(|a| a.to_vector()).unwrap_or(LSLVector::new(0.0, 1.0, 0.0));
                self.actions.push(ScriptAction::TeleportAgent { agent_id: avatar, landmark: sim, position: [pos.x as f32, pos.y as f32, pos.z as f32], look_at: [lookat.x as f32, lookat.y as f32, lookat.z as f32] });
                Ok(LSLValue::Integer(0))
            }
            "osSetSpeed" => {
                let avatar = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let speed = args.get(1).map(|a| a.to_float()).unwrap_or(1.0);
                self.actions.push(ScriptAction::SetSpeed { avatar_id: avatar, speed });
                Ok(LSLValue::Integer(0))
            }
            "osForceAttachToAvatar" => {
                let point = args.first().map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::ForceAttachToAvatar { object_id: self.ctx.object_id, avatar_id: self.ctx.owner_id, attach_point: point });
                Ok(LSLValue::Integer(0))
            }
            "osForceDetachFromAvatar" => {
                self.actions.push(ScriptAction::ForceDetachFromAvatar { object_id: self.ctx.object_id, avatar_id: self.ctx.owner_id });
                Ok(LSLValue::Integer(0))
            }
            "osForceDropAttachment" => {
                self.actions.push(ScriptAction::ForceDetachFromAvatar { object_id: self.ctx.object_id, avatar_id: self.ctx.owner_id });
                Ok(LSLValue::Integer(0))
            }
            "osForceOtherSit" => {
                let avatar = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let target = args.get(1).map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::ForceOtherSit { avatar_id: avatar, target_id: target });
                Ok(LSLValue::Integer(0))
            }
            "osRegionNotice" => {
                let msg = args.first().map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::RegionNotice { message: msg });
                Ok(LSLValue::Integer(0))
            }
            "osRegionRestart" => {
                let secs = args.first().map(|a| a.to_integer()).unwrap_or(120);
                self.actions.push(ScriptAction::RegionRestart { seconds: secs, message: "Region restart requested by script".into() });
                Ok(LSLValue::Integer(0))
            }
            "osAgentSaveAppearance" => {
                let avatar = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let nc = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::AgentSaveAppearance { avatar_id: avatar, notecard: nc });
                Ok(LSLValue::Key(Uuid::new_v4()))
            }
            "osForceAttachToOtherAvatarFromInventory" => {
                let target = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let _item = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                let point = args.get(2).map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::ForceAttachToOtherAvatar { object_id: self.ctx.object_id, avatar_id: target, attach_point: point });
                Ok(LSLValue::Integer(0))
            }
            "osAvatarPlayAnimation" => {
                let avatar = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let anim = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::AvatarPlayAnimation { avatar_id: avatar, anim_name: anim });
                Ok(LSLValue::Integer(0))
            }
            "osAvatarStopAnimation" => {
                let avatar = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let anim = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::AvatarStopAnimation { avatar_id: avatar, anim_name: anim });
                Ok(LSLValue::Integer(0))
            }
            "osMessageObject" => {
                let target = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let msg = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::MessageObject { target_id: target, message: msg });
                Ok(LSLValue::Integer(0))
            }
            "osGetAvatarList" => {
                Ok(LSLValue::List(vec![]))
            }
            "osGetNumberOfNotecardLines" => {
                Ok(LSLValue::Integer(0))
            }

            // ==================== OSSL: Object Animations ====================
            "llStartObjectAnimation" => {
                let anim = args.first().map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::StartObjectAnimation { object_id: self.ctx.object_id, anim_name: anim });
                Ok(LSLValue::Integer(0))
            }
            "llStopObjectAnimation" => {
                let anim = args.first().map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::StopObjectAnimation { object_id: self.ctx.object_id, anim_name: anim });
                Ok(LSLValue::Integer(0))
            }
            "llGetObjectAnimationNames" => Ok(LSLValue::List(vec![])),

            // ==================== OSSL: Projection / Media ====================
            "osSetProjectionParams" => {
                let enabled = args.first().map(|a| a.to_integer() != 0).unwrap_or(false);
                let tex = args.get(1).map(|a| a.to_key()).unwrap_or(Uuid::nil());
                let fov = args.get(2).map(|a| a.to_float()).unwrap_or(0.0);
                let focus = args.get(3).map(|a| a.to_float()).unwrap_or(0.0);
                let amb = args.get(4).map(|a| a.to_float()).unwrap_or(0.0);
                self.actions.push(ScriptAction::SetProjectionParams { object_id: self.ctx.object_id, enabled, texture_id: tex, fov, focus, ambiance: amb });
                Ok(LSLValue::Integer(0))
            }
            "llModifyLand" => {
                let action = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let brush_size = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::ModifyLand { object_id: self.ctx.object_id, action, brush_size });
                Ok(LSLValue::Integer(0))
            }
            "llSetLinkCamera" => {
                let link = args.first().map(|a| a.to_integer()).unwrap_or(0);
                let eye_v = args.get(1).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let at_v = args.get(2).map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                self.actions.push(ScriptAction::SetLinkCamera { object_id: self.ctx.object_id, link_num: link, eye: [eye_v.x as f32, eye_v.y as f32, eye_v.z as f32], at: [at_v.x as f32, at_v.y as f32, at_v.z as f32] });
                Ok(LSLValue::Integer(0))
            }
            "osSetParcelDetails" => {
                let pos = args.first().map(|a| a.to_vector()).unwrap_or(LSLVector::zero());
                let params_list = args.get(1).map(|a| a.to_list()).unwrap_or_default();
                let mut params = Vec::new();
                let mut i = 0;
                while i + 1 < params_list.len() {
                    let code = params_list[i].to_integer();
                    let val = params_list[i + 1].to_string();
                    params.push((code, val));
                    i += 2;
                }
                self.actions.push(ScriptAction::SetParcelDetails { position: [pos.x as f32, pos.y as f32, pos.z as f32], params });
                Ok(LSLValue::Integer(0))
            }

            // ==================== OSSL: Misc Info ====================
            "osGetSimulatorVersion" => Ok(LSLValue::String("OpenSim Next 0.9.3".into())),
            "osGetScriptEngineName" => Ok(LSLValue::String("YEngine".into())),
            "osGetSimulatorMemory" | "osGetSimulatorMemoryKB" => Ok(LSLValue::Integer(1048576)),
            "osGetRegionSize" => Ok(LSLValue::Vector(LSLVector::new(256.0, 256.0, 4096.0))),
            "osGetMapTexture" | "osGetRegionMapTexture" => Ok(LSLValue::Key(Uuid::nil())),
            "osGetGridName" => Ok(LSLValue::String("Gaia Grid".into())),
            "osGetGridLoginURI" | "osGetGridHomeURI" | "osGetGridGatekeeperURI" | "osGetGridCustom" => {
                Ok(LSLValue::String(String::new()))
            }
            "osGetAgentIP" => Ok(LSLValue::String("127.0.0.1".into())),
            "osInviteToGroup" => Ok(LSLValue::Integer(1)),

            // ==================== OSSL: Drawing (state-only, no rendering yet) ====================
            "osDrawLine" | "osDrawRectangle" | "osDrawFilledRectangle"
            | "osDrawEllipse" | "osDrawFilledEllipse" | "osDrawText"
            | "osSetPenColor" | "osSetPenSize" | "osSetPenCap"
            | "osSetFontName" | "osSetFontSize"
            | "osSetDynamicTextureData" | "osSetDynamicTextureURL"
            | "osMovePen" | "osDrawImage" | "osSetDynamicTextureDataBlend"
            | "osSetDynamicTextureDataBlendFace" => Ok(LSLValue::String(String::new())),
            "osGetNotecard" | "osGetNotecardLine" => Ok(LSLValue::String(String::new())),
            "osGetPhysicsEngineType" | "osGetPhysicsEngineName" => Ok(LSLValue::String("ODE".into())),
            "osOwnerSaveAppearance" | "osAgentSaveAppearance" if args.is_empty() => {
                Ok(LSLValue::Key(Uuid::nil()))
            }
            "osOwnerSaveAppearance" => {
                let nc = args.first().map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::AgentSaveAppearance { avatar_id: self.ctx.owner_id, notecard: nc });
                Ok(LSLValue::Key(Uuid::new_v4()))
            }
            "osNpcStopMoveToTarget" => {
                let npc_id = args.first().map(|a| a.to_key()).unwrap_or(Uuid::nil());
                self.actions.push(ScriptAction::NpcMoveTo { npc_id, position: [0.0, 0.0, 0.0], options: -1 });
                Ok(LSLValue::Integer(0))
            }
            "osIsUUID" => {
                let s = args.first().map(|a| a.to_string()).unwrap_or_default();
                let valid = uuid::Uuid::parse_str(&s).is_ok();
                Ok(LSLValue::Integer(if valid { 1 } else { 0 }))
            }
            "osGetAgents" => Ok(LSLValue::List(vec![])),
            "osGetDrawStringSize" => Ok(LSLValue::Vector(LSLVector::new(100.0, 20.0, 0.0))),
            "osSetPrimitiveParams" | "osGetPrimitiveParams" => Ok(LSLValue::List(vec![])),
            "osGetInventoryDesc" => {
                let _item = args.first().map(|a| a.to_string()).unwrap_or_default();
                Ok(LSLValue::String(String::new()))
            }
            "osForceAttachToAvatarFromInventory" => {
                let _item = args.first().map(|a| a.to_string()).unwrap_or_default();
                let point = args.get(1).map(|a| a.to_integer()).unwrap_or(0);
                self.actions.push(ScriptAction::ForceAttachToAvatar { object_id: self.ctx.object_id, avatar_id: self.ctx.owner_id, attach_point: point });
                Ok(LSLValue::Integer(0))
            }
            "llGetParcelMusicURL" => Ok(LSLValue::String(String::new())),
            "osMakeNotecard" => {
                let nc_name = args.first().map(|a| a.to_string()).unwrap_or_default();
                let contents = args.get(1).map(|a| a.to_string()).unwrap_or_default();
                self.actions.push(ScriptAction::MakeNotecard { name: nc_name, contents, object_id: self.ctx.object_id });
                Ok(LSLValue::Integer(0))
            }

            // ==================== CATCH-ALL ====================
            _ => {
                Ok(LSLValue::Integer(0))
            }
        }
    }

    fn encode_particle_system_inline(rules: &[LSLValue]) -> Vec<u8> {
        if rules.is_empty() { return Vec::new(); }
        let pfu16 = |val: f32, ib: u32, fb: u32| -> u16 {
            let mx = ((1u32 << (ib + fb)) - 1) as f32;
            (val * (1u32 << fb) as f32).clamp(0.0, mx) as u16
        };
        let pfs16 = |val: f32, ib: u32, fb: u32| -> u16 {
            let t = ib + fb + 1;
            let mx = ((1u32 << (t - 1)) - 1) as f32;
            let mn = -((1u32 << (t - 1)) as f32);
            ((val * (1u32 << fb) as f32).clamp(mn, mx) as i16) as u16
        };
        let pfu8 = |val: f32, ib: u32, fb: u32| -> u8 {
            let mx = ((1u32 << (ib + fb)) - 1) as f32;
            (val * (1u32 << fb) as f32).clamp(0.0, mx) as u8
        };
        let (mut pdf, mut pat, mut ma, mut sa): (u32, u8, f32, f32) = (0, 0x02, 10.0, 0.0);
        let (mut ia, mut oa, mut br, mut brad): (f32, f32, f32, f32) = (0.0, 0.0, 0.1, 0.0);
        let (mut bsn, mut bsx, mut bpc): (f32, f32, u8) = (1.0, 1.0, 1);
        let (mut av, mut ac) = ([0.0f32; 3], [0.0f32; 3]);
        let (mut tex, mut tgt) = (Uuid::nil(), Uuid::nil());
        let mut pma: f32 = 10.0;
        let (mut sc, mut ec) = ([1.0f32; 4], [1.0f32, 1.0, 1.0, 0.0]);
        let (mut ss, mut es) = ([1.0f32; 2], [1.0f32; 2]);
        let (mut sg, mut eg): (f32, f32) = (0.0, 0.0);
        let (mut bs, mut bd): (u8, u8) = (7, 9);
        let (mut hg, mut hb) = (false, false);
        let mut i = 0;
        while i + 1 < rules.len() {
            let c = rules[i].to_integer(); i += 1;
            match c {
                0 => { pdf = rules[i].to_integer() as u32; i += 1; }
                1 => { let v = rules[i].to_vector(); sc = [v.x, v.y, v.z, sc[3]]; i += 1; }
                2 => { sc[3] = rules[i].to_float(); i += 1; }
                3 => { let v = rules[i].to_vector(); ec = [v.x, v.y, v.z, ec[3]]; i += 1; }
                4 => { ec[3] = rules[i].to_float(); i += 1; }
                5 => { let v = rules[i].to_vector(); ss = [v.x, v.y]; i += 1; }
                6 => { let v = rules[i].to_vector(); es = [v.x, v.y]; i += 1; }
                7 => { pma = rules[i].to_float(); i += 1; }
                8 => { let v = rules[i].to_vector(); ac = [v.x, v.y, v.z]; i += 1; }
                9 => { pat = rules[i].to_integer() as u8; i += 1; }
                12 => { tex = Uuid::parse_str(&rules[i].to_string()).unwrap_or(Uuid::nil()); i += 1; }
                13 => { br = rules[i].to_float(); i += 1; }
                15 => { bpc = rules[i].to_integer().clamp(0, 255) as u8; i += 1; }
                16 => { brad = rules[i].to_float(); i += 1; }
                17 => { bsn = rules[i].to_float(); i += 1; }
                18 => { bsx = rules[i].to_float(); i += 1; }
                19 => { ma = rules[i].to_float(); i += 1; }
                20 => { tgt = rules[i].to_key(); i += 1; }
                21 => { let v = rules[i].to_vector(); av = [v.x, v.y, v.z]; i += 1; }
                22 => { ia = rules[i].to_float(); i += 1; }
                23 => { oa = rules[i].to_float(); i += 1; }
                24 => { bs = rules[i].to_integer() as u8; hb = true; i += 1; }
                25 => { bd = rules[i].to_integer() as u8; hb = true; i += 1; }
                26 => { sg = rules[i].to_float(); hg = true; i += 1; }
                27 => { eg = rules[i].to_float(); hg = true; i += 1; }
                _ => { i += 1; }
            }
        }
        if hg { pdf |= 0x10000; }
        if hb { pdf |= 0x20000; }
        let pds: u32 = 18 + if hg { 2 } else { 0 } + if hb { 2 } else { 0 };
        let mut b = Vec::with_capacity(92);
        b.extend_from_slice(&68u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.push(pat);
        b.extend_from_slice(&pfu16(ma, 8, 8).to_le_bytes());
        b.extend_from_slice(&pfu16(sa, 8, 8).to_le_bytes());
        b.push(pfu8(ia, 3, 5));
        b.push(pfu8(oa, 3, 5));
        b.extend_from_slice(&pfu16(br, 8, 8).to_le_bytes());
        b.extend_from_slice(&pfu16(brad, 8, 8).to_le_bytes());
        b.extend_from_slice(&pfu16(bsn, 8, 8).to_le_bytes());
        b.extend_from_slice(&pfu16(bsx, 8, 8).to_le_bytes());
        b.push(bpc);
        for v in &av { b.extend_from_slice(&pfs16(*v, 8, 7).to_le_bytes()); }
        for v in &ac { b.extend_from_slice(&pfs16(*v, 8, 7).to_le_bytes()); }
        b.extend_from_slice(tex.as_bytes());
        b.extend_from_slice(tgt.as_bytes());
        b.extend_from_slice(&pds.to_le_bytes());
        b.extend_from_slice(&pdf.to_le_bytes());
        b.extend_from_slice(&pfu16(pma, 8, 8).to_le_bytes());
        for c in &sc { b.push((*c * 255.0).clamp(0.0, 255.0) as u8); }
        for c in &ec { b.push((*c * 255.0).clamp(0.0, 255.0) as u8); }
        b.push(pfu8(ss[0], 3, 5)); b.push(pfu8(ss[1], 3, 5));
        b.push(pfu8(es[0], 3, 5)); b.push(pfu8(es[1], 3, 5));
        if hg { b.push((sg * 255.0).clamp(0.0, 255.0) as u8); b.push((eg * 255.0).clamp(0.0, 255.0) as u8); }
        if hb { b.push(bs); b.push(bd); }
        b
    }

    fn binary_op(&self, left: &LSLValue, op: &Token, right: &LSLValue) -> Result<LSLValue> {
        match op {
            Token::Plus => match (left, right) {
                (LSLValue::Integer(a), LSLValue::Integer(b)) => Ok(LSLValue::Integer(a.wrapping_add(*b))),
                (LSLValue::Float(a), LSLValue::Float(b)) => Ok(LSLValue::Float(a + b)),
                (LSLValue::Integer(a), LSLValue::Float(b)) => Ok(LSLValue::Float(*a as f32 + b)),
                (LSLValue::Float(a), LSLValue::Integer(b)) => Ok(LSLValue::Float(a + *b as f32)),
                (LSLValue::String(a), LSLValue::String(b)) => Ok(LSLValue::String(format!("{}{}", a, b))),
                (LSLValue::Vector(a), LSLValue::Vector(b)) => Ok(LSLValue::Vector(*a + *b)),
                (LSLValue::List(a), LSLValue::List(b)) => {
                    let mut result = a.clone();
                    result.extend(b.iter().cloned());
                    Ok(LSLValue::List(result))
                }
                (LSLValue::List(a), other) => {
                    let mut result = a.clone();
                    result.push(other.clone());
                    Ok(LSLValue::List(result))
                }
                (other, LSLValue::List(b)) => {
                    let mut result = vec![other.clone()];
                    result.extend(b.iter().cloned());
                    Ok(LSLValue::List(result))
                }
                _ => Err(anyhow!("Cannot add these types")),
            },
            Token::Minus => match (left, right) {
                (LSLValue::Integer(a), LSLValue::Integer(b)) => Ok(LSLValue::Integer(a.wrapping_sub(*b))),
                (LSLValue::Float(a), LSLValue::Float(b)) => Ok(LSLValue::Float(a - b)),
                (LSLValue::Integer(a), LSLValue::Float(b)) => Ok(LSLValue::Float(*a as f32 - b)),
                (LSLValue::Float(a), LSLValue::Integer(b)) => Ok(LSLValue::Float(a - *b as f32)),
                (LSLValue::Vector(a), LSLValue::Vector(b)) => Ok(LSLValue::Vector(*a - *b)),
                _ => Err(anyhow!("Cannot subtract these types")),
            },
            Token::Multiply => match (left, right) {
                (LSLValue::Integer(a), LSLValue::Integer(b)) => Ok(LSLValue::Integer(a.wrapping_mul(*b))),
                (LSLValue::Float(a), LSLValue::Float(b)) => Ok(LSLValue::Float(a * b)),
                (LSLValue::Integer(a), LSLValue::Float(b)) => Ok(LSLValue::Float(*a as f32 * b)),
                (LSLValue::Float(a), LSLValue::Integer(b)) => Ok(LSLValue::Float(a * *b as f32)),
                (LSLValue::Vector(a), LSLValue::Float(b)) => Ok(LSLValue::Vector(*a * *b)),
                (LSLValue::Float(a), LSLValue::Vector(b)) => Ok(LSLValue::Vector(*b * *a)),
                (LSLValue::Rotation(a), LSLValue::Rotation(b)) => Ok(LSLValue::Rotation(*a * *b)),
                _ => Err(anyhow!("Cannot multiply these types")),
            },
            Token::Divide => {
                match (left, right) {
                    (LSLValue::Integer(a), LSLValue::Integer(b)) => {
                        if *b == 0 { Ok(LSLValue::Integer(0)) }
                        else { Ok(LSLValue::Integer(a / b)) }
                    }
                    (LSLValue::Float(a), LSLValue::Float(b)) => {
                        if *b == 0.0 { Ok(LSLValue::Float(0.0)) }
                        else { Ok(LSLValue::Float(a / b)) }
                    }
                    (LSLValue::Integer(a), LSLValue::Float(b)) => {
                        if *b == 0.0 { Ok(LSLValue::Float(0.0)) }
                        else { Ok(LSLValue::Float(*a as f32 / b)) }
                    }
                    (LSLValue::Float(a), LSLValue::Integer(b)) => {
                        if *b == 0 { Ok(LSLValue::Float(0.0)) }
                        else { Ok(LSLValue::Float(a / *b as f32)) }
                    }
                    (LSLValue::Vector(a), LSLValue::Float(b)) => {
                        if *b == 0.0 { Ok(LSLValue::Vector(LSLVector::zero())) }
                        else { Ok(LSLValue::Vector(*a / *b)) }
                    }
                    _ => Err(anyhow!("Cannot divide these types")),
                }
            }
            Token::Modulo => match (left, right) {
                (LSLValue::Integer(a), LSLValue::Integer(b)) => {
                    if *b == 0 { Ok(LSLValue::Integer(0)) }
                    else { Ok(LSLValue::Integer(a % b)) }
                }
                _ => Err(anyhow!("Cannot modulo these types")),
            },
            Token::Equal => Ok(LSLValue::Integer(if self.values_equal(left, right) { 1 } else { 0 })),
            Token::NotEqual => Ok(LSLValue::Integer(if !self.values_equal(left, right) { 1 } else { 0 })),
            Token::Less => Ok(LSLValue::Integer(if left.to_float() < right.to_float() { 1 } else { 0 })),
            Token::Greater => Ok(LSLValue::Integer(if left.to_float() > right.to_float() { 1 } else { 0 })),
            Token::LessEqual => Ok(LSLValue::Integer(if left.to_float() <= right.to_float() { 1 } else { 0 })),
            Token::GreaterEqual => Ok(LSLValue::Integer(if left.to_float() >= right.to_float() { 1 } else { 0 })),
            Token::And => Ok(LSLValue::Integer(if left.is_true() && right.is_true() { 1 } else { 0 })),
            Token::Or => Ok(LSLValue::Integer(if left.is_true() || right.is_true() { 1 } else { 0 })),
            Token::BitAnd => Ok(LSLValue::Integer(left.to_integer() & right.to_integer())),
            Token::BitOr => Ok(LSLValue::Integer(left.to_integer() | right.to_integer())),
            Token::BitXor => Ok(LSLValue::Integer(left.to_integer() ^ right.to_integer())),
            Token::ShiftLeft => Ok(LSLValue::Integer(left.to_integer() << (right.to_integer() & 31))),
            Token::ShiftRight => Ok(LSLValue::Integer(left.to_integer() >> (right.to_integer() & 31))),
            _ => Err(anyhow!("Unsupported binary operator: {:?}", op)),
        }
    }

    fn unary_op(&self, op: &Token, val: &LSLValue) -> Result<LSLValue> {
        match op {
            Token::Minus => match val {
                LSLValue::Integer(i) => Ok(LSLValue::Integer(-i)),
                LSLValue::Float(f) => Ok(LSLValue::Float(-f)),
                LSLValue::Vector(v) => Ok(LSLValue::Vector(LSLVector { x: -v.x, y: -v.y, z: -v.z })),
                LSLValue::Rotation(r) => Ok(LSLValue::Rotation(LSLRotation { x: -r.x, y: -r.y, z: -r.z, s: -r.s })),
                _ => Err(anyhow!("Cannot negate this type")),
            },
            Token::Not => Ok(LSLValue::Integer(if val.is_true() { 0 } else { 1 })),
            Token::BitNot => Ok(LSLValue::Integer(!val.to_integer())),
            _ => Err(anyhow!("Unsupported unary operator")),
        }
    }

    fn values_equal(&self, left: &LSLValue, right: &LSLValue) -> bool {
        match (left, right) {
            (LSLValue::Integer(a), LSLValue::Integer(b)) => a == b,
            (LSLValue::Float(a), LSLValue::Float(b)) => (a - b).abs() < f32::EPSILON,
            (LSLValue::String(a), LSLValue::String(b)) => a == b,
            (LSLValue::Key(a), LSLValue::Key(b)) => a == b,
            (LSLValue::Vector(a), LSLValue::Vector(b)) => a == b,
            (LSLValue::Rotation(a), LSLValue::Rotation(b)) => a == b,
            (LSLValue::Key(a), LSLValue::String(b)) => a.to_string() == *b,
            (LSLValue::String(a), LSLValue::Key(b)) => *a == b.to_string(),
            (LSLValue::Integer(a), LSLValue::Float(b)) => (*a as f32 - b).abs() < f32::EPSILON,
            (LSLValue::Float(a), LSLValue::Integer(b)) => (a - *b as f32).abs() < f32::EPSILON,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_script() -> Result<()> {
        let executor = TreeWalkExecutor::new();
        let source = r#"
default
{
    state_entry()
    {
        integer x = 42;
    }
}
"#;
        let compiled = executor.compile(source, Uuid::new_v4())?;
        assert!(compiled.states.contains_key("default"));
        assert!(compiled.states["default"].events.contains_key("state_entry"));
        Ok(())
    }

    #[test]
    fn test_execute_simple_event() -> Result<()> {
        let executor = TreeWalkExecutor::new();
        let source = r#"
integer g = 10;

default
{
    state_entry()
    {
        g = g + 5;
    }
}
"#;
        let compiled = executor.compile(source, Uuid::new_v4())?;
        let mut instance = ScriptInstance::new(Uuid::new_v4(), compiled, 65536);

        let result = executor.execute_event(&mut instance, "state_entry", &[])?;
        assert!(matches!(result, ExecutionResult::Complete(_)));
        assert_eq!(instance.global_vars.get("g"), Some(&LSLValue::Integer(15)));
        Ok(())
    }

    #[test]
    fn test_function_call() -> Result<()> {
        let executor = TreeWalkExecutor::new();
        let source = r#"
integer add(integer a, integer b)
{
    return a + b;
}

integer result = 0;

default
{
    state_entry()
    {
        result = add(3, 7);
    }
}
"#;
        let compiled = executor.compile(source, Uuid::new_v4())?;
        let mut instance = ScriptInstance::new(Uuid::new_v4(), compiled, 65536);

        executor.execute_event(&mut instance, "state_entry", &[])?;
        assert_eq!(instance.global_vars.get("result"), Some(&LSLValue::Integer(10)));
        Ok(())
    }

    #[test]
    fn test_while_loop() -> Result<()> {
        let executor = TreeWalkExecutor::new();
        let source = r#"
integer count = 0;

default
{
    state_entry()
    {
        integer i = 0;
        while (i < 5)
        {
            count = count + 1;
            i = i + 1;
        }
    }
}
"#;
        let compiled = executor.compile(source, Uuid::new_v4())?;
        let mut instance = ScriptInstance::new(Uuid::new_v4(), compiled, 65536);

        executor.execute_event(&mut instance, "state_entry", &[])?;
        assert_eq!(instance.global_vars.get("count"), Some(&LSLValue::Integer(5)));
        Ok(())
    }

    #[test]
    fn test_vector_literal() -> Result<()> {
        let executor = TreeWalkExecutor::new();
        let source = r#"
vector pos = <0.0, 0.0, 0.0>;

default
{
    state_entry()
    {
        pos = <1.0, 2.0, 3.0>;
    }
}
"#;
        let compiled = executor.compile(source, Uuid::new_v4())?;
        let mut instance = ScriptInstance::new(Uuid::new_v4(), compiled, 65536);

        executor.execute_event(&mut instance, "state_entry", &[])?;
        if let Some(LSLValue::Vector(v)) = instance.global_vars.get("pos") {
            assert!((v.x - 1.0).abs() < f32::EPSILON);
            assert!((v.y - 2.0).abs() < f32::EPSILON);
            assert!((v.z - 3.0).abs() < f32::EPSILON);
        } else {
            panic!("Expected vector value");
        }
        Ok(())
    }
}
