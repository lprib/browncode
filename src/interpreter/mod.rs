//! The interpreter that runs IntermediateLine IR

use crate::ast::{AssignTarget, Expr};
use crate::graphics::Graphics;
use crate::intermediate_repr::{IntermediateBlock, IntermediateBlockSlice, IntermediateLine};

use std::borrow::Cow;
use std::collections::HashMap;

mod intrinsics;

/// The immutable program data that is run by the interpreter
pub struct Program<'a> {
    pub code: IntermediateBlock<'a>,
    pub data: Vec<u8>,
    /// Maps labels to the line that they point to in code
    pub label_table: HashMap<Cow<'a, str>, usize>,
    /// Maps data labels to the byte in data they point to
    pub data_label_table: HashMap<&'a str, usize>,
}

/// Mutable state the interpreter keeps
pub struct InterpreterState<'a> {
    /// The interpreter's memory segment (contains variables and user defined data)
    data: Vec<u8>,
    /// Maps vars (as they are created) to their location (byte index) in the data vec
    var_table: HashMap<&'a str, usize>,
    /// current instruction pointer
    instr_index: usize,
    graphics: Graphics,
}

pub fn execute<'a>(program: &Program<'a>) {
    // TODO take ownership of program so clones are not needed?

    let mut state = InterpreterState {
        // copy user defined data into a mutable vec
        data: program.data.clone(),
        var_table: program.data_label_table.clone(),
        instr_index: 0,
        graphics: Graphics::try_new().unwrap(),
    };

    while state.instr_index < program.code.len() {
        state.execute_line(program);
    }
}

