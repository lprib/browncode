#[derive(Debug)]
pub enum Error {
    Graphics(String),
    System(String),
    LabelNotFound(String),
    FunctionNotFound(String),
    NameIsNotFunction(String),
    LabelRedefinition(String),
    U32ReadOutOfBounds {
        u32_read_index: usize,
        memory_length: usize,
    },
    U8ReadOutOfBounds {
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
