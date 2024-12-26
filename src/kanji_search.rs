use std::{
    io::{stdin, stdout, Write},
    collections::HashSet,
    fs
};

use kradical_parsing::radk;

pub fn search_by_radical(query: &mut String, radk_list: &[radk::Membership], stroke_info: &[String]) -> Option<()> {
    let mut result: HashSet<_> = HashSet::new();
    let mut aux: HashSet<_> = HashSet::new();

    if !radk_list.is_empty() && !stroke_info.is_empty() {
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

        /* Hash sets are unordered; Will now order the results */
        let mut vec: Vec<Vec<String>> = Vec::with_capacity(30); /* The kanji we care about will have at most 30 strokes */
        for _i in 0..29 {
            vec.push(Vec::new());
        }

        /* 
         * A vector of vectors is useful here to store kanji by number of strokes
         * First vector's index will indicate the number of strokes (minus 1 because it starts at 0)
         * Second vector will hold all of the kanji that is written in that number of strokes
         */
        for r in &result {
            for (i, s) in stroke_info.iter().enumerate() {
                if s.contains(r.as_str()) {
                    vec[i].push(r.to_string());
                    break;
                }
            }
        }

        for (i, v) in vec.iter().enumerate() {
            if !v.is_empty() {
                let ii = i + 1;
                print!("\x1b[90m{:02} -\x1b[m", ii);
                for l in v {
                    print!(" {l}");
                }
                println!();
            }
        }
    } else if radk_list.is_empty() {
        eprintln!("Error while reading radkfile\nIf you don't have the radkfile, download it from\n\
        https://www.edrdg.org/krad/kradinf.html and place it in \"~/.local/share/\" on Linux or \"~\\AppData\\Local\\\" on Windows.\n\
        This file is needed to search radicals by strokes.");
    } else {
        eprintln!("File \"/usr/local/share/ykdt/kanji_strokes\" is missing!");
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

pub fn get_stroke_info() -> Result<Vec<String>, std::io::Error> {
    let file: String;
    #[cfg(unix)]
    {
        file = fs::read_to_string("/usr/local/share/ykdt/kanji_strokes")?;
    }
    #[cfg(windows)]
    {
        file = fs::read_to_string("C:\\Program Files\\ykdt\\kanji_strokes")?;
    }
    let stroke_info: Vec<String> = Vec::from_iter(file.split('\n').map(|s| s.to_owned()));
    Ok(stroke_info)
}