impl<'a> InterpreterState<'a> {
    /// Execute a single line based on the current state (instr pointer)
    ///
    /// Note, this may actually execute several lines because function calls
    /// in the current line will jump to the function definition and fully execute
    /// before returning to the current line.
    fn execute_line(&mut self, program: &Program<'a>) {
        let line = &program.code[self.instr_index];
        use IntermediateLine::*;
        match line {
            Assign(target, expr) => {
                let value_to_assign = self.evaluate_expr(expr, program);
                match target {
                    AssignTarget::Var(name) => {
                        let store_address =
                            get_var_address(name, &mut self.var_table, &mut self.data);
                        set_memory_u32(store_address, &mut self.data, value_to_assign);
                    }
                    AssignTarget::Addr(addr) => {
                        let store_address = self.evaluate_expr(addr, program);
                        set_memory_u32(store_address as usize, &mut self.data, value_to_assign);
                    }
                    AssignTarget::ByteAddr(addr) => {
                        let store_address = self.evaluate_expr(addr, program);
                        // truncate u32 expression into a byte,
                        // and store it into a single byte of the data vec
                        self.data[store_address as usize] = value_to_assign as u8;
                    }
                }
            }

            Expr(expr) => {
                self.evaluate_expr(expr, program);
            }

            JumpFalse(expr, label) => {
                let expr_result = self.evaluate_expr(expr, program);
                if expr_result == 0 {
                    self.instr_index = program.label_table[label];
                }
            }

            // Ignore labels, function decls, and function returns
            // Note, this means that execution can fall through into functions (TODO intended?)
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

    /// Evaluate expression and return its result as u32
    /// Note, this will run any functions called in the expr and obtain their result
    fn evaluate_expr(&mut self, expr: &Expr<'a>, program: &Program<'a>) -> u32 {
        use Expr::*;
        // TODO macro for this?
        match expr {
            Literal(n) => *n,
            Add(l, r) => self.bin_op(&l, &r, program, |a, b| a + b),
            Sub(l, r) => self.bin_op(&l, &r, program, |a, b| a - b),
            Mul(l, r) => self.bin_op(&l, &r, program, |a, b| a * b),
            Div(l, r) => self.bin_op(&l, &r, program, |a, b| a / b),
            Mod(l, r) => self.bin_op(&l, &r, program, |a, b| a % b),
            BitAnd(l, r) => self.bin_op(&l, &r, program, |a, b| a & b),
            BitOr(l, r) => self.bin_op(&l, &r, program, |a, b| a | b),
            BitXor(l, r) => self.bin_op(&l, &r, program, |a, b| a ^ b),
            Shl(l, r) => self.bin_op(&l, &r, program, |a, b| a << b),
            Shr(l, r) => self.bin_op(&l, &r, program, |a, b| a >> b),
            Lt(l, r) => self.bin_bool_op(&l, &r, program, |a, b| a < b),
            Gt(l, r) => self.bin_bool_op(&l, &r, program, |a, b| a > b),
            Leq(l, r) => self.bin_bool_op(&l, &r, program, |a, b| a <= b),
            Geq(l, r) => self.bin_bool_op(&l, &r, program, |a, b| a >= b),
            Neq(l, r) => self.bin_bool_op(&l, &r, program, |a, b| a != b),
            Eq(l, r) => self.bin_bool_op(&l, &r, program, |a, b| a == b),

            FunCall(name, args) => {
                let evaluated_args = args
                    .iter()
                    .map(|e| self.evaluate_expr(e, program))
                    .collect::<Vec<u32>>();
                self.evaluate_funcall(name, &evaluated_args, program)
                // if let Some(intrinsic_fn) = intrinsics::get_intrinsic(name) {
                //     intrinsic_fn((evaluated_args, self))
                // } else {
                //     self.evaluate_funcall(name, &evaluated_args, program)
                // }
            }
            Var(name) => get_var_value(name, &mut self.var_table, &mut self.data),
            Deref(e) => get_memory_u32(self.evaluate_expr(e, program) as usize, &self.data),
            DerefByte(e) => {
                let n = self.evaluate_expr(e, program) as usize;
                u32::from(self.data[n])
            }
            VarAddress(name) => get_var_address(name, &mut self.var_table, &mut self.data) as u32,
        }
    }

    /// Evaluate a binary operation (defined by operation parameter) on the left and right expression
    fn bin_op<F>(
        &mut self,
        left: &Expr<'a>,
        right: &Expr<'a>,
        program: &Program<'a>,
        operation: F,
    ) -> u32
    where
        F: Fn(u32, u32) -> u32,
    {
        let l = self.evaluate_expr(left, program);
        let r = self.evaluate_expr(right, program);
        operation(l, r)
    }

    /// Evaluate a binary operation (defined by operation parameter) that returns a bool (1 or 0 int)
    fn bin_bool_op<F>(
        &mut self,
        left: &Expr<'a>,
        right: &Expr<'a>,
        program: &Program<'a>,
        operation: F,
    ) -> u32
    where
        F: Fn(u32, u32) -> bool,
    {
        let l = self.evaluate_expr(left, program);
        let r = self.evaluate_expr(right, program);

        if operation(l, r) {
            1
        } else {
            0
        }
    }

    /// Executes a function (defined by name) and returns its result
    /// May be an intrinsic function or a user defined one
    fn evaluate_funcall(&mut self, name: &str, args: &[u32], program: &Program<'a>) -> u32 {
        if let Some(intrinsic_fn) = intrinsics::get_intrinsic(name) {
            // intrinsic function
            intrinsic_fn((args, self))
        } else {
            // user-defined function

            let return_instr_index = self.instr_index;
            //jump to function
            self.instr_index = program.label_table[name];

            if let IntermediateLine::FunDeclaration(_, params) = &program.code[self.instr_index] {
                for (i, param) in params.iter().enumerate() {
                    let addr = get_var_address(param, &mut self.var_table, &mut self.data);
                    let arg_value = args.get(i).copied().unwrap_or(0u32);
                    set_memory_u32(addr, &mut self.data, arg_value);
                }

                loop {
                    if let IntermediateLine::FunReturn = program.code[self.instr_index] {
                        break;
                    }
                    self.execute_line(program);
                }
                self.instr_index = return_instr_index;
                get_var_value("ans", &mut self.var_table, &mut self.data)
            } else {
                panic!("{} is not a valid function", name);
            }
        }
    }
}

/// Iterates over a program and returns the mapping from labels to the line index the label points to
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

/// Returns the address in memory (data vec) that a var points to.
/// If the var does not already exist, append a slot to memory and point the var's name to the new slot
fn get_var_address<'a>(
    name: &'a str,
    var_table: &mut HashMap<&'a str, usize>,
    data: &mut Vec<u8>,
) -> usize {
    if var_table.contains_key(name) {
        var_table[name]
    } else {
        let next_addr_in_data = data.len();
        var_table.insert(name, next_addr_in_data);
        // push 4 bytes (to fit a u32 variable)
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);
        next_addr_in_data
    }
}

/// Returns value of var based on data vec and var table
fn get_var_value<'a>(
    name: &'a str,
    var_table: &mut HashMap<&'a str, usize>,
    data: &mut Vec<u8>,
) -> u32 {
    let addr = get_var_address(name, var_table, data);
    get_memory_u32(addr, data)
}

/// Panics on out of bounds
/// TODO return result
fn get_memory_u32(index: usize, vec: &[u8]) -> u32 {
    //todo better (unsafe) way? (mem::transmute)
    u32::from(vec[index]) << 24
        | u32::from(vec[index + 1]) << 16
        | u32::from(vec[index + 2]) << 8
        | u32::from(vec[index + 3])
}

fn set_memory_u32(index: usize, vec: &mut Vec<u8>, value: u32) {
    vec[index] = (value >> 24 & 0xFF) as u8;
    vec[index + 1] = (value >> 16 & 0xFF) as u8;
    vec[index + 2] = (value >> 8 & 0xFF) as u8;
    vec[index + 3] = (value & 0xFF) as u8;
}
