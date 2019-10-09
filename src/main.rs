
#[allow(clippy::all)]
#[allow(warnings)]
mod grammar;

mod ast;
mod intermediate_repr;
mod interpreter;

fn main() {
    // println!("{:#?}", grammar::lines("x*3+1->x"));

    let pro = include_str!("program.brown");
    // let block = grammar::lines(pro).unwrap();
    // println!("{:#?}", block);

    // let block = intermediate_repr::to_intermediate_repr(block);
    // println!("{:#?}", block);

    // let mut interpreter = interpreter::Interpreter::new(block);
    // interpreter.execute();

    let b = grammar::data_segment(pro).unwrap();
    println!("{:#?}", b);
    let (x, y) = intermediate_repr::convert_data_segment(b);
    println!("{:#?}\n{:#?}", x, y);
}
