// This macro makes it easier to work with raw terminal output,
// with this macro, there is no need to specify format specifiers e.g "{}{}{}"
// following the DRY rule, it also has some shorthand for colors, gotos,
// clear, and "runcolor" (which is used to specify the color according to the
// state of the counter.
#[macro_export]
macro_rules! writeraw {
    // Base case
    ($out:expr$(,)?) => {};

    // Clears Screen
    ($out:expr, clear$(, $($ts:tt)*)?) => {{
        use termion::clear;
        write!($out, "{}", clear::All).unwrap();
        $(writeraw!($out, $($ts)*);)?
    }};

    // Set color according to a boolean specifying if counter is running
    ($out:expr, runcolor $running:expr$(, $($ts:tt)*)?) => {{
        use termion::color::{Green, Red, Fg};
        if $running {
            write!($out, "{}", Fg(Green)).unwrap();
        } else {
            write!($out, "{}", Fg(Red)).unwrap();
        }
        $(writeraw!($out, $($ts)*);)?
    }};

    // Set color for terminal
    ($out:expr, color $color:expr$(, $($ts:tt)*)?) => {{
        use termion::color;
        write!($out, "{}", color::Fg($color)).unwrap();
        $(writeraw!($out, $($ts)*);)?
    }};

    // Goto a particular column and line
    ($out:expr, ($x:expr, $y:expr)$(, $($ts:tt)*)?) => {{
        use termion::cursor::Goto;
        write!($out, "{}", Goto($x, $y)).unwrap();
        $(writeraw!($out, $($ts)*);)?
    }};

    // Prints text onto terminal
    ($out:expr, text $t:expr$(, $($ts:tt)*)?) => {{
        write!($out, "{}", $t).unwrap();
        $(writeraw!($out, $($ts)*);)?
    }};

    // The rules below are for printing on a line with swapped orders
    // These special rules allow for text to be specified first before color and goto
    // somewhat maintaining the readability
    ($out:expr, %text $t:expr, color $color:expr, ($x:expr, $y:expr)%$(, $($ts:tt)*)?) => {{
        use termion::{color::Fg, cursor::Goto};
        write!($out, "{co}{go}{te}", te = $t, go = Goto($x, $y), co = Fg($color)).unwrap();
        $(writeraw!($out, $($ts)*);)?
    }};

    ($out:expr, %text $t:expr, runcolor $running:expr, ($x:expr, $y:expr)%$(, $($ts:tt)*)?) => {{
        use termion::cursor::Goto;
        writeraw!($out, runcolor $running);
        write!($out, "{go}{te}", te = $t, go = Goto($x, $y)).unwrap();
        $(writeraw!($out, $($ts)*);)?
    }};

    ($out:expr, %text $t:expr, ($x:expr, $y:expr)%$(, $($ts:tt)*)?) => {{
        use termion::cursor::Goto;
        write!($out, "{go}{te}", te = $t, go = Goto($x, $y)).unwrap();
        $(writeraw!($out, $($ts)*);)?
    }};
}
