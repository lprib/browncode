//! Converts ast::Block into intermediate_repr::IntermediateBlock
//! This flattens all control structures (for, if, while) into goto and jump instructions.
//! Expression trees are kept in their original parsed state.

use crate::ast::{AssignTarget, Block, DataBlock, DataDef, Expr, Line};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

pub type IntermediateBlock<'a> = Vec<IntermediateLine<'a>>;
pub type IntermediateBlockSlice<'a> = [IntermediateLine<'a>];

/// Vec of data paired with a map from labels to their index in the data
pub type DataSegment<'a> = (Vec<u8>, HashMap<&'a str, usize>);

/// Intermediate AST line
pub enum IntermediateLine<'a> {
    Assign(AssignTarget<'a>, Expr<'a>),
    Goto(Cow<'a, str>),
    Label(Cow<'a, str>),
    //TODO is this needed?
    JumpFalse(Expr<'a>, Cow<'a, str>),
    FunDeclaration(Cow<'a, str>, Vec<&'a str>),
    FunReturn,
    Expr(Expr<'a>),
}

/// Converts parsed AST lines to intermediate representaiton lines
pub fn to_intermediate_repr(ast: Block<'_>) -> IntermediateBlock<'_> {
    // label counter is used for generating internal labels
    // (counts up so each label is uniquely named)
    let mut label_counter = 0u32;
    convert_block(ast, &mut label_counter)
}

fn convert_block<'a>(block: Block<'a>, counter: &mut u32) -> IntermediateBlock<'a> {
    block
        .into_iter()
        .flat_map(|line| convert_line(line, counter))
        .collect()
}

fn convert_line<'a>(line: Line<'a>, counter: &mut u32) -> IntermediateBlock<'a> {
    let mut block = Vec::new();
    match line {
        Line::Assign(t, e) => block.push(IntermediateLine::Assign(t, e)),
        Line::Goto(l) => block.push(IntermediateLine::Goto(Cow::from(l))),
        Line::Label(l) => block.push(IntermediateLine::Label(Cow::from(l))),

        Line::If(test_expr, then_block, else_block) => {
            let else_label = next_label_name(counter);
            // If condition is false, skip the if block body and jump to else
            block.push(IntermediateLine::JumpFalse(test_expr, else_label.clone()));
            block.extend(convert_block(then_block, counter));
            if let Some(else_block) = else_block {
                //there is an else block
                let exit_label = next_label_name(counter);
                // if falling through from the if block, jump over the else block to the exit
                block.push(IntermediateLine::Goto(exit_label.clone()));
                block.push(IntermediateLine::Label(else_label));
                block.extend(convert_block(else_block, counter));
                block.push(IntermediateLine::Label(exit_label));
            } else {
                //no else block
                block.push(IntermediateLine::Label(else_label));
            }
        }

        Line::For(counter_variable, start, end, body) => {
            let start_label = next_label_name(counter);
            let exit_label = next_label_name(counter);
            //TODO macro for multiple pushes (does it exist?)
            block.extend(vec![
                IntermediateLine::Assign(AssignTarget::Var(counter_variable), start),
                IntermediateLine::Label(start_label.clone()),
                // if outside the bounds of the for loop (counter_variable >= end),
                // jump out of loop
                IntermediateLine::JumpFalse(
                    Expr::Lt(Box::new(Expr::Var(counter_variable)), Box::new(end)),
                    exit_label.clone(),
                ),
            ]);
            block.extend(convert_block(body, counter));
            block.extend(vec![
                // counter_variable = counter_variable + 1
                IntermediateLine::Assign(
                    AssignTarget::Var(counter_variable),
                    Expr::Add(
                        Box::new(Expr::Var(counter_variable)),
                        Box::new(Expr::Literal(1)),
                    ),
                ),
                IntermediateLine::Goto(start_label),
                IntermediateLine::Label(exit_label),
            ])
        }

        Line::While(condition, body) => {
            let start_label = next_label_name(counter);
            let exit_label = next_label_name(counter);
            block.push(IntermediateLine::Label(start_label.clone()));
            // if while condition is false, jump out of loop
            block.push(IntermediateLine::JumpFalse(condition, exit_label.clone()));
            block.extend(convert_block(body, counter));
            block.push(IntermediateLine::Goto(start_label));
            block.push(IntermediateLine::Label(exit_label));
        }

        Line::FunDeclaration(name, args, body) => {
            block.push(IntermediateLine::FunDeclaration(Cow::from(name), args));
            block.extend(convert_block(body, counter));
            block.push(IntermediateLine::FunReturn);
        }

        Line::Expr(e) => block.push(IntermediateLine::Expr(e)),
    }
    block
}

/// Generates and returns the name of a new internal label
/// (increments counter to create the name)
fn next_label_name<'a, 'b>(counter: &'a mut u32) -> Cow<'b, str> {
    //TODO maybe pass in description for debug puposes (eg. for_start / for_exit)
    
    //uses `$` because this char is not available in user label names, to avoid collision
    let label_name = format!("$internal_{}", counter);
    *counter += 1;
    Cow::from(label_name)
}

impl fmt::Debug for IntermediateLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IntermediateLine::Assign(t, e) => write!(f, "{:?} -> {:?}", e, t),
            IntermediateLine::Goto(l) => write!(f, "goto {}", l),
            IntermediateLine::Label(l) => write!(f, "{}:", l),
            IntermediateLine::JumpFalse(e, l) => write!(f, "if not {:?}: goto {}", e, l),
            IntermediateLine::FunDeclaration(n, a) => write!(f, "func {}{:?}", n, a),
            IntermediateLine::FunReturn => write!(f, "return"),
            IntermediateLine::Expr(e) => write!(f, "{:?}", e),
        }
    }
}

/// Flatten data block so all of the data is in a byte vec.
/// Labels are in a hashmap mapping them to the index of data they're pointing to
pub fn convert_data_segment(data: DataBlock<'_>) -> DataSegment<'_> {
    let mut map = HashMap::new();
    let mut data_vec = Vec::new();

    for data_def in data {
        match data_def {
            DataDef::Label(name) => {
                // data_vec.len() will point to the next data byte when it is appended
                map.insert(name, data_vec.len());
            }
            DataDef::Bytes(vec) => {
                data_vec.extend(vec);
            }
        }
    }

    (data_vec, map)
}
