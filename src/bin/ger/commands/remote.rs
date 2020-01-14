use prelude::*;
use std::io::Write;

mod prelude {
    pub use crate::config::{CliConfig, Verbosity};
    pub use clap::{App, Arg, ArgMatches, SubCommand};
}

pub fn cli() -> App<'static, 'static> {
    SubCommand::with_name("remote")
        .about("Manage gerrit remote servers.")
        .template("{about}\n\n{usage}\n\n{all-args}")
        .subcommands(vec![add::cli()])
}

pub fn exec(config: &mut CliConfig, args: Option<&ArgMatches>) -> Result<(), failure::Error> {
    let args = args.unwrap();
    match args.subcommand() {
        ("", _) => show(config, Verbosity::Normal),
        _ => Ok(()),
    }
}

pub fn show(config: &mut CliConfig, verbose: Verbosity) -> Result<(), failure::Error> {
    let mut name_maxlen: usize = 0;
    // compute format variables
    for remote in config.user_cfg.remotes.iter() {
        if remote.0.len() > name_maxlen {
            name_maxlen = remote.0.len();
        }
    }
    // print remotes table
    for remote in config.user_cfg.remotes.iter() {
        let mut stdout = config.stdout.lock();
        if verbose.ge(&Verbosity::Verbose) {
            writeln!(
                stdout,
                "{0:1$} - {2} [{3}]",
                remote.0,
                name_maxlen,
                remote.1.url,
                remote.1.port.unwrap_or(8080)
            )?;
        } else {
            writeln!(stdout, "{0:1$}", remote.0, name_maxlen)?;
        }
    }
    Ok(())
}

/**************************************************************************************************/
mod add {
    use super::prelude::*;

    pub fn cli() -> App<'static, 'static> {
        SubCommand::with_name("add")
            .about("Add a new remote")
            .template("{about}\n\n{usage}\n\n{all-args}")
            .arg(
                Arg::with_name("name")
                    .value_name("NAME")
                    .required(true)
                    .help("Remote unique name"),
            )
            .arg(Arg::with_name("url"))
    }

    pub fn exec(_config: &mut CliConfig, _args: Option<&ArgMatches>) -> Result<(), failure::Error> {
        println!("Command: remote -> add");
        Ok(())
    }
}
