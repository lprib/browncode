//! intrinsic functions (standard library)

use super::{error::Error, IResult, InterpreterState};
use lazy_static::lazy_static;
use rand::Rng;
use std::char;
use std::io::{stdout, Write};

/// This is passed as a single argument to instrinsic functions
type IntrinsicFnArgs<'a, 'b> = (&'a [u32], &'a mut InterpreterState<'b>);

/// Represents how many args an intrinsic function expects.
/// If ExpectedArgs::Fixed, the function can take any of the specified argument counts from the slice.
enum ExpectedArgs {
    Fixed(&'static [usize]),
    VarArg,
}

/// Represents an intrinsic function.
struct Intrinsic {
    name: &'static str,
    expected_args: ExpectedArgs,
    f: fn(IntrinsicFnArgs) -> IResult<u32>,
}

/// Attempts to execute an intrinsic, returning Some(intrinsic_return) or None if the instrinsic doesnt exist.
/// Panics if the wrong number of arguments are passed.
pub fn try_execute_intrinsic(
    name: &str,
    args: &[u32],
    state: &mut InterpreterState,
) -> Option<IResult<u32>> {
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
                        .ok_or_else(|| Error::IntrinsicArgumentMismatch {
                            expected: arg_lens,
                            got: args.len(),
                            func_name: name.to_string(),
                        })?;
                }
                ExpectedArgs::VarArg => { /*No arg length checking for varargs*/ }
            }
            (intrinsic.f)((args, state))
        })
}

/// Syntactic sugar for instantiating an Intrinsic{..} struct.
/// `intrinsic!(name, [accepted_arg_count_1, accepted_arg_count_2, ...], argument_binding_pattern => {body})`
/// alternatively can use `vararg` instead of accepted_arg_count list.
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
        intrinsic!(numprintln, vararg, (args, _) => {
            if args.is_empty() {
                println!();
                return Ok(0);
            }

            for arg in args {
                println!("{}", arg);
            }
            Ok(0)
        }),
        intrinsic!(numprint, vararg, (args, _) => {
            for arg in args {
                print!("{}", arg);
            }
            flush_stdout()?;
            Ok(0)
        }),
        intrinsic!(print, [1], (args, state) => {
            let mut i = args[0] as usize;
            while state.get_memory_u8(i)? != 0 {
                print!("{}", state.get_memory_u8(i)? as char);
                i += 1;
            }
            flush_stdout()?;
            Ok(0)
        }),
        //TODO println for strings
        intrinsic!(printchar, [1], (args, _) => {
            print!("{}", char::from_u32(args[0]).ok_or(Error::InvalidCharacterValue(args[0]))?);
            flush_stdout()?;
            Ok(0)
        }),
        intrinsic!(exit, [0], _ => {
            std::process::exit(0)
        }),

        intrinsic!(random, [0], _ => {
            Ok(rand::random())
        }),
        intrinsic!(randomrange, [2], (args, _) => {
            Ok(rand::thread_rng().gen_range(args[0], args[1]))
        }),

        intrinsic!(present, [0], (_, state) => {
            state.graphics.present();
            Ok(0)
        }),

        intrinsic!(drawcolor, [1], (args, state) => {
            state.graphics.draw_color(args[0]);
            Ok(0)
        }),
        intrinsic!(pixel, [2], (args, state) => {
            state.graphics.pixel(args[0], args[1]);
            Ok(0)
        }),
        intrinsic!(fillrect, [4], (args, state) => {
            state.graphics.fill_rect(args[0], args[1], args[2], args[3]);
            Ok(0)
        }),
        intrinsic!(line, [4], (args, state) => {
            state.graphics.line(args[0], args[1], args[2], args[3]);
            Ok(0)
        }),
        intrinsic!(keypressed, [1], (args, state) => {
            Ok(if state.graphics.is_key_pressed(args[0]) {
                1
            } else {
                0
            })
        }),
        intrinsic!(clear, [0], (_, state) => {
            state.graphics.clear();
            Ok(0)
        }),
        intrinsic!(delay, [1], (args, state) => {
            state.graphics.delay(args[0]);
            Ok(0)
        }),
        intrinsic!(pollexit, [0], (_, state) => {
            state.graphics.poll_exit();
            Ok(0)
        }),
        intrinsic!(createmonosprite, [4], (args, state) => {
            // must be a multiple of 8
            let w = args[1];
            let h = args[2];
            let color = args[3];
            let data_start_index = args[0] as usize;
            let data_end_index = (args[0] + (w / 8) * h) as usize;

            // do the bounds check manually
            if data_end_index >= state.data.len() || data_start_index >= state.data.len() {
                return Err(Error::U8OutOfBounds {
                    u8_read_index: data_end_index,
                    memory_length: state.data.len(),
                });
            }

            // bounds check has already happened, so we can use get_unchecked
            let sprite_data = unsafe { &state.data.get_unchecked(data_start_index..data_end_index) };
            Ok(state.sprites.create_sprite_mono(sprite_data, w, h, color))
        }),
        intrinsic!(sprite, [3], (args, state) => {
            state
                .graphics
                .sprite(&state.sprites, args[0], args[1], args[2]);
            Ok(0)
        })
    ];
}

fn flush_stdout() -> IResult<()> {
    stdout()
        .flush()
        .map_err(|_| Error::System(String::from("Unable to flush stdout")))?;
    Ok(())
}
