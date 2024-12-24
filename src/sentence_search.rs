use std::fmt::Write;
use crate::util::*;

use serde_json::Value;

pub fn sentence_search(options: &Options, body: Value, output: &mut String) -> Option<usize>{

    let mut lines = 0;
    let body = value_to_arr({
        let body = body.get("results");

        body?
    });

    let mut i = 1;

    /*
     * Each entry is an english or japanese sentence and we're pairing it up
     * with the equivalent sentences in the translations array
     */
    for entry in body.iter() {
        if i >= options.limit && options.limit != 0 {
            break;
        }

        /* json nonsense */
        let translations = value_to_arr({
            let translations = entry.get("translations");
            let translations = value_to_arr(translations?).first();

            translations?
        });


        for translation in translations.iter() {

            /* Prefer to keep japanese sentences on top */
            if entry.get("lang")? == "eng" {
                write!(output, "{}", format_args!("\x1b[90m{}.\x1b[m {}\n   {}\n\n", i, value_to_str(translation.get("text")?), value_to_str(entry.get("text")?))).unwrap();
            } else {
                write!(output, "{}", format_args!("\x1b[90m{}.\x1b[m {}\n   {}\n\n", i, value_to_str(entry.get("text")?), value_to_str(translation.get("text")?))).unwrap();
            }

            i += 1;
            lines += 3;
        }

    }
    Some(lines)
}
