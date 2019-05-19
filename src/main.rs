extern crate regex;
extern crate clap;
use clap::{App, Arg, AppSettings};
#[macro_use]
extern crate failure;

mod convert;
mod helpers;


fn main() {
    let app = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .version_short("v")
        // AllowLeadingHyphen allows passing negative seconds:
        .setting(AppSettings::AllowLeadingHyphen)
        .about("Modify the time encoding of movie subtitles.\n(UTF-8 encoded .srt or .vtt files)")
        .arg(Arg::with_name("INPUT")
            .help("(Path to) .srt or .vtt subtitle file to convert")
            .required(true)
            .index(1)
            .validator(helpers::is_srt_or_vtt))
        .arg(Arg::with_name("SECONDS")
            .help("Seconds by which to add or subtract the time encoding")
            .required(true)
            .index(2)
            .validator(helpers::is_float))
        .arg(Arg::with_name("convert")
            .help("Converts to other subtitle format")
            .short("c")
            .long("convert")
            .value_name("extension")
            .takes_value(true)
            .possible_values(&["srt", "vtt"]));
    let matches = app.get_matches();

    // Calling .unwrap() on "INPUT" and "SECONDS" is safe because both are required arguments.
    // (If they weren't required we could use an 'if let' to conditionally get the value)
    let input = matches.value_of("INPUT").unwrap();
    let seconds: f64 = matches.value_of("SECONDS").unwrap().parse().unwrap();
    // The second unwrap call on parse() is also safe because we've already
    // validated SECONDS as a float during argument parsing (using is_float())

    let (input_path, output_path) = match helpers::get_paths(input, seconds, matches.value_of("convert")) {
        Ok(paths) => paths,
        Err(error) => {
            helpers::report_error(error);
            return;
        }
    };

    let deleted_subs = match convert::convert(&input_path, &output_path, seconds) {
        Ok(num) => num,
        Err(error) => {
            helpers::report_error(error);
            return;
        }
    };

    helpers::report_success(deleted_subs, &output_path);
}
