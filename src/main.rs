
#[allow(dead_code)]
#[allow(clippy::all)]
mod grammar;

mod ast;
mod intermediate_repr;

fn main() {
    let pro = include_str!("program.brown");
    let block = grammar::lines(pro).unwrap();
    println!("{:#?}", block);

    let block = intermediate_repr::to_intermediate_repr(block);
    println!("{:#?}", block);
}
