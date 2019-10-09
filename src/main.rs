
#[allow(clippy::all)]
#[allow(warnings)]
mod grammar;

mod ast;
mod intermediate_repr;
mod interpreter;
mod util;

fn main() {
    // println!("{:#?}", grammar::lines("x*3+1->x"));

    let pro = include_str!("program.brown");
    let (data, code) = grammar::program(pro).unwrap();
    let code = intermediate_repr::to_intermediate_repr(code);
    let (data, data_label_table) = intermediate_repr::convert_data_segment(data);
    let label_table = interpreter::build_label_table(&code);

    println!("{:#?}", code);

    let program = interpreter::Program {
        code,
        data,
        data_label_table,
        label_table
    };

    interpreter::execute(&program);
}
