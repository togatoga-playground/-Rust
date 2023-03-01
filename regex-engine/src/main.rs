use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

use helper::DynError;

mod engine;
mod helper;

fn match_file(expr: &str, file: &str) -> Result<(), DynError> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);
    engine::print(expr)?;
    println!();
    for line in reader.lines() {
        let line = line?;
        for (i, _) in line.char_indices() {
            if engine::do_matching(expr, &line[i..], true)? {
                println!("{line}");
                break;
            }
        }
    }
    Ok(())
}
fn main() -> Result<(), DynError> {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 2 {
        eprintln!("usage: {} regex file", args[0]);
        return Err("invalid arguments".into());
    } else {
        match_file(&args[1], &args[2])?;
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use crate::{helper::{SafeAdd, safe_add}, engine::do_matching};

    #[test]
    fn test_safe_add() {
        let n: usize = 10;
        assert_eq!(Some(30), n.safe_add(&20));
        let n: usize = std::usize::MAX;
        assert_eq!(None, n.safe_add(&1));

        let mut n: usize = 10;
        assert!(safe_add(&mut n, &20, || ()).is_ok());

        let mut n: usize = std::usize::MAX;
        assert!(safe_add(&mut n, &1, || ()).is_err());
    }

    #[test]
    fn test_matching() {
        // Error
        assert!(do_matching("+b", "bbb", true).is_err());
        assert!(do_matching("*b", "bbb", true).is_err());
        assert!(do_matching("|b", "bbb", true).is_err());
        assert!(do_matching("?b", "bbb", true).is_err());
        
        // OK
        assert!(do_matching("abc|def", "def", true).unwrap());
        assert!(do_matching("(abc)*", "abcab", true).unwrap());
        assert!(do_matching("(ab|cd)+", "abcdabcd", true).unwrap());
        assert!(do_matching("abc?", "ab", true).unwrap());
    }
}