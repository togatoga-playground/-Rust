use std::fmt::{self, Display};

use crate::helper::DynError;

use self::evaluator::eval;

mod codegen;
mod evaluator;
mod parser;

#[derive(Debug)]
pub enum Instruction {
    Char(char),
    Match,
    Jump(usize),
    Split(usize, usize),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Char(c) => write!(f, "char {}", c),
            Instruction::Match => write!(f, "match"),
            Instruction::Jump(addr) => write!(f, "jump {:>04}", addr),
            Instruction::Split(addr1, addr2) => {
                write!(f, "split {:>04}, {:>04}", addr1, addr2)
            }
        }
    }
}

pub fn print(expr: &str) -> Result<(), DynError> {
    println!("expr: {expr}");
    let ast = parser::parse(expr)?;
    println!("AST: {:?}", ast);
    println!();
    println!("code:");
    let code = codegen::gen_code(&ast)?;
    for (n, c) in code.iter().enumerate() {
        println!("{:>04}: {c}", n);
    }

    Ok(())
}

///
/// ```
/// use regex_engine;
/// regex_engine::do_matching("abc|(de|cd)+", "decddede", true);
/// ```
pub fn do_matching(expr: &str, line: &str, is_depth: bool) -> Result<bool, DynError> {
    let ast = parser::parse(expr)?;
    let code = codegen::gen_code(&ast)?;
    let line = line.chars().collect::<Vec<char>>();
    Ok(eval(&code, &line, is_depth)?)
}
