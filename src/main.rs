// Allow warnings on generated code
#[allow(clippy::all)]
#[allow(warnings)]
mod grammar;

mod ast;
mod graphics;
mod intermediate_repr;
mod interpreter;
mod util;

use std::fs::read_to_string;
use std::path::PathBuf;
use structopt::clap::arg_enum;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    enum OutputType {
        Ast,
        PrettyAst,
        DataAst,
        Ir,
        Run
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "brown", author = "Liam Pribis")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input_file: PathBuf,

    #[structopt(short = "t", long = "output-type", possible_values = &OutputType::variants(), case_insensitive = true, default_value = "run")]
    output_type: OutputType,
}

fn main() -> Result<(), String> {
    let opt = Opt::from_args();

    let program =
        read_to_string(opt.input_file).map_err(|_| String::from("Could not read file"))?;
    let (data, ast) = grammar::program(&program).map_err(|e| e.to_string())?;
    match opt.output_type {
        OutputType::Ast => {
            println!("{:?}", ast);
            Ok(())
        }

        OutputType::PrettyAst => {
            println!("{:#?}", ast);
            Ok(())
        }

        OutputType::DataAst => {
            println!("{:#?}", data);
            Ok(())
        }

        OutputType::Ir => {
            let ir = intermediate_repr::to_intermediate_repr(ast);
            println!("{}", intermediate_repr::display_intermediate_block(&ir));
            Ok(())
        }

        OutputType::Run => {
            let ir = intermediate_repr::to_intermediate_repr(ast);
            let (data, data_label_table) = intermediate_repr::convert_data_segment(data);
            let label_table = interpreter::build_label_table(&ir).map_err(|e| e.to_string())?;
            let program = interpreter::Program {
                ir,
                data,
                data_label_table,
                label_table,
            };

            interpreter::execute(&program).map_err(|e| e.to_string())?;
            Ok(())
        }
    }
}
