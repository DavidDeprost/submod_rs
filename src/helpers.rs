use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use regex::Regex;
use failure::Error;


pub fn get_paths<'a>(input: &'a str, seconds: f64, partial: bool,
        output_opt: Option<&str>, convert_opt: Option<&str>, copy_opt: Option<()>)
    -> Result<(&'a Path, PathBuf, Option<PathBuf>), Error>
{
    // Create full path for inputfile:
    let input_path = Path::new(input); // creates Path that references input!

    if let Some(file) = output_opt {
        let output_path = Path::new(file).to_owned();
        return Ok( (input_path, output_path, None) );
    }

    // Find parent: path without filename
    // => parent will be empty if the path consists of the filename alone
    let parent = input_path.parent()
        .ok_or(format_err!("Invalid value for '\u{001b}[33m<INPUT>\u{001b}[0m': incorrect path"))?;

    // Create output file name without path:
    let mut output_file = input_path.file_name().and_then(OsStr::to_str)
        .ok_or(format_err!("Invalid value for '\u{001b}[33m<INPUT>\u{001b}[0m': invalid file name"))?
        .to_owned();
    // Change extension if necessary:
    if let Some(to_ext) = convert_opt {
        output_file = output_file.rsplitn(2, '.') // split in 2 on '.' starting from the end
            .nth(1) // take out second &str from iterator: file name without extension
            .unwrap() // is safe because extension is guaranteed by is_srt_or_vtt validator
            .to_owned() + "." + to_ext; // add new extension and return as String.
    }
    // Create smart output name, different from input:
    output_file = smart_name(&output_file, seconds, partial)?;

    // Create full path for output file:
    let output_path = parent.join(output_file); // creates owned PathBuf!

    let mut path_opt = None;
    if let Some(_) = copy_opt {
        let original = input_path.file_stem()
            .and_then(OsStr::to_str).unwrap()
            .to_owned() + "__[Original]." +
            input_path.extension()
            .and_then(OsStr::to_str).unwrap();
        let original_path = parent.join(original); // creates owned PathBuf!
        path_opt = Some(original_path);
    }

    return Ok( (input_path, output_path, path_opt) );
}

/// This functions smartly formats the default output file name,
/// such that output files that are reused as input still receive a sane name,
/// without any redundant extra suffixes from repeated calls.
fn smart_name(filename: &str, seconds: f64, partial: bool) -> Result<String, Error> {
    // Regex to check if the inputfile was generated by submod:
    let tag = Regex::new(r"__\[[+-]\d+\.\d+_Sec[+-]\]")?;
    let processed: bool = tag.is_match(filename);
    let len = filename.len();
    let extension = &filename[len-4..];
    let stem: &str;
    let mut incr: f64;
    // '-' indicates that only part of the file was modified:
    let partial = if partial { "-" } else { "+" };

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
        stem = &filename[..tag_start];
    } else {
        incr = seconds;
        stem = &filename[..len-4];
    }

    let output = if incr >= 0.0 {
        format!("{}__[+{:.2}_Sec{}]{}", stem, incr, partial, extension)
    } else {
        format!("{}__[{:.2}_Sec{}]{}", stem, incr, partial, extension)
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
    // Ideally, we should be able to return the f64 in Ok variant,
    // but this most likely requires more advanced `dyn` or `impl` returns
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

pub fn report_success(deleted_subs: i32, output_path: &Path, overwrite: bool, copy_opt: Option<PathBuf>) {
    println!("\u{001b}[32;1mSuccess.\u{001b}[0m");

    if deleted_subs > 0 {
        if deleted_subs == 1 {
            println!("    \u{001b}[41;1m ! \u{001b}[0m   One subtitle was deleted at the beginning of the file.");
        } else {
            println!("    \u{001b}[41;1m ! \u{001b}[0m   {} subtitles were deleted at the beginning of the file.",
                deleted_subs);
        }
    }

    if let Some(copy) = copy_opt {
        println!(" The input file was renamed to {}", copy.display());
    } else if overwrite {
        println!(" The input file was overwritten.");
    }
    println!(" Output: \u{001b}[1m \u{001b}[48;5;238m {} \u{001b}[0m", output_path.display());
}