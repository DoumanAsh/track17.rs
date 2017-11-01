extern crate clap;

use self::clap::{App, Arg};

const NAME: &'static str = env!("CARGO_PKG_NAME");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const ABOUT: &'static str = "
Simple parcel tracking tool.

It uses http://www.17track.net API.
";

#[inline(always)]
///Shortcut to create CLI argument
pub fn arg(name: &str) -> Arg {
    Arg::with_name(name)
}

#[inline(always)]
///Shortcut to create CLI option/flag
pub fn flag(name: &str) -> Arg {
    arg(name).long(name)
}

pub fn parser() -> App<'static, 'static> {
    App::new(NAME).about(ABOUT)
                  .author(AUTHOR)
                  .version(VERSION)
                  .arg(arg("track-id").required(true).help("Tracking ID to inquiry about."))
                  .arg(flag("detailed").help("Whether to print all events for each tracking"))
}

#[derive(Debug)]
pub struct Args {
    ///Path to config file
    pub detailed: bool,
    pub to_track: String
}

impl Args {
    pub fn new() -> Result<Self, String> {
        let matches = parser().get_matches();

        Ok(Args {
            detailed: matches.is_present("detailed"),
            to_track: matches.value_of("track-id").unwrap().to_string()
        })
    }
}
