//! The interpreter that runs IntermediateLine IR

use self::state::InterpreterState;
use crate::error::{Error, IResult};
use crate::graphics::{Graphics, Sprites};
use crate::intermediate_repr::{IntermediateBlock, IntermediateBlockSlice, IntermediateLine, DataSegment};

use std::borrow::Cow;
use std::collections::HashMap;

mod intrinsics;
mod state;

/// The immutable program data that is run by the interpreter
pub struct Program<'a> {
    ir: IntermediateBlock<'a>,
    data: Vec<u8>,
    /// Maps labels to the line that they point to in code
    label_table: HashMap<Cow<'a, str>, usize>,
    /// Maps data labels to the byte in data they point to
    data_label_table: HashMap<&'a str, usize>,
}

impl<'a> Program<'a> {
    pub fn try_new(ir: IntermediateBlock<'a>, data_segment: DataSegment<'a>) -> IResult<Self> {
        let label_table = build_label_table(&ir)?;
        Ok(Program {
            ir,
            data: data_segment.0,
            label_table,
            data_label_table: data_segment.1
        })
    }
}

pub fn execute<'a>(program: &Program<'a>) -> IResult<()> {
    // TODO take ownership of program so clones are not needed?
    let graphics = Graphics::try_new()?;
    let sprite_creator = &graphics.get_sprite_creator();
    let sprites = Sprites::new(&sprite_creator);

    let mut state = InterpreterState {
        // clone user defined data into a mutable vec
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
) -> IResult<HashMap<Cow<'a, str>, usize>> {
    let mut map = HashMap::new();
    for (i, line) in program.iter().enumerate() {
        // fun declarations are essentially labels, so add them to the map as well
        // NOTE this means there can be name conflicts between fun names and label names
        if let IntermediateLine::Label(name) | IntermediateLine::FunDeclaration(name, ..) = line {
            if map.contains_key(name) {
                // panic!("label {} is defined more than once", name);
                return Err(Error::LabelRedefinition(name.to_string()));
            }
            map.insert(name.clone(), i);
        }
    }
    Ok(map)
}
