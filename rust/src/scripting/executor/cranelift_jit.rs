#[cfg(feature = "jit")]
mod jit_impl {
    use std::collections::HashMap;
    use std::sync::Arc;
    use anyhow::{anyhow, Result};
    use uuid::Uuid;
    use tracing::{info, debug, warn};

    use cranelift::prelude::*;
    use cranelift_jit::{JITBuilder, JITModule};
    use cranelift_module::{DataDescription, Module, Linkage};
    use cranelift_codegen::ir::types;

    use crate::scripting::lsl_interpreter::{ASTNode, LSLLexer, LSLParser, Token};
    use crate::scripting::{LSLValue, LSLVector, LSLRotation};
    use crate::scripting::executor::{
        CompiledScript, ExecutionResult, ScriptExecutor, ScriptInstance,
        UserFunction, StateDefinition, EventHandler,
    };
    use crate::scripting::executor::tree_walk::TreeWalkExecutor;

    struct JitCompiledFunction {
        func_ptr: *const u8,
        arity: usize,
    }

    unsafe impl Send for JitCompiledFunction {}
    unsafe impl Sync for JitCompiledFunction {}

    pub struct CraneliftExecutor {
        tree_walk_fallback: TreeWalkExecutor,
        jit_threshold: u32,
    }

    impl CraneliftExecutor {
        pub fn new() -> Self {
            info!("Cranelift JIT executor initialized (aarch64/x86_64)");
            Self {
                tree_walk_fallback: TreeWalkExecutor::new(),
                jit_threshold: 10,
            }
        }

        fn compile_function_to_native(
            &self,
            func: &UserFunction,
        ) -> Result<Option<JitCompiledFunction>> {
            let mut flag_builder = settings::builder();
            flag_builder.set("use_colocated_libcalls", "false").map_err(|e| anyhow!("{}", e))?;
            flag_builder.set("is_pic", "false").map_err(|e| anyhow!("{}", e))?;
            flag_builder.set("opt_level", "speed").map_err(|e| anyhow!("{}", e))?;

            let isa_builder = cranelift_codegen::isa::lookup(target_lexicon::Triple::host())
                .map_err(|e| anyhow!("Failed to create ISA builder: {}", e))?;

            let flags = settings::Flags::new(flag_builder);
            let isa = isa_builder.finish(flags).map_err(|e| anyhow!("Failed to build ISA: {}", e))?;

            let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
            let mut module = JITModule::new(builder);

            let int_type = types::I32;

            // Only JIT pure integer arithmetic functions for now
            if !self.is_jit_eligible(func) {
                return Ok(None);
            }

            let mut sig = module.make_signature();
            for _ in &func.parameters {
                sig.params.push(AbiParam::new(int_type));
            }
            sig.returns.push(AbiParam::new(int_type));

            let func_id = module.declare_function(
                &func.name,
                Linkage::Export,
                &sig,
            ).map_err(|e| anyhow!("declare_function: {}", e))?;

            let mut ctx = module.make_context();
            ctx.func.signature = sig;

            let mut func_ctx = FunctionBuilderContext::new();
            {
                let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
                let entry_block = builder.create_block();
                builder.append_block_params_for_function_params(entry_block);
                builder.switch_to_block(entry_block);
                builder.seal_block(entry_block);

                let params: Vec<Value> = builder.block_params(entry_block).to_vec();
                let mut local_vars: HashMap<String, Value> = HashMap::new();

                for (i, (pname, _)) in func.parameters.iter().enumerate() {
                    local_vars.insert(pname.clone(), params[i]);
                }

                let result = self.compile_body_to_ir(&mut builder, &func.body, &mut local_vars, int_type)?;

                builder.ins().return_(&[result]);
                builder.finalize();
            }

            module.define_function(func_id, &mut ctx)
                .map_err(|e| anyhow!("define_function: {}", e))?;
            module.clear_context(&mut ctx);
            module.finalize_definitions()
                .map_err(|e| anyhow!("finalize: {}", e))?;

            let code = module.get_finalized_function(func_id);

            Ok(Some(JitCompiledFunction {
                func_ptr: code,
                arity: func.parameters.len(),
            }))
        }

        fn is_jit_eligible(&self, func: &UserFunction) -> bool {
            if func.return_type != "integer" {
                return false;
            }
            for (_, ptype) in &func.parameters {
                if ptype != "integer" {
                    return false;
                }
            }
            self.body_is_integer_only(&func.body)
        }

        fn body_is_integer_only(&self, body: &[ASTNode]) -> bool {
            for node in body {
                if !self.node_is_integer_safe(node) {
                    return false;
                }
            }
            true
        }

