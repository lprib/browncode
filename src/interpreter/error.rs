use std::fmt::{Display, Formatter};
use std::string::ToString;

#[derive(Debug)]
pub enum Error {
    Graphics(String),
    System(String),
    LabelNotFound(String),
    FunctionNotFound(String),
    NameIsNotFunction(String),
    LabelRedefinition(String),
    U32OutOfBounds {
        u32_read_index: usize,
        memory_length: usize,
    },
    U8OutOfBounds {
        u8_read_index: usize,
        memory_length: usize,
    },
    IntrinsicArgumentMismatch {
        expected: &'static [usize],
        got: usize,
        func_name: String,
    },
    ArgumentMismatch {
        expected: usize,
        got: usize,
        func_name: String,
    },
    InvalidCharacterValue(u32),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Graphics(s) => write!(f, "graphics error {}", s),
            System(s) => write!(f, "system error {}", s),
            LabelNotFound(s) => write!(f, "could not find label `{}`", s),
            FunctionNotFound(s) => write!(f, "could not find function {}", s),
            NameIsNotFunction(s) => write!(
                f,
                "tried to call `{}` as a function, but it is not a function",
                s
            ),
            LabelRedefinition(s) => write!(f, "label `{}` is defined twice", s),
            U32OutOfBounds {
                u32_read_index,
                memory_length,
            } => write!(
                f,
                "attempted to read 32 bit number at index {}, but memory is only {} bytes long",
                u32_read_index, memory_length
            ),
            U8OutOfBounds {
                u8_read_index,
                memory_length,
            } => write!(
                f,
                "attempted to read 8 bit number at index {}, but memory is only {} bytes long",
                u8_read_index, memory_length
            ),
            IntrinsicArgumentMismatch {
                expected,
                got,
                func_name,
            } => write!(
                f,
                "function `{}` expects {} arguments but {} were supplied",
                func_name,
                expected
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(" or "),
                got
            ),
            ArgumentMismatch {
                expected, got, func_name
            } => write!(f, "function `{}` expects {} arguments but {} were supplied", func_name, expected, got),
            InvalidCharacterValue(c) => write!(f, "invalid character value 0x{:X}", c)
        }
    }
}
