use lazy_static::lazy_static;

type IntrinsicFn = fn(Vec<u32>) -> u32;

lazy_static! {
    static ref INTRINSICS: &'static [(&'static str, IntrinsicFn)] = &[
        ("print", print)
    ];
}

pub fn get_intrinsic(name: &str) -> Option<&IntrinsicFn> {
    for (ref f_name, f) in INTRINSICS.iter() {
        if &name == f_name {
            return Some(f);
        }
    }
    None
}

fn print(args: Vec<u32>) -> u32 {
    println!("{}", args[0]);
    0
}