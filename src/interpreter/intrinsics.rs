//! intrinsic functions (standard library)

use super::InterpreterState;
use lazy_static::lazy_static;
use rand::Rng;
use std::char;

type IntrinsicFnArgs<'a, 'b> = (&'a [u32], &'a mut InterpreterState<'b>);

enum ExpectedArgs {
    Fixed(&'static [usize]),
    VarArg,
}

struct Intrinsic {
    name: &'static str,
    expected_args: ExpectedArgs,
    f: fn(IntrinsicFnArgs) -> u32,
}

pub fn try_execute_intrinsic(
    name: &str,
    args: &[u32],
    state: &mut InterpreterState,
) -> Option<u32> {
    INTRINSICS
        .iter()
        // find the intrinsic that matches the name
        .find(
            |Intrinsic {
                 name: test_name, ..
             }| test_name == &name,
        )
        // map intrinsic to its return value
        .map(|intrinsic| {
            // check arg lengths
            match intrinsic.expected_args {
                ExpectedArgs::Fixed(arg_lens) => {
                    arg_lens
                        .iter()
                        .find(|&&arg_len| arg_len == args.len())
                        .or_else(|| {
                            panic!(
                                "{} expects {:?} arguments, {} given",
                                name,
                                arg_lens,
                                args.len()
                            )
                        });
                }
                ExpectedArgs::VarArg => { /*No arg length checking for varargs*/ }
            }
            (intrinsic.f)((args, state))
        })
}

macro_rules! intrinsic {
    ($name:ident, [ $( $expected_arg:expr ),+ ], $arguments:pat => $fn:block) => {
        Intrinsic {
            name: stringify!($name),
            expected_args: ExpectedArgs::Fixed(&[ $($expected_arg),* ]),
            f: |$arguments| $fn,
        }
    };

    ($name:ident, vararg, $arguments:pat => $fn:block) => {
        Intrinsic {
            name: stringify!($name),
            expected_args: ExpectedArgs::VarArg,
            f: |$arguments| $fn,
        }
    }
}
lazy_static! {
    static ref INTRINSICS: &'static [Intrinsic] = &[
        intrinsic!(println, vararg, (args, _) => {
            if args.is_empty() {
                println!();
                return 0;
            }

            for arg in args {
                println!("{}", arg);
            }
            0
        }),
        intrinsic!(print, vararg, (args, _) => {
            for arg in args {
                print!("{}", arg);
            }
            0
        }),
        intrinsic!(puts, [1], (args, _) => {
            for arg in args {
                print!("{}", arg);
            }
            0
        }),
        intrinsic!(putc, [1], (args, _) => {
            print!("{}", char::from_u32(args[0]).unwrap());
            0
        }),
        intrinsic!(exit, [0], _ => {
            std::process::exit(0)
        }),

        intrinsic!(random, [0], _ => {
            rand::random()
        }),
        intrinsic!(random_range, [2], (args, _) => {
            rand::thread_rng().gen_range(args[0], args[1])
        }),

        intrinsic!(present, [0], (_, state) => {
            state.graphics.present();
            0
        }),

        intrinsic!(draw_color, [1], (args, state) => {
            state.graphics.draw_color(args[0]);
            0
        }),
        intrinsic!(pixel, [2], (args, state) => {
            state.graphics.pixel(args[0], args[1]);
            0
        }),
        intrinsic!(fill_rect, [4], (args, state) => {
            state.graphics.fill_rect(args[0], args[1], args[2], args[3]);
            0
        }),
        intrinsic!(key_pressed, [1], (args, state) => {
            if state.graphics.is_key_pressed(args[0]) {
                1
            } else {
                0
            }
        }),
        intrinsic!(clear, [0], (_, state) => {
            state.graphics.clear();
            0
        }),
        intrinsic!(delay, [1], (args, state) => {
            state.graphics.delay(args[0]);
            0
        }),
        intrinsic!(poll_events, [0], (_, state) => {
            state.graphics.poll_events();
            0
        }),
        intrinsic!(create_sprite_mono, [4], (args, state) => {
            // must be a multiple of 8
            let w = args[1];
            let h = args[2];
            let color = args[3];
            let sprite_data = &state.data[args[0] as usize..(args[0] + (w / 8) * h) as usize];
            state.sprites.create_sprite_mono(sprite_data, w, h, color)
                }),
                intrinsic!(sprite, [3], (args, state) => {
                    state
                .graphics
                .sprite(&state.sprites, args[0], args[1], args[2]);
            0
        })
    ];
}
