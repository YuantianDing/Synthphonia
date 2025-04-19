use std::cell::RefCell;
use std::cell::Cell;


thread_local! {
    pub static LOGLEVEL: Cell<u8>  = const { Cell::new(2) };
}
thread_local! {
    pub static INDENT: RefCell<String> = RefCell::new(String::from(""));
}

/// Sets the logging level for the application. 
/// 
/// This adjusts the verbosity of the log output by setting the specified logging level within a thread-local variable. 
/// The function modifies the global logging behavior based on the provided level, influencing which log messages are output according to their priority.
/// 
pub fn set_log_level(level: u8) {
    LOGLEVEL.set(level);
}
/// Returns the current logging level as an unsigned 8-bit integer. 
/// 
/// This function retrieves the logging level by accessing a globally maintained `LOGLEVEL` value using its `get` method. 
/// The logging level is likely set elsewhere in the program, potentially allowing different components to conditionally alter their logging verbosity based on this value.
/// 
pub fn log_level() -> u8 {
    LOGLEVEL.get()
}

/// Increases the current indentation level by adding two spaces. 
/// This function accesses a thread-local string variable to modify its content by appending two spaces, effectively augmenting the indentation for subsequent operations that rely on this thread-local state.
pub fn indent() {
    INDENT.with_borrow_mut(|s| {
        s.push_str("  ");
    })
}

/// Reduces the indentation level by two steps. 
/// 
/// This function modifies a thread-local mutable state, specifically adjusting a global `INDENT` string to reflect a reduction in indentation by removing the last two characters. 
/// This is likely used in contexts where indentation levels are dynamically adjusted, such as generating or formatting text outputs where the indentation signifies scope or hierarchy.
/// 
pub fn dedent() {
    INDENT.with_borrow_mut(|s| { s.pop(); s.pop() } );
}


#[macro_export]
/// Expands a logging macro for outputting informational messages. 
/// 
/// It checks the current log level, and if it's set to 3 or higher, it processes the provided format arguments to display a message prefixed with "INFO". 
/// This message includes the current indentation level, styled with specific ANSI escape codes for color formatting, and appends the file name and line number from where the log was called. 
/// The macro ensures that informational messages are logged consistently across the module whenever the appropriate log level is set.
/// 
macro_rules! info {
    ($($fmt:expr),+) => {
        if $crate::log::LOGLEVEL.get() >= 3 {
            $crate::log::INDENT.with_borrow(|s| {
                eprintln!("{}\u{001b}[34;1m\u{001b}[1mINFO\u{001b}[0m \u{001b}[36m{:?}\u{001b}[0m {}:{}", s, format_args!($($fmt),+), file!(), line!())
            })
        }
    };
}

#[macro_export]
/// A macro for custom debug logging. 
/// 
/// This macro checks if the current logging level is set to 4 or higher, indicating that debug-level messages should be printed. 
/// If the condition is met, it formats the provided arguments using `format_args!`, and then prints them with a specific format including colored annotations for better readability in the terminal. 
/// The output includes an indentation prefix, a "DEBG" tag in green, the formatted message in cyan, and the originating file and line number. 
/// The macro utilizes thread-local storage for the indentation level, ensuring that debugging messages are neatly organized according to the current execution context.
/// 
macro_rules! debg {
    ($($fmt:expr),+) => {
        if $crate::log::LOGLEVEL.get() >= 4 {
            $crate::log::INDENT.with_borrow(|s| {
                eprintln!("{}\u{001b}[32mDEBG\u{001b}[0m \u{001b}[36m{:?}\u{001b}[0m {}:{}", s, format_args!($($fmt),+), file!(), line!())
            })
        }
    };
}

#[macro_export]
/// Expands to produce a debug logging statement for high verbosity levels. 
/// 
/// 
/// The macro is designed to output formatted debug messages to the standard error stream when a global logging level set within the crate is 5 or higher. 
/// It formats the message using provided expressions, marks the message with "DEBG" using ANSI color coding for enhanced terminal visibility, and appends the current file and line number for precise location tracking. 
/// This enables developers to efficiently trace and debug program execution, especially when dealing with complex synthesis algorithms in the Synthphonia module. 
/// The macro relies on a thread-local storage for consistent formatting in multi-threaded scenarios.
macro_rules! debg2 {
    ($($fmt:expr),+) => {
        if $crate::log::LOGLEVEL.get() >= 5 {
            $crate::log::INDENT.with_borrow(|s| {
                eprintln!("{}\u{001b}[32mDEBG\u{001b}[0m \u{001b}[36m{:?}\u{001b}[0m {}:{}", s, format_args!($($fmt),+), file!(), line!())
            })
        }
    };
}

