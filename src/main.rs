#[allow(clippy::all)]
#[allow(warnings)]
mod grammar;

mod ast;
mod graphics;
mod intermediate_repr;
mod interpreter;
mod util;

use sdl2::keyboard::Scancode;

fn main() {
    // let mut g = graphics::Graphics::try_new().unwrap();
    // g.set_draw_color(0xFFFF00FF);
    // loop {
    //     g.draw_pixel(2, 2);
    //     g.present();
    //     g.delay(100);
    //     if g.is_key_pressed(41) {
    //         std::process::exit(0);
    //     }
    //     g.poll_events();
    // }

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
        label_table,
    };

    interpreter::execute(&program);
}
