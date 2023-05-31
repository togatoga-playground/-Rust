use std::{
    error::Error,
    fmt::{self, Display},
    mem::take,
};

#[derive(Debug)]
pub enum Ast {
    Char(char),
    Plus(Box<Ast>),
    Star(Box<Ast>),
    Question(Box<Ast>),
    Or(Box<Ast>, Box<Ast>),
    Seq(Vec<Ast>),
}

#[derive(Debug)]
pub enum ParseError {
    InvalidEscape(usize, char),
    InvalidRightParen(usize),
    NoPrev(usize),
    NoRightParen,
    Empty,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidEscape(pos, c) => {
                write!(f, "ParseError: invalid escape: pos = {pos}, char = '{c}'")
            }
            ParseError::InvalidRightParen(pos) => {
                write!(f, "ParseError: invalid right parenthesis: pos = {pos}")
            }
            ParseError::NoPrev(pos) => {
                write!(f, "ParseError: no previous expression: pos = {pos}")
            }
            ParseError::NoRightParen => {
                write!(f, "ParseError: no right parenthesis")
            }
            ParseError::Empty => write!(f, "ParseError: empty expression"),
        }
    }
}

impl Error for ParseError {}

fn parse_escape(pos: usize, c: char) -> Result<Ast, ParseError> {
    match c {
        '\\' | '(' | ')' | '|' | '+' | '*' | '?' => Ok(Ast::Char(c)),
        _ => Err(ParseError::InvalidEscape(pos, c)),
    }
}

/// `PSQ` is for `parse_plus_star_question`
enum Psq {
    Plus,
    Star,
    Question,
}

/// a+ a? (abc|def)*
fn parse_plus_star_question(
    seq: &mut Vec<Ast>,
    ast_type: Psq,
    pos: usize,
) -> Result<(), ParseError> {
    if let Some(prev) = seq.pop() {
        let ast = match ast_type {
            Psq::Plus => Ast::Plus(Box::new(prev)),
            Psq::Star => Ast::Star(Box::new(prev)),
            Psq::Question => Ast::Question(Box::new(prev)),
        };
        seq.push(ast);
        Ok(())
    } else {
        Err(ParseError::NoPrev(pos))
    }
}

/// abc|def|ghi AST::Or("abc", AST::Or("def", "ghi"))
fn fold_or(mut seq_or: Vec<Ast>) -> Option<Ast> {
    if seq_or.len() > 1 {
        let mut ast = seq_or.pop().unwrap();
        seq_or.reverse();
        for s in seq_or {
            ast = Ast::Or(Box::new(s), Box::new(ast));
        }
        Some(ast)
    } else {
        seq_or.pop()
    }
}
pub fn parse(expr: &str) -> Result<Ast, ParseError> {
    enum ParseState {
        Char,
        Escape,
    }
    let mut seq = Vec::new();
    let mut seq_or = Vec::new();
    let mut stack = Vec::new();
    let mut state = ParseState::Char;

    for (i, c) in expr.chars().enumerate() {
        match &state {
            ParseState::Char => {
                match c {
                    '+' => parse_plus_star_question(&mut seq, Psq::Plus, i)?,
                    '*' => parse_plus_star_question(&mut seq, Psq::Star, i)?,
                    '?' => parse_plus_star_question(&mut seq, Psq::Question, i)?,
                    '(' => {
                        let prev = take(&mut seq);
                        let prev_or = take(&mut seq_or);
                        stack.push((prev, prev_or));
                    }
                    ')' => {
                        if let Some((mut prev, prev_or)) = stack.pop() {
                            // ()
                            if !seq.is_empty() {
                                seq_or.push(Ast::Seq(seq));
                            }
                            if let Some(ast) = fold_or(seq_or) {
                                prev.push(ast);
                            }

                            seq = prev;
                            seq_or = prev_or;
                        } else {
                            // abc)
                            return Err(ParseError::InvalidRightParen(i));
                        }
                    }
                    '|' => {
                        if seq.is_empty() {
                            // "||", "(|abc)"
                            return Err(ParseError::NoPrev(i));
                        } else {
                            let prev = take(&mut seq);
                            seq_or.push(Ast::Seq(prev));
                        }
                    }
                    '\\' => state = ParseState::Escape,
                    _ => seq.push(Ast::Char(c)),
                }
            }
            ParseState::Escape => {
                let ast = parse_escape(i, c)?;
                seq.push(ast);
                state = ParseState::Char;
            }
        }
    }

    if !stack.is_empty() {
        return Err(ParseError::NoRightParen);
    }

    if !seq.is_empty() {
        seq_or.push(Ast::Seq(seq));
    }

    if let Some(ast) = fold_or(seq_or) {
        Ok(ast)
    } else {
        Err(ParseError::Empty)
    }
}