#[macro_export]
/// Logs critical messages based on the defined logging level. 
/// 
/// This macro evaluates whether the current logging level is at least 1 and, if so, prints a formatted critical log message to standard error. 
/// The message is prefixed with an indentation from a thread-local storage followed by the formatted critical prefix in bold red. 
/// The message content is then followed by the related source file and line number of the log statement for easier debugging and tracing. 
/// The formatting allows developers to have an immediate visual indication of a critical logging event in their output.
/// 
macro_rules! crit {
    ($($fmt:expr),+) => {
        if $crate::log::LOGLEVEL.get() >= 1 {
        $crate::log::INDENT.with_borrow(|s| {
            eprintln!("{}\u{001b}[31;1m\u{001b}[1mCRIT\u{001b}[0m \u{001b}[36m{:?}\u{001b}[0m {}:{}", s, format_args!($($fmt),+), file!(), line!())
        })
    }
    };
}

#[macro_export]
/// This macro generates a warning message. 
/// 
/// It formats and prints a warning when the global log level is set to 2 or higher. 
/// It leverages a thread-local storage for indentation to structure the output appropriately, highlighting the word "WARN" and the formatted message with ANSI escape codes to apply color and style. 
/// The macro uses `eprintln!` to output the warning to the standard error stream, including the file name and line number where the macro is invoked, enhancing debugging and log tracing capabilities.
/// 
macro_rules! warn {
    ($($fmt:expr),+) => {
        if $crate::log::LOGLEVEL.get() >= 2 {
            $crate::log::INDENT.with_borrow(|s| {
                eprintln!("{}\u{001b}[33;1m\u{001b}[1mWARN\u{001b}[0m \u{001b}[36m{:?}\u{001b}[0m {}:{}", s, format_args!($($fmt),+), file!(), line!())
            })
        }
    };
}

#[macro_export]
/// Evaluates a given expression with conditional logging of informational messages based on the current log level. 
/// 
/// This macro checks if the logging level is set to 3 or higher, indicating verbose output. 
/// If the condition is met, it formats a log message with a custom indentation and colored "INFO" prefix, appending the source file name and line number. 
/// The macro uses thread-local storage to manage log indentation, ensuring that nested logging maintains a consistent structure. 
/// It then executes the provided expression while temporarily adjusting the indentation, allowing for nested operations to be clearly delineated in the log output. 
/// If the log level is below 3, the expression is evaluated without additional logging.
/// 
macro_rules! infob {
    ($fmt:literal, $e:expr) => {
        if $crate::log::LOGLEVEL.get() >= 3 {
            $crate::log::INDENT.with_borrow(|s| {
                eprintln!("{}\u{001b}[36m\u{001b}[1mINFO\u{001b}[0m \u{001b}[36m{:?}\u{001b}[0m {}:{}", s, format_args!($fmt), file!(), line!())
            });
            $crate::log::indent();
            let _result_ = $e;
            $crate::log::dedent();
            _result_
        } else {
            $e
        }
    };
}

#[macro_export]
/// Expands into a conditional logging block for debugging expressions. 
/// 
/// It evaluates whether the log level is sufficiently high (4 or above) to output detailed debug information. 
/// If the condition is met, it inserts an entry into the log with a specific format that highlights the debug message and includes the file and line number where the macro is used. 
/// The macro adjusts the indentation context before and after the expression is evaluated to maintain a coherent log structure. 
/// Regardless of the log level, the expression is always executed, and its result is returned. 
/// This macro is useful for temporarily inserting detailed logging around specific sections of code, especially during development and debugging sessions.
/// 
macro_rules! debgb {
    ($fmt:literal, $e:expr) => {
        if $crate::log::LOGLEVEL.get() >= 4 {
            $crate::log::INDENT.with_borrow(|s| {
                eprintln!("{}\u{001b}[32mDEBG\u{001b}[0m \u{001b}[36m{:?}\u{001b}[0m {}:{}", s, format_args!($fmt), file!(), line!())
            });
            $crate::log::indent();
            let _result_ = $e;
            $crate::log::dedent();
            _result_
        } else {
            $e
        }
    };
}

#[macro_export]
/// Expands to a logging and evaluation mechanism. 
/// 
/// This macro evaluates an expression, logging detailed debug information if the current logging level is set to 5 or higher. 
/// It leverages the project's logging infrastructure to output a well-formatted message showing the provided format string and its evaluated result, along with the file and line number where the macro is invoked. 
/// The macro manipulates an indent level to ensure the logs maintain structured readability through related log invocations. 
/// In case the log level condition is not met, it simply evaluates the expression without any additional logging.
/// 
macro_rules! debgb2 {
    ($fmt:literal, $e:expr) => {
        if $crate::log::LOGLEVEL.get() >= 5 {
            $crate::log::INDENT.with_borrow(|s| {
                eprintln!("{}\u{001b}[32mDEBG\u{001b}[0m \u{001b}[36m{:?}\u{001b}[0m {}:{}", s, format_args!($fmt), file!(), line!())
            });
            $crate::log::indent();
            let _result_ = $e;
            $crate::log::dedent();
            _result_
        } else {
            $e
        }
    };
}
