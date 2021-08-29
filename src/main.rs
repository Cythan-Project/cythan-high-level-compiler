extern crate pest;
#[macro_use]
extern crate pest_derive;

#[cfg(test)]
mod tests;

mod compiler;
mod template;

mod executable;

mod bit_utils;

use std::sync::{Arc, Mutex};

use crate::compiler::type_defs::Result;
use compiler::{
    asm::CompilableInstruction,
    error::{CError, CSpan},
    parser::{codeblock::CodeBlock, parse_file},
    scope::ScopedState,
    state::State,
};
use cythan::Cythan;
use executable::{encode, CythanCode};
use template::{get_int_pos_from_base, Template};

use crate::compiler::asm;

pub enum ExportFormat {
    Run,
    ByteCode,
    CythanV3,
    Cythan,
    Binary,
}

fn main() {
    let format = ExportFormat::Run;
    let out = "out.ct";

    let mut state = State::default();
    let mut scope = ScopedState::new();

    if let Err(e) = execute_file("test1.ct1", &mut state, &mut scope, None) {
        println!("{}", e.display());
        panic!()
    }

    match format {
        ExportFormat::Run => {
            if let Err(e) = compile_and_run_stdio(&state.instructions, state.base, &state) {
                println!("{}", e.display());
                panic!()
            }
        }
        ExportFormat::ByteCode => {
            std::fs::write(
                out,
                state
                    .instructions
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
            .unwrap();
        }
        ExportFormat::CythanV3 => {
            std::fs::write(out, compile_v3(&state.instructions, state.base)).unwrap();
        }
        ExportFormat::Cythan => match compile(&state.instructions, state.base, &state) {
            Ok(e) => {
                std::fs::write(
                    out,
                    e.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
                .unwrap();
            }
            Err(e) => {
                println!("{}", e.display());
                panic!()
            }
        },
        ExportFormat::Binary => {
            match compile_binary(&state.instructions, state.base, &state) {
                Ok(e) => {
                    std::fs::write(out, e).unwrap();
                }
                Err(e) => {
                    println!("{}", e.display());
                    panic!()
                }
            };
        }
    }

    // ...
}

pub fn compile_and_run_stdio(
    instructions: &[CompilableInstruction],
    base: u8,
    state: &State,
) -> Result<()> {
    let mut machine = cythan::InterruptedCythan::new_stdio(
        compile(instructions, base, state)?,
        state.base,
        get_int_pos_from_base(state.base),
    );

    loop {
        for _ in 0..1000 {
            machine.next();
        }

        let o = machine.cases.clone();

        machine.next();

        if o == machine.cases {
            break Ok(());
        }
    }
}

pub fn compile_and_run(
    instructions: &[CompilableInstruction],
    base: u8,
    state: &State,
    inputs: Vec<char>,
) -> Result<String> {
    let string = Arc::new(Mutex::new(String::new()));
    let string1 = string.clone();
    let k = Arc::new(Mutex::new(inputs.into_iter()));
    let mut machine = cythan::InterruptedCythan::new(
        compile(instructions, base, state)?,
        state.base,
        get_int_pos_from_base(state.base),
        move |a| {
            string.lock().unwrap().push(a as char);
        },
        move || k.lock().unwrap().next().unwrap() as u8,
    );

    loop {
        for _ in 0..1000 {
            machine.next();
        }

        let o = machine.cases.clone();

        machine.next();

        if o == machine.cases {
            break Ok(string1.lock().unwrap().clone());
        }
    }
}

pub fn compile_binary(
    instructions: &[CompilableInstruction],
    base: u8,
    state: &State,
) -> Result<Vec<u8>> {
    Ok(encode(&CythanCode {
        code: compile(instructions, base, state)?,
        base: 4,
        start_pos: 35,
    }))
}

pub fn compile(
    instructions: &[CompilableInstruction],
    base: u8,
    state: &State,
) -> Result<Vec<usize>> {
    cythan_compiler::compile(&compile_v3(&state.instructions, state.base))
        .map_err(|e| e.to_string())
        .map_err(CError::InternalCompilerError)
}

fn compile_v3(instructions: &[CompilableInstruction], base: u8) -> String {
    let mut template = Template::new(include_str!("../template.ct"), base);
    let mut ctx = asm::Context::default();
    instructions
        .iter()
        .for_each(|x| x.compile(&mut template, &mut ctx));
    template.build()
}

pub fn execute_file(
    file_name: &str,
    state: &mut State,
    scope: &mut ScopedState,
    span: Option<CSpan>,
) -> Result<()> {
    CodeBlock(parse_file(
        file_name,
        match std::fs::read_to_string(file_name) {
            Ok(a) => a,
            Err(e) => {
                if let Some(span) = span {
                    return Err(CError::FileNotFound(span, file_name.to_owned()));
                } else {
                    panic!("{}", e);
                }
            }
        },
        span,
    )?)
    .execute_with_scope(state, scope)
    .map(|_| ())
}
