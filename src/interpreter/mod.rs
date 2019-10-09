use crate::ast::Expr;
use crate::intermediate_repr::{IntermediateBlock, IntermediateBlockSlice, IntermediateLine};

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
}

pub fn execute<'a>(program: &Program<'a>) {
    let mut state = InterpreterState {
        data: program.data.clone(),
        var_table: program.data_label_table.clone(),
        instr_index: 0,
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
            Assign(name, expr) => {
                let n = self.evaluate_expr(expr, program);
                let addr = get_var_address(name, &mut self.var_table, &mut self.data);
                set_u32(addr, &mut self.data, n);
            }

            Expr(expr) => {
                self.evaluate_expr(expr, program);
            }

            JumpFalse(expr, label) => {
                let n = self.evaluate_expr(expr, program);
                if n == 0 {
                    self.instr_index = program.label_table[label];
                }
            }

            Label(..) | FunDeclaration(..) | FunReturn => {}

            Goto(name) => {
                self.instr_index = *program
                    .label_table
                    .get(name)
                    .unwrap_or_else(|| panic!("no label {}", name));
            }
        };
        self.instr_index += 1;
    }

    fn evaluate_expr(&mut self, expr: &Expr<'a>, program: &Program<'a>) -> u32 {
        use Expr::*;
        // println!("{:?}", expr);
        match expr {
            Literal(n) => *n,
            Add(l, r) => self.bin_op(&l, &r, program, |a, b| a + b),
            Sub(l, r) => self.bin_op(&l, &r, program, |a, b| a - b),
            Mul(l, r) => self.bin_op(&l, &r, program, |a, b| a * b),
            Div(l, r) => self.bin_op(&l, &r, program, |a, b| a / b),
            Mod(l, r) => self.bin_op(&l, &r, program, |a, b| a % b),
            Lt(l, r) => self.bin_bool_op(&l, &r, program, |a, b| a < b),
            Gt(l, r) => self.bin_bool_op(&l, &r, program, |a, b| a > b),
            Leq(l, r) => self.bin_bool_op(&l, &r, program, |a, b| a <= b),
            Geq(l, r) => self.bin_bool_op(&l, &r, program, |a, b| a >= b),
            Neq(l, r) => self.bin_bool_op(&l, &r, program, |a, b| a != b),
            Eq(l, r) => self.bin_bool_op(&l, &r, program, |a, b| a == b),
            FunCall(name, args) => {
                let args = args
                    .iter()
                    .map(|e| self.evaluate_expr(e, program))
                    .collect::<Vec<u32>>();
                if let Some(f) = intrinsics::get_intrinsic(name) {
                    f(args, self)
                } else {
                    self.evaluate_funcall(name, &args, program)
                }
            }
            Var(name) => get_var(name, &mut self.var_table, &mut self.data),
            Deref(e) => get_u32(self.evaluate_expr(e, program) as usize, &self.data),
            VarAddress(name) => get_var_address(name, &mut self.var_table, &mut self.data) as u32,
        }
    }

    fn bin_op<F>(&mut self, left: &Expr<'a>, right: &Expr<'a>, program: &Program<'a>, op: F) -> u32
    where
        F: Fn(u32, u32) -> u32,
    {
        let l = self.evaluate_expr(left, program);
        let r = self.evaluate_expr(right, program);
        op(l, r)
    }

    fn bin_bool_op<F>(
        &mut self,
        left: &Expr<'a>,
        right: &Expr<'a>,
        program: &Program<'a>,
        op: F,
    ) -> u32
    where
        F: Fn(u32, u32) -> bool,
    {
        let l = self.evaluate_expr(left, program);
        let r = self.evaluate_expr(right, program);

        if op(l, r) {
            1
        } else {
            0
        }
    }

    fn evaluate_funcall(&mut self, name: &str, args: &[u32], program: &Program<'a>) -> u32 {
        let return_isp = self.instr_index;
        //jump to function
        self.instr_index = program.label_table[name];

        if let IntermediateLine::FunDeclaration(_, params) = &program.code[self.instr_index] {
            for (i, param) in params.iter().enumerate() {
                let addr = get_var_address(param, &mut self.var_table, &mut self.data);
                if let Some(arg) = args.get(i) {
                    set_u32(addr, &mut self.data, *arg);
                } else {
                    //if not enough arguments are passed to the function, fill them with 0
                    set_u32(addr, &mut self.data, 0);
                }
            }

            loop {
                if let IntermediateLine::FunReturn = program.code[self.instr_index] {
                    break;
                }
                self.execute_line(program);
            }
            self.instr_index = return_isp;
            get_var("ans", &mut self.var_table, &mut self.data)
        } else {
            panic!("{} is not a valid function", name);
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

fn get_var<'a>(name: &'a str, var_table: &mut HashMap<&'a str, usize>, data: &mut Vec<u8>) -> u32 {
    let addr = get_var_address(name, var_table, data);
    get_u32(addr, data)
}

//todo better (unsafe) way?
fn get_u32(index: usize, vec: &[u8]) -> u32 {
    u32::from(vec[index]) << 24
        | u32::from(vec[index + 1]) << 16
        | u32::from(vec[index + 2]) << 8
        | u32::from(vec[index + 3])
}

fn set_u32(index: usize, vec: &mut Vec<u8>, value: u32) {
    vec[index] = (value >> 24 & 0xFF) as u8;
    vec[index + 1] = (value >> 16 & 0xFF) as u8;
    vec[index + 2] = (value >> 8 & 0xFF) as u8;
    vec[index + 3] = (value & 0xFF) as u8;
}
