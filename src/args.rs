use clap::{App, Arg, ArgMatches};

pub fn parse_args() -> ArgMatches {
    App::new("Spotify Recorder")
        .version("0.1")
        .about("Record and tag music from spotify")
        .subcommand(
            App::new("load").arg(
                Arg::with_name("session_dir")
                    .index(1)
                    .about("Sets the directory the session is stored in")
                    .required(true),
            ),
        )
        .subcommand(
            App::new("record").arg(
                Arg::with_name("session_dir")
                    .index(1)
                    .about("Sets the directory to store the session in")
                    .required(true),
            ),
        )
        .arg(
            Arg::with_name("v")
                .multiple(true)
                .about("Sets the level of verbosity"),
        )
        .get_matches()
}
