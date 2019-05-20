extern crate regex;
use regex::Regex;

use std::path::Path;
use std::path::PathBuf;
use std::ffi::OsStr;

use failure::Error;


pub fn get_paths(input: &str, seconds: f64, convert: Option<&str>)
    -> Result<(PathBuf, PathBuf), Error>
{
    // Create full path for inputfile:
    let input_path = Path::new(input);

    // Find parent: path without filename
    // => parent will be empty if the path consists of the filename alone
    let parent = input_path.parent()
        .ok_or(format_err!("Invalid value for '\u{001b}[33m<INPUT>\u{001b}[0m': incorrect path"))?;

    // Create output file name without path:
    let mut output_file = input_path.file_name() // returns Option<&OsStr>
        .and_then(OsStr::to_str) // returns Option<&str>
        .and_then(|filename| {
            let mut filename = filename.to_owned();
            // Change extension if necessary:
            if let Some(to_ext) = convert {
                let len = filename.len();
                filename.truncate(len - 3);
                filename.push_str(to_ext);
            }
            // The closure needs to manually wrap its return value with Some:
            Some(filename) // returns Option<String>
        })
        // Transform to Result<String> and finally to String with `?`:
        .ok_or(format_err!("Invalid value for '\u{001b}[33m<INPUT>\u{001b}[0m': invalid file name"))?;
    output_file = smart_name(&output_file, seconds)?;

    // Create full path for output file:
    let output_path = Path::new(parent).join(output_file);

    return Ok( ( input_path.to_owned(), output_path.to_owned() ) );
}

/// This functions smartly formats the default output file name,
/// such that output files that are reused as input still receive a sane name,
/// without any redundant extra suffixes from repeated calls.
fn smart_name(filename: &str, seconds: f64) -> Result<String, Error> {
    // Regex to check if the inputfile was generated by submod:
    let tag = Regex::new(r"__\[[+-]\d+\.\d+_Sec\]")?;
    let processed: bool = tag.is_match(filename);
    let mut incr: f64;
    let len = filename.len();
    let extension = &filename[len-4..];
    let stem;

    if processed {
        // Extract the increment number from the filename:
        incr = Regex::new(r"[+-]\d+\.\d+")? // regex for finding incr in filename
            .captures(filename) // returns capture groups corresponding to the leftmost-first match as an Option<>
            .unwrap()
            .get(0) // take out entire match
            .unwrap()
            .as_str()  // convert to str
            .parse()?; // convert to float
        incr += seconds;

        let tag_start = tag.find(filename).unwrap().start();
        stem = filename[..tag_start].to_string();
    } else {
        incr = seconds;
        stem = filename[..len-4].to_string();
    }

    let output = if incr >= 0.0 {
        format!("{}__[+{:.2}_Sec]{}", stem, incr, extension)
    } else {
        format!("{}__[{:.2}_Sec]{}", stem, incr, extension)
    };

    return Ok(output);
}

pub fn is_srt_or_vtt(input: String) -> Result<(), String> {
    if input.ends_with(".srt") || input.ends_with(".vtt") {
        return Ok(());
    }
    Err(String::from("incorrect file extension\n\n\
        Only \u{001b}[32m.srt\u{001b}[0m or \u{001b}[32m.vtt\u{001b}[0m files are allowed."))
}

pub fn is_float(seconds: String) -> Result<(), String> {
    if let Ok(_) = seconds.parse::<f64>() {
        Ok(())
    } else {
        Err("should be a number".to_string())
    }
}

pub fn is_timing(time_string: String) -> Result<(), String> {
    let result: Result<Vec<_>, _> = time_string.rsplit(":")
        .map(|t| t.parse::<f64>())
        .collect(); // use collect() on iterator of Result<T, E>s to see if any of them failed!

    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("incorrect time formatting\n\n\
            Use ':' to separate hours, minutes and seconds, like so:\n    \
            \u{001b}[32mhh:mm:ss\u{001b}[0m to specify hours, minutes and seconds\n       \
            \u{001b}[32mmm:ss\u{001b}[0m to only specify minutes and seconds\n          \
            \u{001b}[32mss\u{001b}[0m to only specify seconds"))
    }
}

pub fn report_error(error: Error) {
    eprintln!("\u{001b}[38;5;208mError:\u{001b}[0m {}\n", error);
    println!("USAGE:\n    \
                submod [FLAGS] [OPTIONS] <INPUT> <SECONDS>\n        \
                    INPUT: (Path to) .srt or .vtt subtitle file to convert\n        \
                    SECONDS: seconds to add or subtract from time encoding\n\n\
                    For more information try \u{001b}[32m--help\u{001b}[0m");
}

pub fn report_success(deleted_subs: i32, output_path: &PathBuf) {
    println!("\u{001b}[32;1mSuccess.\u{001b}[0m");

    if deleted_subs > 0 {
        if deleted_subs == 1 {
            println!("    \u{001b}[41;1m ! \u{001b}[0m   One subtitle was deleted at the beginning of the file.");
        } else {
            println!("    \u{001b}[41;1m ! \u{001b}[0m   {} subtitles were deleted at the beginning of the file.",
                deleted_subs);
        }
    }

    println!(" Output: \u{001b}[1m \u{001b}[48;5;238m {} \u{001b}[0m", output_path.display());
}