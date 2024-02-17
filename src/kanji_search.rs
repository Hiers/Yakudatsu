use std::{
    io::{stdin, stdout, Write},
    collections::HashSet,
};

use kradical_parsing::radk;

pub fn search_by_radical(query: &mut String, radk_list: &[radk::Membership]) -> Option<()> {
    let mut result: HashSet<_> = HashSet::new();
    let mut aux: HashSet<_> = HashSet::new();

    if !radk_list.is_empty() {
        result.clear();

        /* First iteration: get the baseline for the results */
        let mut rad = query.chars().nth(1).unwrap();
        if rad == '*' || rad == '＊' {
            /* if search_by_strokes failed, then something is very wrong */
            rad = search_by_strokes(query, radk_list, 1)?;
        }

        for k in radk_list.iter() {
            if k.radical.glyph.contains(rad) {
                for input in &k.kanji {
                    result.insert(input);
                }
                break;
            }
        }

        /* Iterate until you've exhausted user input: refine the baseline to get final output */
        for (i, mut rad) in query.clone().chars().skip(2).enumerate() {
            if rad == '*' || rad == '＊' {
                /* if search_by_strokes failed, then something is very wrong */
                rad = search_by_strokes(query, radk_list, i+2)?;
            }

            for k in radk_list.iter() {
                if k.radical.glyph.contains(rad) {
                    for input in &k.kanji {
                        aux.insert(input);
                    }
                    result = &result & &aux;
                    aux.clear();
                    break;
                }
            }
        }
        for r in result {
            print!("{r} ");
        }
        println!();
    } else {
        eprintln!("Error while reading radkfile\nIf you don't have the radkfile, download it from\n\
        https://www.edrdg.org/krad/kradinf.html and place it in \"~/.local/share/\" on Linux or \"~\\AppData\\Local\\\" on Windows.\n\
        This file is needed to search radicals by strokes.");
    }
    Some(())
}

fn search_by_strokes(query: &mut String, radk_list: &[radk::Membership], n: usize) -> Option<char> {

    let mut strokes = String::new();
    let mut radicals: Vec<char> = Vec::new();
    let rad;
    loop{
        print!("How many strokes does your radical have? ");
        stdout().flush().ok()?;
        strokes.clear();
        if stdin().read_line(&mut strokes).ok()? == 0{
            std::process::exit(0);
        }

        match strokes.trim().parse::<u8>() {
            Ok(strk) => {
                let mut i = 1;
                for k in radk_list.iter() {
                    if k.radical.strokes == strk {
                        print!("{}{} ", i, k.radical.glyph);
                        radicals.push(k.radical.glyph.chars().next()?);
                        i += 1;
                    } else if k.radical.strokes > strk {
                        println!();
                        break;
                    }
                }
                loop {
                    print!("Choose the radical to use for your search: ");
                    stdout().flush().ok()?;
                    strokes.clear();
                    if stdin().read_line(&mut strokes).ok()? == 0{
                        std::process::exit(0);
                    }

                    match strokes.trim().parse::<usize>() {
                        Ok(strk) => {
                            if strk < 1 || strk > i-1 {
                                eprintln!("Couldn't parse input: number not in range");
                            } else {
                                rad = radicals.get(strk-1)?;
                                /* UTF-8 is not fun */
                                let char_and_index = query.char_indices().nth(n)?;
                                query.replace_range(char_and_index.0..
                                                    char_and_index.0 +
                                                    char_and_index.1.len_utf8(),
                                                    rad.to_string().as_str());
                                println!("\x1b[90m{}\x1b[m", query);
                                return Some(*rad);
                            }
                        },
                        Err(e) => { eprintln!("{e}"); }
                    }
                }
            },
            Err(e) => { eprintln!("{e}") }
        }
    }
}

