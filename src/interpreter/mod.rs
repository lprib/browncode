use crate::ast::Expr;
use crate::intermediate_repr::{
    DataSegment, IntermediateBlock, IntermediateBlockSlice, IntermediateLine,
};

use std::borrow::Cow;
use std::collections::HashMap;

mod intrinsics;

pub struct Program<'a> {
    pub code: IntermediateBlock<'a>,
    pub data: Vec<u8>,
    pub label_table: HashMap<Cow<'a, str>, usize>,
    pub data_label_table: HashMap<&'a str, usize>,
}

pub struct InterpreterState<'a> {
    data: Vec<u8>,
    var_table: HashMap<&'a str, usize>,
    instr_index: usize,
    return_stack: Vec<usize>,
}

pub fn execute<'a>(program: &Program<'a>) {
    let mut state = InterpreterState {
        data: program.data.clone(),
        var_table: HashMap::new(),
        instr_index: 0,
        return_stack: Vec::new(),
    };

    loop {
        if state.instr_index >= program.code.len() {
            break;
        }
        state.execute_line(program);
    }
}

impl<'a> InterpreterState<'a> {
    fn execute_line(&mut self, program: &Program<'a>) {
        let line = &program.code[self.instr_index];
        // println!("{:?}", line);
        use IntermediateLine::*;
        match line {
            Assign(ref name, ref expr) => {
                let n = self.evaluate_expr(expr);
                let addr = get_var_address(name, &mut self.var_table, &mut self.data);
                set_u32(addr, &mut self.data, n);
            }

            Expr(e) => {
                self.evaluate_expr(e);
            }

            JumpFalse(expr, label) => {
                let n = self.evaluate_expr(expr);
                if n == 0 {
                    self.instr_index = program.label_table[label];
                }
            }

            Label(_) => {}

            Goto(name) => {
                self.instr_index = program.label_table[name];
            }

            _ => unimplemented!(),
        };
        self.instr_index += 1;
    }

    fn evaluate_expr(&mut self, expr: &Expr<'a>) -> u32 {
        use Expr::*;
        // println!("{:?}", expr);
        match expr {
            Literal(n) => *n,
            Add(l, r) => self.bin_op(&l, &r, |a, b| a + b),
            Sub(l, r) => self.bin_op(&l, &r, |a, b| a - b),
            Mul(l, r) => self.bin_op(&l, &r, |a, b| a * b),
            Div(l, r) => self.bin_op(&l, &r, |a, b| a / b),
            Lt(l, r) => self.bin_bool_op(&l, &r, |a, b| a < b),
            Gt(l, r) => self.bin_bool_op(&l, &r, |a, b| a > b),
            Leq(l, r) => self.bin_bool_op(&l, &r, |a, b| a <= b),
            Geq(l, r) => self.bin_bool_op(&l, &r, |a, b| a >= b),
            Neq(l, r) => self.bin_bool_op(&l, &r, |a, b| a != b),
            Eq(l, r) => self.bin_bool_op(&l, &r, |a, b| a == b),
            FunCall(name, args) => {
                let args = args
                    .iter()
                    .map(|e| self.evaluate_expr(e))
                    .collect::<Vec<u32>>();
                if let Some(f) = intrinsics::get_intrinsic(name) {
                    f(args)
                } else {
                    //TODO
                    0
                }
            },
            Var(name) => get_var(name, &mut self.var_table, &mut self.data),
            _ => unimplemented!(),
        }
    }

    fn bin_op<F>(&mut self, l: &Expr<'a>, r: &Expr<'a>, op: F) -> u32
    where
        F: Fn(u32, u32) -> u32,
    {
        let l = self.evaluate_expr(l);
        let r = self.evaluate_expr(r);
        op(l, r)
    }

    fn bin_bool_op<F>(&mut self, l: &Expr<'a>, r: &Expr<'a>, op: F) -> u32
    where
        F: Fn(u32, u32) -> bool,
    {
        let l = self.evaluate_expr(l);
        let r = self.evaluate_expr(r);

        if op(l, r) {
            1
        } else {
            0
        }
    }
}

pub fn build_label_table<'a>(program: &IntermediateBlockSlice<'a>) -> HashMap<Cow<'a, str>, usize> {
    let mut map = HashMap::new();
    for (i, line) in program.iter().enumerate() {
        // fun declarations are essentially labels, so add them to the map as well
        // NOTE this means there can be name conflicts between fun names and label names
        if let IntermediateLine::Label(name) | IntermediateLine::FunDeclaration(name, _) = line {
            //TODO check if name already exists in table (conflict)
            map.insert(name.clone(), i);
        }
    }
    map
}

fn get_var_address<'a>(
    name: &'a str,
    var_table: &mut HashMap<&'a str, usize>,
    data: &mut Vec<u8>,
) -> usize {
    if var_table.contains_key(name) {
        var_table[name]
    } else {
        let addr = data.len();
        var_table.insert(name, addr);
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);
        addr
    }
}

fn get_var<'a> (name: &'a str, var_table: &mut HashMap<&'a str, usize>, data: &mut Vec<u8>) -> u32 {
    let addr = get_var_address(name, var_table, data);
    get_u32(addr, data)
}

//todo better (unsafe) way?
fn get_u32(index: usize, vec: &Vec<u8>) -> u32 {
    (vec[index] as u32) << 24
        | (vec[index + 1] as u32) << 16
        | (vec[index + 2] as u32) << 8
        | (vec[index + 3] as u32)
}

fn set_u32(index: usize, vec: &mut Vec<u8>, value: u32) {
    vec[index + 0] = (value >> 24 & 0xFF) as u8;
    vec[index + 1] = (value >> 16 & 0xFF) as u8;
    vec[index + 2] = (value >> 8 & 0xFF) as u8;
    vec[index + 3] = (value >> 0 & 0xFF) as u8;
}
