# BrownCode
BrownCode is a simple interpreted programming lanuguage.

Main ideas:
- All data is unsigned 32 bit numbers.
- Programs contain a `.DATA` and `.CODE` section.
- Data can be baked into the program using the `.DATA` section of a program. This is similar to the `.data` section in assembly.
- The `.DATA` section is mutable while a program is running.
- Even variables defined in the code are appended to the `.DATA` section when they are encountered by the interpreter for the first time.
- All data is accessed through labels and offsets (variables are just labels in the data section)
- All variables are global (mostly... see `savearg` function modifier).


## Syntax
Program is split into `.DATA` and `.CODE` segments. `.DATA` can only contain data definitions, `.CODE` can only contain code.
```
#!/usr/bin/browncode
.DATA
//data definitions
.CODE
//code
```
### Data section
Syntax | Description
--- | ---
`label:` | `label` points to the following byte index into the data
`zeros 5` | Insert 5 zeros
`{05FF}` | Insert the bytes 0x05 and 0xFF
`{HEX}` | Insert a byte for every 2 hex characters (must be an even number of characters)
`b230` | Insert the unsigned decimal byte 230
`503282` | Insert the unsigned 32 bit number 503282
`0b01101111` | Insert the binary byte `0110 1111`
`"hello"` | Insert the UTF-8 string. Not null terminated, you must add a `b0` after to null terminate

Like code, data lines must either be separated by newline(s), or a colon.

### Code section
#### Expressions
Syntax | Description
--- | ---
`234` | Decimal number
`0x23FA6E00` | Hex number
`0b1100` | Binary number (up to 32 digits)
`{EXPR}` | The value of the data segment at address EXPR (big endian bytewise addresssing)
`[EXPR]` | The 8 bit value at address EXPR, auto extended to 32 bits
`&var` | The address of `var`
`!EXPR` | Logical inversion of EXPR. (EXPR != 0 is true, EXPR = 0 is false)
`(EXPR)` | Evaluates to EXPR
`EXPR op EXPR` | Perform the operator on EXPRs. op can be `| ^ & = != < > <= >= << >> + - * / %`. C-like order of operations is respected.
`FUNC(ARG, ARG)` | Calls FUNC with ARGs (may be any number, including 0, args), evaluates to the function's return

#### Control Flow / Top Level Syntax
The syntax is generally line based, but `:` is interpreted as a newline. Except for newlines/`:`, it is whitespace independent (indentation tabs/spaces do not matter).

Eg.
```
if a
    function(b)
else
    function(c)
end
```
is equivalent to
```
if a:function(b):else:function(c):end
```
Syntax | Description
--- | ---
`//comment` | comment
`EXPR -> VAR` | store result of EXPR into VAR
`EXPR1 -> [EXPR2]` | truncate EXPR1 into an 8 bit value, and store it into the single byte where EXPR2 points
`EXPR1 -> {EXPR2}` | store the result of EXPR1 into the 32 bits where EXPR2 points (big endian)
`for VAR, EXPR1, EXPR2:CODE:end` | loop over CODE, incrementing VAR. Start with VAR = EXPR1, end with VAR = EXPR2 - 1. VAR in [EXPR1, EXPR2)
`while EXPR:CODE:end` | loop over CODE while EXPR is non-zero
`if EXPR:CODE:end` | only execute CODE if EXPR is non-zero
`if EXPR:CODE:else:CODE:end` | execute first block if EXPR is non-zero, otherwise execute second block of code
`LABEL:` | introduce LABEL that points to the following line of code
`goto LABEL` | unconditionally jump to LABEL in the code
`NAME(ARG, ARG)` | calls the function NAME with args, discarding the result
`func NAME(ARG, ARG):CODE:end` | define function NAME, which takes the args given. Args must be names, not expressions.
`savearg func NAME(ARG, ARG):CODE:end` | define a function NAME, which takes the args given. (see below)
#### Functions
Functions take arguments, and return a single result. The result is returned by setting a variable named `ans`. Whatever value `ans` has when the function returns will be the return value. Early returns are possible through labels and gotos.

By default, functions will just set each argument variable defined in their declaration to the calling value. This will clobber other variables by the same name. Example
```
3 -> x
add1(5) -> y
// x now unexpectedly equals 5

func add1(x)
    // on calling, 5 is stored in x, which clobbers the previous value of 3
    x + 1 -> ans
end
```

This problem can be mitigated by using `savearg func`. This saved the values of the variables used as function parameters, and restores them on function return. In the above example, the value of `3` for x would be saved when `add1` is called, and restored when `add1` returns.

## Intrinsics
Intrinsic | Description
--- | ---
`numprintln(args...)` | prints each argument on a newline as a unsigned 32 bit decimal
`numprint(args...)` | prints each argument as a unsigned 32 bit decimal
`printchar(char)` | prints the character
`exit()` | ends the program
`random()` | returns a random 32 bit number
`randomrange(start, end)` | returns a number between start and end (TODO inclusive/exclusive?)
`present()` | copies the buffered graphics to the screen
`drawcolor(color)` | sets the draw color. Uses RGBA8888 format
`pixel(x, y)` | draws pixel at x and y
`fillrect(x, y, w, h)` | draws filled rectangle
`line(x0, y0, x1, y1)` | draws line
`keypressed(scancode)` | returns 1 if the scancode is currently pressed, 0 if not. Uses [SDL2 scancodes](https://wiki.libsdl.org/SDLScancodeLookup)
`clear()` | clears the screen with the specified draw color. Does not respect the transparency of the draw color
`delay(ms)` | pauses for specified amount of milliseconds
`pollexit()` | checks if the window was closed by the user, and exits if it has. You should call this periodically if you want the window to be closeable.
`createmonosprite(ptr, w, h, color)` | creates a monochromatic sprite of given width and height. The bitwise data at pointer describes the sprite data (1 = specified color, 0 = transparent). Width must be a multiple of 8. Returns a sprite index which can be used to refer to this sprite when drawing it. NOTE index is always incremented by 1 between succesive calls, so this can be called in a loop while only storing the first index.