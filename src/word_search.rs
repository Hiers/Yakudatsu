use std::fmt::Write;
use crate::util::*;

use serde_json::Value;

pub fn word_search(options: &Options, body: Value, query: &str, output: &mut String) -> Option<usize> {
    let mut lines_output = 0;

    // Try to get the data json-object
    let body = value_to_arr({
        let body = body.get("data");

        body?
    });

    /* Iterate over meanings and print them */
    for (i, entry) in body.iter().enumerate() {
        if i >= options.limit && options.limit != 0 {
            break;
        }
        if let Some(r) = print_item(query, entry, output) {
            lines_output += r;
        }

        output.push('\n');
        lines_output += 1;
    }
    output.pop();
    lines_output = lines_output.saturating_sub(1);

    Some(lines_output)
}

fn print_item(query: &str, value: &Value, output: &mut String) -> Option<usize> {
    let japanese = value_to_arr(value.get("japanese")?);
    let main_form = japanese.get(0)?;
    let mut num_of_lines = 0;

    format_form(query, main_form, output)?;
    format_result_tags(value, output);
    output.push('\n');

    /* Print senses */
    let senses = value_to_arr(value.get("senses")?);
    let mut prev_parts_of_speech = String::new();

    for (i, sense) in senses.iter().enumerate() {
        let new_part_of_speech = format_sense(sense, i+1, output, &mut prev_parts_of_speech);
        /*
         * If the current meaning of our word is a different part of speech
         * (e.g. previous meaning was 'Noun' and the current is 'Adverb'), an extra line will be
         * printed with this information
         */
        if new_part_of_speech {
            num_of_lines += 1;
        }

    }

    /* Print alternative readings and kanji usage */
    if let Some(form) = japanese.get(1) {
        num_of_lines += 2;

        output.push_str("    \x1b[94mOther forms\x1b[m\n    ");
        format_form(query, form, output)?;

        for i in 2..japanese.len() {
            output.push_str(", ");
            format_form(query, japanese.get(i)?, output)?;
        }
        output.push('\n');
    }

    /* Clear any font effect or colour */
    output.push_str("\x1b[m");

    num_of_lines += senses.len() + 1;
    Some(num_of_lines)
}

fn format_form(query: &str, form: &Value, output: &mut String) -> Option<()> {
    if let Some(reading) = form.get("reading") {

        let word = value_to_str(form.get("word").unwrap_or(reading));
        write!(output, "{}", format_args!("{}[{}]", word, value_to_str(reading))).ok()?;

    } else if let Some(word) = form.get("word") {
        write!(output, "{}", format_args!("{}", value_to_str(word))).ok()?;
    } else {
        write!(output, "{}", format_args!("{}",query)).ok()?;
    }

    Some(())
}

fn format_sense(value: &Value, index: usize, output: &mut String, prev_parts_of_speech: &mut String) -> bool {
    let english_definitons = value.get("english_definitions");
    let parts_of_speech = value.get("parts_of_speech");
    if english_definitons.is_none() {
        return false;
    }

    let english_definiton = value_to_arr(english_definitons.unwrap());

    let is_part_of_speech_new;
    if let Some(parts_of_speech) = parts_of_speech {
        let parts = value_to_arr(parts_of_speech)
            .iter()
            .map(value_to_str)
            .collect::<Vec<&str>>()
            .join(", ");

        /* Do not repeat a meaning's part of speech if it is the same as the previous meaning */
        if !parts.is_empty() && parts != *prev_parts_of_speech {
            *prev_parts_of_speech = parts.clone();
            is_part_of_speech_new = true;

            write!(output, "{}", format_args!("    \x1b[34m{}\x1b[m\n", parts)).unwrap();
        } else {
            is_part_of_speech_new = false;
        }
    } else {
        is_part_of_speech_new = false;
    }

    write!(output, "{}",
        format_args!("    \x1b[90m{}.\x1b[m {}",
        index,
        english_definiton
            .iter()
            .map(value_to_str)
            .collect::<Vec<&str>>()
            .join(", "),
    )).unwrap();

    let t = format_sense_tags(value, output);
    format_sense_info(value, output, t);
    output.push('\n');


    return is_part_of_speech_new;
}

/// Format tags from a whole meaning
fn format_result_tags(value: &Value, output: &mut String) {

    let is_common_val = value.get("is_common");
    if is_common_val.is_some() && value_to_bool(is_common_val.unwrap()) {
        output.push_str("\x1b[92m(common)\x1b[m");
    }

    if let Some(jlpt) = value.get("jlpt") {
        /*
         * The jisho API actually returns an array of all of JLTP levels for each alternative of a word
         * Since the main one is always at index 0, we take that for formatting
         */
        let jlpt = value_to_arr(jlpt);
        if !jlpt.is_empty() {
            let jlpt = value_to_str(jlpt.get(0).unwrap())
                .replace("jlpt-", "")
                .to_uppercase();
            write!(output, "{}", format_args!("\x1b[94m({})\x1b[m", jlpt)).unwrap();
        }
    }
}

/// Format tags from a single sense entry
fn format_sense_tags(value: &Value, output: &mut String) -> bool {
    if let Some(tags) = value.get("tags") {
        let tags = value_to_arr(tags);

        if let Some(tag) = tags.get(0) {
            let t = format_sense_tag(value_to_str(tag));
            output.push_str(" \x1b[90m");
            output.push_str(t);
        } else {
            return false;
        }

        for tag in tags.get(1).iter() {
            let t = format_sense_tag(value_to_str(tag));
            output.push_str(", ");
            output.push_str(t);
        }
        
        return true;
    }
    return false;
}

fn format_sense_tag(tag: &str) -> &str{
    match tag {
        "Usually written using kana alone" => "UK",
        s => s,
    }
}

fn format_sense_info(value: &Value, output: &mut String, t: bool) {
    if let Some(all_info) = value.get("info") {
        let all_info = value_to_arr(all_info);

        if let Some(info) = all_info.get(0) {
            if t {
                output.push(',');
            }
            output.push_str(" \x1b[90m");
            output.push_str(value_to_str(info));
        }

        for info in all_info.get(1).iter() {
            output.push_str(", ");
            output.push_str(value_to_str(info));
        }
    }
}
