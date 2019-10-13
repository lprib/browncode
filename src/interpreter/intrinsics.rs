use super::InterpreterState;
use crate::graphics::Graphics;
use lazy_static::lazy_static;
use std::char;

type IntrinsicFn = fn(Vec<u32>, &mut InterpreterState) -> u32;

lazy_static! {
    static ref INTRINSICS: &'static [(&'static str, IntrinsicFn)] = &[
        ("println", println),
        ("print", print),
        ("puts", puts),
        ("putc", putc),
        ("exit", exit),
        ("present", present),
        ("draw_color", draw_color),
        ("draw_pixel", draw_pixel),
        ("key_pressed", key_pressed),
        ("clear", clear),
        ("delay", delay),
        ("poll_events", poll_events)
    ];
}

pub fn get_intrinsic(name: &str) -> Option<&IntrinsicFn> {
    for (ref f_name, f) in INTRINSICS.iter() {
        if &name == f_name {
            return Some(f);
        }
    }
    None
}

fn println(args: Vec<u32>, _: &mut InterpreterState) -> u32 {
    if args.is_empty() {
        println!();
        return 0;
    }

    for arg in args {
        println!("{}", arg);
    }
    0
}

fn print(args: Vec<u32>, _: &mut InterpreterState) -> u32 {
    for arg in args {
        print!("{}", arg);
    }
    0
}

fn puts(args: Vec<u32>, state: &mut InterpreterState) -> u32 {
    let mut i = args[0] as usize;
    while state.data[i] != 0 {
        print!("{}", state.data[i] as char);
        i += 1;
    }
    0
}

fn putc(args: Vec<u32>, _: &mut InterpreterState) -> u32 {
    print!("{}", char::from_u32(args[0]).unwrap());
    0
}

fn exit(_: Vec<u32>, _: &mut InterpreterState) -> u32 {
    std::process::exit(0)
}

//GRAPHICS ROUTINES
fn present(_: Vec<u32>, state: &mut InterpreterState) -> u32 {
    state.graphics.present();
    0
}

fn draw_color(args: Vec<u32>, state: &mut InterpreterState) -> u32 {
    state.graphics.set_draw_color(args[0]);
    0
}

fn draw_pixel(args: Vec<u32>, state: &mut InterpreterState) -> u32 {
    state.graphics.draw_pixel(args[0], args[1]);
    0
}

fn key_pressed(args: Vec<u32>, state: &mut InterpreterState) -> u32 {
    if state.graphics.is_key_pressed(args[0]) {
        1
    } else {
        0
    }
}

fn clear(_: Vec<u32>, state: &mut InterpreterState) -> u32 {
    state.graphics.clear();
    0
}

fn delay(args: Vec<u32>, state: &mut InterpreterState) -> u32 {
    state.graphics.delay(args[0]);
    0
}

fn poll_events(_: Vec<u32>, state: &mut InterpreterState) -> u32 {
    state.graphics.poll_events();
    0
}