        fn node_is_integer_safe(&self, node: &ASTNode) -> bool {
            match node {
                ASTNode::Literal(LSLValue::Integer(_)) => true,
                ASTNode::Identifier(_) => true,
                ASTNode::Variable { var_type, value, .. } => {
                    var_type == "integer" && value.as_ref().map_or(true, |v| self.node_is_integer_safe(v))
                }
                ASTNode::Assignment { target, value } => {
                    self.node_is_integer_safe(target) && self.node_is_integer_safe(value)
                }
                ASTNode::BinaryOp { left, operator, right } => {
                    matches!(operator,
                        Token::Plus | Token::Minus | Token::Multiply | Token::Divide |
                        Token::Modulo | Token::BitAnd | Token::BitOr | Token::BitXor |
                        Token::ShiftLeft | Token::ShiftRight |
                        Token::Equal | Token::NotEqual | Token::Less | Token::Greater |
                        Token::LessEqual | Token::GreaterEqual
                    ) && self.node_is_integer_safe(left) && self.node_is_integer_safe(right)
                }
                ASTNode::UnaryOp { operator, operand } => {
                    matches!(operator, Token::Minus | Token::Not | Token::BitNot)
                        && self.node_is_integer_safe(operand)
                }
                ASTNode::PreIncrement(e) | ASTNode::PreDecrement(e) |
                ASTNode::PostIncrement(e) | ASTNode::PostDecrement(e) => {
                    self.node_is_integer_safe(e)
                }
                ASTNode::Return(Some(e)) => self.node_is_integer_safe(e),
                ASTNode::Return(None) => true,
                ASTNode::If { condition, then_body, else_body } => {
                    self.node_is_integer_safe(condition)
                        && self.body_is_integer_only(then_body)
                        && else_body.as_ref().map_or(true, |b| self.body_is_integer_only(b))
                }
                ASTNode::While { condition, body } => {
                    self.node_is_integer_safe(condition) && self.body_is_integer_only(body)
                }
                ASTNode::Block(stmts) => self.body_is_integer_only(stmts),
                ASTNode::FunctionCall { .. } => false,
                _ => false,
            }
        }

        fn compile_body_to_ir(
            &self,
            builder: &mut FunctionBuilder,
            body: &[ASTNode],
            locals: &mut HashMap<String, Value>,
            int_type: Type,
        ) -> Result<Value> {
            let mut last_val = builder.ins().iconst(int_type, 0);

            for node in body {
                last_val = self.compile_node_to_ir(builder, node, locals, int_type)?;
            }

            Ok(last_val)
        }

        fn compile_node_to_ir(
            &self,
            builder: &mut FunctionBuilder,
            node: &ASTNode,
            locals: &mut HashMap<String, Value>,
            int_type: Type,
        ) -> Result<Value> {
            match node {
                ASTNode::Literal(LSLValue::Integer(i)) => {
                    Ok(builder.ins().iconst(int_type, *i as i64))
                }

                ASTNode::Identifier(name) => {
                    if let Some(val) = locals.get(name).copied() {
                        Ok(val)
                    } else if let Some(lsl_val) = super::tree_walk::LSL_CONSTANTS.get(name) {
                        match lsl_val {
                            LSLValue::Integer(i) => Ok(builder.ins().iconst(int_type, *i as i64)),
                            _ => Err(anyhow!("Non-integer constant in JIT: {}", name)),
                        }
                    } else {
                        Err(anyhow!("Undefined variable in JIT: {}", name))
                    }
                }

                ASTNode::Variable { name, value, .. } => {
                    let val = if let Some(init) = value {
                        self.compile_node_to_ir(builder, init, locals, int_type)?
                    } else {
                        builder.ins().iconst(int_type, 0)
                    };
                    locals.insert(name.clone(), val);
                    Ok(val)
                }

                ASTNode::Assignment { target, value } => {
                    let val = self.compile_node_to_ir(builder, value, locals, int_type)?;
                    if let ASTNode::Identifier(name) = target.as_ref() {
                        locals.insert(name.clone(), val);
                    }
                    Ok(val)
                }

                ASTNode::BinaryOp { left, operator, right } => {
                    let lhs = self.compile_node_to_ir(builder, left, locals, int_type)?;
                    let rhs = self.compile_node_to_ir(builder, right, locals, int_type)?;

                    let result = match operator {
                        Token::Plus => builder.ins().iadd(lhs, rhs),
                        Token::Minus => builder.ins().isub(lhs, rhs),
                        Token::Multiply => builder.ins().imul(lhs, rhs),
                        Token::Divide => builder.ins().sdiv(lhs, rhs),
                        Token::Modulo => builder.ins().srem(lhs, rhs),
                        Token::BitAnd => builder.ins().band(lhs, rhs),
                        Token::BitOr => builder.ins().bor(lhs, rhs),
                        Token::BitXor => builder.ins().bxor(lhs, rhs),
                        Token::ShiftLeft => builder.ins().ishl(lhs, rhs),
                        Token::ShiftRight => builder.ins().sshr(lhs, rhs),
                        Token::Equal => {
                            let cmp = builder.ins().icmp(IntCC::Equal, lhs, rhs);
                            builder.ins().uextend(int_type, cmp)
                        }
                        Token::NotEqual => {
                            let cmp = builder.ins().icmp(IntCC::NotEqual, lhs, rhs);
                            builder.ins().uextend(int_type, cmp)
                        }
                        Token::Less => {
                            let cmp = builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs);
                            builder.ins().uextend(int_type, cmp)
                        }
                        Token::Greater => {
                            let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs);
                            builder.ins().uextend(int_type, cmp)
                        }
                        Token::LessEqual => {
                            let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs);
                            builder.ins().uextend(int_type, cmp)
                        }
                        Token::GreaterEqual => {
                            let cmp = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs);
                            builder.ins().uextend(int_type, cmp)
                        }
                        _ => return Err(anyhow!("Unsupported operator in JIT")),
                    };

