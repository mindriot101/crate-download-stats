use clap::{Arg, App, ArgMatches};

pub fn cmdline_args<'a>() -> ArgMatches<'a> {
    let matches = App::new("crate-downloads")
        .arg(Arg::with_name("crate")
             .long("crate")
             .short("c")
             .required(true)
             .multiple(true)
             .takes_value(true))
        .get_matches();
    matches
}
