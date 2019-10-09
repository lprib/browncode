use crate::ast::Expr;
use crate::intermediate_repr::{IntermediateBlock, IntermediateBlockSlice, IntermediateLine};

use std::borrow::Cow;
use std::collections::HashMap;

pub struct Interpreter<'a> {
    program: IntermediateBlock<'a>,
    label_table: HashMap<Cow<'a, str>, usize>,
}

impl<'a> Interpreter<'a> {
    pub fn new(program: IntermediateBlock<'a>) -> Self {
        let label_table = build_label_table(&program);

        Interpreter {
            program,
            label_table,
        }
    }

    pub fn execute(&mut self) {
        if let IntermediateLine::Assign(_, ex) = &self.program[0] {
            println!("{}", self.evaluate_expr(ex));
        }
    }

    fn evaluate_expr(&self, expr: &Expr<'_>) -> u32 {
        match expr {
            Expr::Literal(n) => *n,
            Expr::Add(l, r) => self.bin_op(&l, &r, |a, b| a + b),
            Expr::Sub(l, r) => self.bin_op(&l, &r, |a, b| a - b),
            Expr::Mul(l, r) => self.bin_op(&l, &r, |a, b| a * b),
            Expr::Div(l, r) => self.bin_op(&l, &r, |a, b| a / b),
            
            Expr::Lt(l, r) => self.bin_bool_op(&l, &r, |a, b| a < b),
            Expr::Gt(l, r) => self.bin_bool_op(&l, &r, |a, b| a > b),
            Expr::Leq(l, r) => self.bin_bool_op(&l, &r, |a, b| a <= b),
            Expr::Geq(l, r) => self.bin_bool_op(&l, &r, |a, b| a >= b),
            Expr::Neq(l, r) => self.bin_bool_op(&l, &r, |a, b| a != b),
            Expr::Eq(l, r) => self.bin_bool_op(&l, &r, |a, b| a == b),
            _ => unimplemented!(),
        }
    }

    fn bin_op<F>(&self, l: &Expr<'_>, r: &Expr<'_>, op: F) -> u32
    where
        F: Fn(u32, u32) -> u32,
    {
        let l = self.evaluate_expr(l);
        let r = self.evaluate_expr(r);
        op(l, r)
    }

    fn bin_bool_op<F>(&self, l: &Expr<'_>, r: &Expr<'_>, op: F) -> u32
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

fn build_label_table<'a>(program: &IntermediateBlockSlice<'a>) -> HashMap<Cow<'a, str>, usize> {
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
