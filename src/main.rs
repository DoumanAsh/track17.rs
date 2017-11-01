#[macro_use] extern crate hyper;
extern crate tokio_core;
extern crate futures;

#[macro_use]
extern crate serde_derive;

use futures::{
    Stream,
    Future
};

#[macro_use]
mod utils;
mod api;
mod cli;

fn run() -> Result<i32, String> {
    let args = cli::Args::new()?;

    let mut tokio_core = tokio_core::reactor::Core::new().map_err(error_formatter!("Unable to create tokio's event loop."))?;
    let client = api::Client::new(tokio_core.handle());

    let track = client.track(&args.to_track)
                      .map_err(error_formatter!("Cannot send track!"))
                      .and_then(handle_bad_hyper_response!("Cannot track package."))
                      .and_then(read_hyper_resp_body!("Cannot read track's response"))
                      .map(api::Client::parse_track_response);

    let response = tokio_core.run(track)?.map_err(error_formatter!("Invalid 17track response."))?;

    if let Err(error) = response.result {
        return Err(format!("Failed to retrieve information. Error: {}", error));
    }

    for data in response.data {
        println!("==={}===", data.num);
        match data.track {
            Some(track) => if args.detailed {
                println!("{}", track)
            }
            else {
                println!("{}", track.last)
            },
            None => println!("Temporarily unavailable. Try again in {}s", data.delay)
        }
        println!("=========");
    }

    Ok(0)
}

fn main() {
    use std::process::exit;

    let code: i32 = match run() {
        Ok(res) => res,
        Err(error) => {
            eprintln!("{}", error);
            1
        }
    };

    exit(code);
}
