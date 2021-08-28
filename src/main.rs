extern crate pest;
#[macro_use]
extern crate pest_derive;

mod compiler;
mod template;

mod executable;

mod bit_utils;

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
        println!("{}", e.as_pest_error());
        panic!()
    }

    match format {
        ExportFormat::Run => {
            match cythan_compiler::compile(&compile(&state.instructions, state.base)) {
                Ok(e) => {
                    let mut machine = cythan::InterruptedCythan::new(
                        e,
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
                            break;
                        }
                    }
                }
                Err(e) => {
                    println!("This error originated from the CythanV3 compiler and should be reported on https://github.com/Cythan-Project/cythan-high-level-compiler");
                    println!("You should include your source code and the following error in the report.");
                    println!("{}", e);
                    panic!()
                }
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
            std::fs::write(out, compile(&state.instructions, state.base)).unwrap();
        }
        ExportFormat::Cythan => {
            match cythan_compiler::compile(&compile(&state.instructions, state.base)) {
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
                    println!("This error originated from the CythanV3 compiler and should be reported on https://github.com/Cythan-Project/cythan-high-level-compiler");
                    println!("You should include your source code and the following error in the report.");
                    println!("{}", e);
                    panic!()
                }
            }
        }
        ExportFormat::Binary => {
            match cythan_compiler::compile(&compile(&state.instructions, state.base)) {
                Ok(e) => {
                    std::fs::write(
                        out,
                        encode(&CythanCode {
                            code: e,
                            base: 4,
                            start_pos: 35,
                        }),
                    )
                    .unwrap();
                }
                Err(e) => {
                    println!("This error originated from the CythanV3 compiler and should be reported on https://github.com/Cythan-Project/cythan-high-level-compiler");
                    println!("You should include your source code and the following error in the report.");
                    println!("{}", e);
                    panic!()
                }
            };
        }
    }

    // ...
}

fn compile(instructions: &[CompilableInstruction], base: u8) -> String {
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