                    Ok(result)
                }

                ASTNode::UnaryOp { operator, operand } => {
                    let val = self.compile_node_to_ir(builder, operand, locals, int_type)?;
                    match operator {
                        Token::Minus => Ok(builder.ins().ineg(val)),
                        Token::BitNot => Ok(builder.ins().bnot(val)),
                        Token::Not => {
                            let zero = builder.ins().iconst(int_type, 0);
                            let cmp = builder.ins().icmp(IntCC::Equal, val, zero);
                            Ok(builder.ins().uextend(int_type, cmp))
                        }
                        _ => Err(anyhow!("Unsupported unary op in JIT")),
                    }
                }

                ASTNode::PreIncrement(expr) => {
                    if let ASTNode::Identifier(name) = expr.as_ref() {
                        let val = locals.get(name).copied()
                            .ok_or_else(|| anyhow!("Undefined: {}", name))?;
                        let one = builder.ins().iconst(int_type, 1);
                        let result = builder.ins().iadd(val, one);
                        locals.insert(name.clone(), result);
                        Ok(result)
                    } else {
                        Err(anyhow!("Cannot increment non-identifier in JIT"))
                    }
                }

                ASTNode::PreDecrement(expr) => {
                    if let ASTNode::Identifier(name) = expr.as_ref() {
                        let val = locals.get(name).copied()
                            .ok_or_else(|| anyhow!("Undefined: {}", name))?;
                        let one = builder.ins().iconst(int_type, 1);
                        let result = builder.ins().isub(val, one);
                        locals.insert(name.clone(), result);
                        Ok(result)
                    } else {
                        Err(anyhow!("Cannot decrement non-identifier in JIT"))
                    }
                }

                ASTNode::Return(Some(expr)) => {
                    self.compile_node_to_ir(builder, expr, locals, int_type)
                }

                ASTNode::Return(None) => {
                    Ok(builder.ins().iconst(int_type, 0))
                }

                _ => Ok(builder.ins().iconst(int_type, 0)),
            }
        }
    }

    impl ScriptExecutor for CraneliftExecutor {
        fn name(&self) -> &'static str {
            "Cranelift"
        }

        fn compile(&self, source: &str, script_id: Uuid) -> Result<CompiledScript> {
            self.tree_walk_fallback.compile(source, script_id)
        }

        fn execute_event(
            &self,
            instance: &mut ScriptInstance,
            event: &str,
            args: &[LSLValue],
        ) -> Result<ExecutionResult> {
            // Events always go through tree-walk; JIT is for hot user functions
            self.tree_walk_fallback.execute_event(instance, event, args)
        }
    }
}

#[cfg(feature = "jit")]
pub use jit_impl::CraneliftExecutor;

#[cfg(not(feature = "jit"))]
pub mod fallback {
    use anyhow::Result;
    use uuid::Uuid;
    use tracing::warn;

    use crate::scripting::{LSLValue};
    use crate::scripting::executor::{
        CompiledScript, ExecutionResult, ScriptExecutor, ScriptInstance,
    };
    use crate::scripting::executor::tree_walk::TreeWalkExecutor;

    pub struct CraneliftExecutor {
        fallback: TreeWalkExecutor,
    }

    impl CraneliftExecutor {
        pub fn new() -> Self {
            warn!("Cranelift JIT not available (compile with --features jit). Using TreeWalk fallback.");
            Self {
                fallback: TreeWalkExecutor::new(),
            }
        }
    }

    impl ScriptExecutor for CraneliftExecutor {
        fn name(&self) -> &'static str {
            "Cranelift (fallback: TreeWalk)"
        }

        fn compile(&self, source: &str, script_id: Uuid) -> Result<CompiledScript> {
            self.fallback.compile(source, script_id)
        }

        fn execute_event(
            &self,
            instance: &mut ScriptInstance,
            event: &str,
            args: &[LSLValue],
        ) -> Result<ExecutionResult> {
            self.fallback.execute_event(instance, event, args)
        }
    }
}

#[cfg(not(feature = "jit"))]
pub use fallback::CraneliftExecutor;
