//! intrinsic functions (standard library)

use super::InterpreterState;
use lazy_static::lazy_static;
use std::char;


type IntrinsicFn = fn(IntrinsicFnArgs) -> u32;
type IntrinsicFnArgs<'a, 'b> = (&'a [u32], &'a mut InterpreterState<'b>);

lazy_static! {
    /// Maps names of instrinsic functions to their definition
    /// Uses array with linear search at the moment (can be converted to HashMap)
    static ref INTRINSICS: &'static [(&'static str, IntrinsicFn)] = &[
        ("println", println),
        ("print", print),
        ("puts", puts),
        ("putc", putc),
        ("exit", exit),
        ("present", present),
        ("draw_color", draw_color),
        ("pixel", pixel),
        ("fill_rect", fill_rect),
        ("key_pressed", key_pressed),
        ("clear", clear),
        ("delay", delay),
        ("poll_events", poll_events),
        ("create_sprite_mono", create_sprite_mono),
        ("sprite", sprite),
    ];
}

/// Get instrinsic function from name
pub fn get_intrinsic(name: &str) -> Option<&IntrinsicFn> {
    for (ref f_name, f) in INTRINSICS.iter() {
        if &name == f_name {
            return Some(f);
        }
    }
    None
}

fn println((args, _): IntrinsicFnArgs) -> u32 {
    if args.is_empty() {
        println!();
        return 0;
    }

    for arg in args {
        println!("{}", arg);
    }
    0
}

fn print((args, _): IntrinsicFnArgs) -> u32 {
    for arg in args {
        print!("{}", arg);
    }
    0
}

fn puts((args, state): IntrinsicFnArgs) -> u32 {
    let mut i = args[0] as usize;
    while state.data[i] != 0 {
        print!("{}", state.data[i] as char);
        i += 1;
    }
    0
}

fn putc((args, _): IntrinsicFnArgs) -> u32 {
    print!("{}", char::from_u32(args[0]).unwrap());
    0
}

fn exit(_: IntrinsicFnArgs) -> u32 {
    std::process::exit(0)
}

//GRAPHICS ROUTINES
fn present((_, state): IntrinsicFnArgs) -> u32 {
    state.graphics.present();
    0
}

fn draw_color((args, state): IntrinsicFnArgs) -> u32 {
    state.graphics.draw_color(args[0]);
    0
}

fn pixel((args, state): IntrinsicFnArgs) -> u32 {
    state.graphics.pixel(args[0], args[1]);
    0
}

fn fill_rect((args, state): IntrinsicFnArgs) -> u32 {
    state.graphics.fill_rect(args[0], args[1], args[2], args[3]);
    0
}

fn key_pressed((args, state): IntrinsicFnArgs) -> u32 {
    if state.graphics.is_key_pressed(args[0]) {
        1
    } else {
        0
    }
}

fn clear((_, state): IntrinsicFnArgs) -> u32 {
    state.graphics.clear();
    0
}

fn delay((args, state): IntrinsicFnArgs) -> u32 {
    state.graphics.delay(args[0]);
    0
}

fn poll_events((_, state): IntrinsicFnArgs) -> u32 {
    state.graphics.poll_events();
    0
}

fn create_sprite_mono((args, state): IntrinsicFnArgs) -> u32 {
    // must be a multiple of 8
    let w = args[1];
    let h = args[2];
    let color = args[3];
    let sprite_data = &state.data[args[0] as usize..(args[0] + (w/8) * h) as usize];
    state.sprites.create_sprite_mono(sprite_data, w, h, color)
}

fn sprite((args, state): IntrinsicFnArgs) -> u32 {
    state.graphics.sprite(&state.sprites, args[0], args[1], args[2]);
    0
}