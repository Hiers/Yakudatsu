use crate::util::*;

use serde_json::Value;
use colored::*;

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
            let translations = value_to_arr(translations?).get(0);

            translations?
        });


        for translation in translations.iter() {
            let index_str = format!("{}.", i).bright_black();
            
            /* Prefer to keep japanese sentences on top */
            if entry.get("lang")? == "eng" {
                *output += &format!("{} {}\n   {}\n\n", index_str, value_to_str(translation.get("text")?), value_to_str(entry.get("text")?));
            } else {
                *output += &format!("{} {}\n   {}\n\n", index_str, value_to_str(entry.get("text")?), value_to_str(translation.get("text")?));
            }

            i += 1;
            lines += 3;
        }

    }
    Some(lines)
}
