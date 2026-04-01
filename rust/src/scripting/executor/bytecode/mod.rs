pub mod opcodes;
pub mod compiler;
pub mod vm;

use std::collections::HashMap;
use anyhow::{anyhow, Result};
use uuid::Uuid;

use super::{
    CompiledScript, ExecutionResult, ScriptExecutor, ScriptInstance,
};
use crate::scripting::lsl_interpreter::{LSLLexer, LSLParser};
use crate::scripting::{LSLValue, LSLVector, LSLRotation};
use compiler::BytecodeCompiler;
use opcodes::BytecodeProgram;
use vm::{Vm, VmResult};

pub struct BytecodeExecutor {
    max_instructions: u64,
}

impl BytecodeExecutor {
    pub fn new() -> Self {
        Self {
            max_instructions: 100_000,
        }
    }

    pub fn with_max_instructions(mut self, max: u64) -> Self {
        self.max_instructions = max;
        self
    }

    fn compile_to_bytecode(&self, source: &str, script_id: Uuid) -> Result<(CompiledScript, BytecodeProgram)> {
        let mut lexer = LSLLexer::new(source.to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = LSLParser::new(tokens);
        let ast = parser.parse()?;

        let (globals, functions, states) = super::tree_walk::TreeWalkExecutor::extract_script_parts(&ast)?;

        let compiled = CompiledScript {
            script_id,
            globals,
            functions,
            states,
            ast: ast.clone(),
        };

        let mut bc_compiler = BytecodeCompiler::new();
        let program = bc_compiler.compile_program(&ast)?;

        Ok((compiled, program))
    }
}

impl ScriptExecutor for BytecodeExecutor {
    fn name(&self) -> &'static str {
        "Bytecode"
    }

    fn compile(&self, source: &str, script_id: Uuid) -> Result<CompiledScript> {
        let (compiled, _program) = self.compile_to_bytecode(source, script_id)?;
        Ok(compiled)
    }

    fn execute_event(
        &self,
        instance: &mut ScriptInstance,
        event: &str,
        args: &[LSLValue],
    ) -> Result<ExecutionResult> {
        let mut bc_compiler = BytecodeCompiler::new();
        let program = bc_compiler.compile_program(&instance.compiled.ast)?;

        let event_chunk = match program.find_event(&instance.current_state, event) {
            Some(ev) => ev,
            None => return Ok(ExecutionResult::Complete(LSLValue::Integer(0))),
        };

        let mut vm = Vm::new().with_max_instructions(self.max_instructions);

        let mut globals = instance.global_vars.clone();
        vm.set_globals(globals);

        vm.run_initializers(&program.global_initializers)?;

        let result = vm.execute(&event_chunk.chunk, args, &program)?;

        let final_globals = vm.take_globals();
        for (key, val) in final_globals {
            instance.global_vars.insert(key, val);
        }

        match result {
            VmResult::Complete(v) => Ok(ExecutionResult::Complete(v)),
            VmResult::StateChange(name) => Ok(ExecutionResult::StateChange(name)),
            VmResult::Yield => Ok(ExecutionResult::Yield),
            VmResult::Halt => Ok(ExecutionResult::Complete(LSLValue::Integer(0))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ScriptInstance;

    #[test]
    fn test_bytecode_compile_simple() -> Result<()> {
        let executor = BytecodeExecutor::new();
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
        Ok(())
    }

    #[test]
    fn test_bytecode_execute_event() -> Result<()> {
        let executor = BytecodeExecutor::new();
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
    fn test_bytecode_function_call() -> Result<()> {
        let executor = BytecodeExecutor::new();
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
    fn test_bytecode_while_loop() -> Result<()> {
        let executor = BytecodeExecutor::new();
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
    fn test_bytecode_vector() -> Result<()> {
        let executor = BytecodeExecutor::new();
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
