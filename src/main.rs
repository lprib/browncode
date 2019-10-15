// Allow warnings on generated code
#[allow(clippy::all)]
#[allow(warnings)]
mod grammar;

mod ast;
mod graphics;
mod intermediate_repr;
mod interpreter;
mod util;

fn main() {
    // let mut g = graphics::Graphics::try_new().unwrap();
    // let c = g.get_sprite_creator();
    // let mut s = graphics::Sprites::new(&c);
    // s.create_sprite_mono(
    //     &[0b0110_1010, 0b10101010, 0b01101001, 0b10101010],
    //     8,
    //     4,
    //     0x00FF0005,
    // );
    // g.draw_color(0xFF000030);
    // g.fill_rect(20, 20, 20, 20);
    // g.draw_color(0xFFFF_00FF);
    // loop {
    //     // g.pixel(2, 2);
    //     g.sprite(&s, 0, 0, 0);
    //     g.present();
    //     g.delay(100);
    //     if g.is_key_pressed(41) {
    //         std::process::exit(0);
    //     }
    //     g.poll_events();
    // }

    let pro = include_str!("program3.brown");
    let (data, code) = grammar::program(pro).unwrap();
    // println!("{:#?}", code);
    let code = intermediate_repr::to_intermediate_repr(code);
    let (data, data_label_table) = intermediate_repr::convert_data_segment(data);
    let label_table = interpreter::build_label_table(&code);

    // println!("{:#?}", code);

    let program = interpreter::Program {
        code,
        data,
        data_label_table,
        label_table,
    };

    interpreter::execute(&program);
}
