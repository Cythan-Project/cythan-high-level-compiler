#![feature(option_result_unwrap_unchecked)]

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[cfg(test)]
mod tests;

mod compiler;
mod template;

mod executable;

mod bit_utils;

use std::{
    process::exit,
    sync::{Arc, Mutex},
};

use crate::compiler::{
    asm::opt_asm,
    mir::{optimizer, MirCodeBlock, MirState},
    type_defs::Result,
};
use compiler::{
    asm::CompilableInstruction,
    error::{CError, CErrorType, CSpan},
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

pub fn show_usage() {
    println!("Usages:");
    println!("   cyc run <INPUT FILENAME> [Optional: base, Default: 16]");
    println!("   cyc build <INPUT FILENAME> <OUTPUT FILENAME> <TYPE> [Optional: base, Default: 4]");
    println!("    TYPE: V3, Bytecode, Binary, Default");
}

fn parse() -> Option<(String, String, ExportFormat, u8)> {
    let mut args = std::env::args();
    args.next()?;

    match args.next()?.as_str() {
        "run" => Some((
            args.next()?,
            String::new(),
            ExportFormat::Run,
            args.next().map(|x| x.parse().unwrap()).unwrap_or(4),
        )),
        "build" => Some((
            args.next()?,
            args.next()?,
            match args.next()?.to_lowercase().as_str() {
                "cythan" | "default" | "cy" => ExportFormat::Cythan,
                "cythanv3" | "cythan-v3" | "v3" => ExportFormat::CythanV3,
                "bytecode" | "bc" => ExportFormat::ByteCode,
                "bin" | "binary" | "exe" | "executable" => ExportFormat::Binary,
                _ => return None,
            },
            args.next().map(|x| x.parse().unwrap()).unwrap_or(4),
        )),
        _ => None,
    }
}

fn main() {
    /* let format = ExportFormat::Run;
    let out = "out.ct"; */

    let (input, out, format, base) = if let Some(e) = parse() {
        e
    } else {
        show_usage();
        exit(-2);
    };

    let mut state = State::default();
    state.base = base;
    let mut scope = ScopedState::new();

    if let Err(e) = execute_file(&input, &mut state, &mut scope, vec![]) {
        println!("{}", e);
        exit(-1);
    }

    match format {
        ExportFormat::Run => {
            if let Err(e) = compile_and_run_stdio(&state) {
                println!("{}", e);
                panic!()
            }
        }
        ExportFormat::ByteCode => {
            let mut mrstate = MirState::default();

            MirCodeBlock(optimizer::opt(state.instructions.0.clone())).to_asm(&mut mrstate);

            std::fs::write(
                out,
                mrstate
                    .instructions
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
            .unwrap();
        }
        ExportFormat::CythanV3 => {
            let mut k = MirState::default();
            MirCodeBlock(optimizer::opt(state.instructions.0.clone())).to_asm(&mut k);
            std::fs::write(out, compile_v3(opt_asm(k.instructions), state.base)).unwrap();
        }
        ExportFormat::Cythan => match compile(&state) {
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
                println!("{}", e);
                panic!()
            }
        },
        ExportFormat::Binary => {
            match compile_binary(&state) {
                Ok(e) => {
                    std::fs::write(out, e).unwrap();
                }
                Err(e) => {
                    println!("{}", e);
                    exit(-3);
                }
            };
        }
    }

    // ...
}

pub fn compile_and_run_stdio(state: &State) -> Result<()> {
    let mut machine = cythan::InterruptedCythan::new_stdio(
        compile(state)?,
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

pub fn compile_and_run(state: &State, inputs: Vec<char>) -> Result<String> {
    let string = Arc::new(Mutex::new(String::new()));
    let string1 = string.clone();
    let k = Arc::new(Mutex::new(inputs.into_iter()));
    let mut machine = cythan::InterruptedCythan::new(
        compile(state)?,
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

pub fn compile_binary(state: &State) -> Result<Vec<u8>> {
    Ok(encode(&CythanCode {
        code: compile(state)?,
        base: 4,
        start_pos: 35,
    }))
}

pub fn compile(state: &State) -> Result<Vec<usize>> {
    let mut k = MirState::default();
    MirCodeBlock(optimizer::opt(state.instructions.0.clone())).to_asm(&mut k);
    cythan_compiler::compile(&compile_v3(opt_asm(k.instructions), state.base))
        .map_err(|e| e.to_string())
        .map_err(|e| CError(vec![], CErrorType::InternalCompilerError(e)))
}

fn compile_v3(instructions: Vec<CompilableInstruction>, base: u8) -> String {
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
    span: Vec<CSpan>,
) -> Result<()> {
    CodeBlock(parse_file(
        file_name,
        match std::fs::read_to_string(file_name) {
            Ok(a) => a,
            Err(_e) => {
                return Err(CError(span, CErrorType::FileNotFound(file_name.to_owned())));
            }
        },
        span,
    )?)
    .execute_with_scope(state, scope)
    .map(|_| ())
}
