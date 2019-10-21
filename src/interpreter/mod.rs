//! The interpreter that runs IntermediateLine IR

use self::error::Error;
use self::state::InterpreterState;
use crate::graphics::{Graphics, Sprites};
use crate::intermediate_repr::{IntermediateBlock, IntermediateBlockSlice, IntermediateLine};

use std::borrow::Cow;
use std::collections::HashMap;

mod error;
mod intrinsics;
mod state;

type InterpreterResult<T> = Result<T, Error>;

/// The immutable program data that is run by the interpreter
pub struct Program<'a> {
    pub ir: IntermediateBlock<'a>,
    pub data: Vec<u8>,
    /// Maps labels to the line that they point to in code
    pub label_table: HashMap<Cow<'a, str>, usize>,
    /// Maps data labels to the byte in data they point to
    pub data_label_table: HashMap<&'a str, usize>,
}

pub fn execute<'a>(program: &Program<'a>) -> InterpreterResult<()> {
    // TODO take ownership of program so clones are not needed?
    let graphics = Graphics::try_new().map_err(Error::Graphics)?;
    let sprite_creator = &graphics.get_sprite_creator();
    let sprites = Sprites::new(&sprite_creator);

    let mut state = InterpreterState {
        // copy user defined data into a mutable vec
        data: program.data.clone(),
        var_table: program.data_label_table.clone(),
        instr_index: 0,
        graphics,
        sprites,
    };

    while state.instr_index < program.ir.len() {
        state.execute_line(program)?;
    }
    Ok(())
}

/// Iterates over a program and returns the mapping from labels to the line index the label points to
pub fn build_label_table<'a>(
    program: &IntermediateBlockSlice<'a>,
) -> InterpreterResult<HashMap<Cow<'a, str>, usize>> {
    let mut map = HashMap::new();
    for (i, line) in program.iter().enumerate() {
        // fun declarations are essentially labels, so add them to the map as well
        // NOTE this means there can be name conflicts between fun names and label names
        if let IntermediateLine::Label(name) | IntermediateLine::FunDeclaration(name, _) = line {
            if map.contains_key(name) {
                // panic!("label {} is defined more than once", name);
                return Err(Error::LabelRedefinition(name.to_string()));
            }
            map.insert(name.clone(), i);
        }
    }
    Ok(map)
}