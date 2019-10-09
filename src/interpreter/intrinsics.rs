use super::InterpreterState;
use lazy_static::lazy_static;

type IntrinsicFn = fn(Vec<u32>, &mut InterpreterState) -> u32;

lazy_static! {
    static ref INTRINSICS: &'static [(&'static str, IntrinsicFn)] =
        &[("println", println), ("print", print), ("puts", puts)];
}

pub fn get_intrinsic(name: &str) -> Option<&IntrinsicFn> {
    for (ref f_name, f) in INTRINSICS.iter() {
        if &name == f_name {
            return Some(f);
        }
    }
    None
}

fn println(args: Vec<u32>, _: &mut InterpreterState) -> u32 {
    if args.is_empty() {
        println!();
        return 0;
    }

    for arg in args {
        println!("{}", arg);
    }
    0
}

fn print(args: Vec<u32>, _: &mut InterpreterState) -> u32 {
    for arg in args {
        print!("{}", arg);
    }
    0
}

fn puts(args: Vec<u32>, state: &mut InterpreterState) -> u32 {
    let mut i = args[0] as usize;
    while state.data[i] != 0 {
        print!("{}", state.data[i] as char);
        i += 1;
    }
    0
}
