mod word_search;
mod kanji_search;
mod sentence_search;
mod util;
use std::{
    io::{stdin, stdout, Write},
    process::{Command, Stdio},
    env,
};

use word_search::word_search;
use kanji_search::search_by_radical;
use sentence_search::sentence_search;


use argparse::{ArgumentParser, List, Print, Store, StoreTrue};
use serde_json::Value;
use atty::Stream;

macro_rules! JISHO_URL {
    () => {
        "https://jisho.org/api/v1/search/words?keyword={}"
    };
}
macro_rules! TATOEBA_URL_ENG_QUERY { () => {
        "https://tatoeba.org/en/api_v0/search?from=eng&orphans=no&to=jpn&unapproved=no&query={}"
    };
}
macro_rules! TATOEBA_URL_JPN_QUERY { () => {
        "https://tatoeba.org/en/api_v0/search?from=jpn&orphans=no&to=eng&unapproved=no&query={}"
    };
}

fn main() -> Result<(), ureq::Error> {

    let term_size = if atty::is(Stream::Stdout) {
        terminal_size().unwrap_or(0)
    } else {
        0
    };

    let options = parse_args();

    let mut query = String::new();
    let mut output = String::with_capacity(51200); /* Give output 50KiB of buffer; Should be enough to avoid reallocs*/
    query = options.query.trim().to_string().clone();

    loop {
        if options.interactive || options.query.trim().is_empty() {
            while query.is_empty() || query == ":" || query == "：" || query == "_" || query == "＿" {
                query.clear();
                print!("=> ");
                stdout().flush().unwrap();
                if (stdin().read_line(&mut query).expect("Can't read from stdin")) == 0 {
                    /* Exit on EOF */
                    return Ok(());
                }
            }
        } else if query == ":" || query == "："  || query == "_" || query == "＿" {
                return Ok(());
        }

        let mut lines_output = 0;
        output.clear();

        if query.starts_with(':') || query.starts_with('：') { /* Kanji search */
            /* if search_by_radical failed, then something is very wrong */
            if search_by_radical(&mut query).is_none() {
                eprintln!("Couldn't parse input");
            }

        } else if query.starts_with('_') || query.starts_with('＿') { /* Sentence search */
            let bytes = query.chars().next().unwrap().len_utf8();

            /* Do API request */
            let body: Value = if query.chars().nth(1).unwrap().len_utf8() == 1 { /* Check if the query is jpn->eng or eng->jpn */
                ureq::get(&format!(TATOEBA_URL_ENG_QUERY!(), &query[bytes..]))
                    .call()?.into_json()?
            } else {
                ureq::get(&format!(TATOEBA_URL_JPN_QUERY!(), &query[bytes..]))
                    .call()?.into_json()?
            };

            if let Some(r) = sentence_search(&options, body, &mut output) {
                lines_output += r;
            } else {
                eprintln!("error: invalid json returned");
                return Ok(());
            }

        } else { /* Word search */
            // Do API request
            let body: Value = ureq::get(&format!(JISHO_URL!(), query))
                .call()?
                .into_json()?;

            if let Some(r) = word_search(&options, body, &query, &mut output) {
                lines_output += r;
            } else {
                eprintln!("Error: invalid json returned");
                return Ok(());
            }

        }
        if lines_output >= term_size - 1 && term_size != 0 {
            /* Output is a different process that is not a tty (i.e. less), but we want to keep colour */
            env::set_var("CLICOLOR_FORCE", "1");
            pipe_to_less(&output);
        } else {
            print!("{}", output);
        }
        if !options.interactive && !options.query.trim().is_empty() {
            break;
        }

        query.clear();
    }
    Ok(())
}

fn parse_args() -> util::Options {
    let mut options = util::Options::default();
    let mut query_vec: Vec<String> = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Use jisho.org from the cli. \
                            Searching for kanji by radicals is also available if the radkfile file is \
                            installed in \"~/.local/share\" (linux) or \"~\\AppData\\Local\\\" (windows). \
                            Additionally, searching for sentences in tatoeba is also possible.");
        ap.add_option(
            &["-V", "--version"],
            Print(env!("CARGO_PKG_VERSION").to_string()),
            "Show version",
        );
        ap.refer(&mut options.limit).add_option(
            &["-n", "--limit"],
            Store,
            "Limit the amount of results",
        );
        ap.refer(&mut query_vec)
            .add_argument("Query", List, "Search terms using jisho.org;
                          Prepend it with ':' to search a kanji by radicals instead \
                          and ':*' to search a radical by strokes (e.g. ':口*'); \
                          You can also use '_' to see example sentences from tatoeba.");

        ap.refer(&mut options.interactive).add_option(
            &["-i", "--interactive"],
            StoreTrue,
            "Don't exit after running a query",
        );

        ap.parse_args_or_exit();
    }

    options.query = query_vec.join(" ");
    options
}

fn pipe_to_less(output: &String) {

    let command = Command::new("less")
                    .arg("-R")
                    .stdin(Stdio::piped())
                    .spawn();

    match command {
        Ok(mut process) => {
            if let Err(e) = process.stdin.as_ref().unwrap().write_all(output.as_bytes()) {
                panic!("couldn't pipe to less: {}", e);
            }

            /* We don't care about the return value, only whether wait failed or not */
            if process.wait().is_err() {
                panic!("wait() was called on non-existent child process\
                 - this should not be possible");
            }
        }

        /* less not found in PATH; print normally */
        Err(_e) => print!("{}", output)
    };
}

/* OS specific part of the program */
#[cfg(unix)]
fn terminal_size() -> Result<usize, i16> {
    use libc::{ioctl, STDOUT_FILENO, TIOCGWINSZ, winsize};

    unsafe {
        let mut size = winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size as *mut _) == 0 {
            Ok(size.ws_row as usize)
        } else {
            Err(-1)
        }
    }
}

#[cfg(windows)]
fn terminal_size() -> Result<usize, i16> {
    use windows_sys::Win32::System::Console::*;
    use colored::control;

    if let Err(e) = control::set_virtual_terminal(true) {
        panic!("Could not set terminal as virtual: {:?}", e);
    }

    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE) as windows_sys::Win32::Foundation::HANDLE;

        let mut window = CONSOLE_SCREEN_BUFFER_INFO {
            dwSize: COORD { X: 0, Y: 0},
            dwCursorPosition: COORD { X: 0, Y: 0},
            wAttributes: 0,
            dwMaximumWindowSize: COORD {X: 0, Y: 0},
            srWindow: SMALL_RECT {
                Top: 0,
                Bottom: 0,
                Left: 0,
                Right: 0
            }
        };
        if GetConsoleScreenBufferInfo(handle, &mut window) == 0 {
            Err(0)
        } else {
            Ok((window.srWindow.Bottom - window.srWindow.Top) as usize)
        }
    }
}
