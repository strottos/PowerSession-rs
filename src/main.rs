#[allow(non_snake_case)]
extern crate clap;
extern crate core;

mod commands;
mod terminal;

use commands::{Asciinema, Auth, Play};
use commands::{Record, Upload};

use clap::{crate_version, Arg, Command};
use tracing::trace;
use tracing_subscriber::{
    fmt::writer::BoxMakeWriter, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry,
};

fn setup_logger(filter: Option<&str>) {
    let filter = filter.map_or(EnvFilter::default(), EnvFilter::new);

    let writer = BoxMakeWriter::new(std::io::stderr);

    let fmt_layer = tracing_subscriber::fmt::layer().with_writer(writer);

    Registry::default().with(filter).with(fmt_layer).init();
}

fn main() {
    let app = Command::new("PowerSession")
        .version(crate_version!())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("rec")
                .about("Record and save a session")
                .arg(
                    Arg::new("file")
                        .help("The filename to save the record")
                        .num_args(1)
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::new("command")
                        .help("The command to record, defaults to $SHELL")
                        .num_args(1)
                        .short('c')
                        .long("command"),
                )
                .arg(
                    Arg::new("force")
                        .help("Overwrite if session already exists")
                        .num_args(0)
                        .short('f')
                        .long("force"),
                ),
        )
        .subcommand(
            Command::new("play").about("Play a recorded session").arg(
                Arg::new("file")
                    .help("The record session")
                    .index(1)
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("auth").about("Authentication with api server (default is asciinema.org)"),
        )
        .subcommand(
            Command::new("upload")
                .about("Upload a session to api server")
                .arg(
                    Arg::new("file")
                        .help("The file to be uploaded")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("server")
                .about("The url of asciinema server")
                .arg(
                    Arg::new("url")
                        .help("The url of asciinema server. default is https://asciinema.org")
                        .index(1)
                        .required(true),
                ),
        )
        .arg(
            Arg::new("log-level")
                .help("can be one of [error|warn|info|debug|trace]")
                .short('l')
                .long("log-level")
                .default_value("error")
                .default_missing_value("trace")
                .global(true)
                .num_args(0..=1),
        );

    let m = app.get_matches();

    setup_logger(m.get_one::<String>("log-level").map(|s| s.as_str()));

    let span = tracing::span!(tracing::Level::TRACE, "root");
    let _enter = span.enter();

    trace!("PowerSession running");

    match m.subcommand() {
        Some(("play", play_matches)) => {
            let play = Play::new(play_matches.get_one::<String>("file").unwrap().to_owned());
            play.execute();
        }
        Some(("rec", rec_matches)) => {
            let mut record = Record::new(
                rec_matches.get_one::<String>("file").unwrap().to_owned(),
                None,
                rec_matches.get_one::<String>("command").map(Into::into),
                rec_matches.get_flag("force"),
            );
            record.execute();
        }
        Some(("auth", _)) => {
            let api_service = Asciinema::new();
            let auth = Auth::new(Box::new(api_service));
            auth.execute();
        }
        Some(("upload", upload_matches)) => {
            let api_service = Asciinema::new();
            let upload = Upload::new(
                Box::new(api_service),
                upload_matches.get_one::<String>("file").unwrap().to_owned(),
            );
            upload.execute();
        }
        Some(("server", new_server)) => {
            let url = &new_server.get_one::<String>("url").unwrap().to_owned();
            let is_url = reqwest::Url::parse(url);
            match is_url {
                Ok(_) => Asciinema::change_server(url.to_string()),
                Err(_) => println!("Error: not a correct URL - e.g: https://asciinema.org"),
            }
        }
        _ => unreachable!(),
    }
}
