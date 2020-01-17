use prelude::*;

mod prelude {
    pub use crate::config::Remote;
    pub use crate::config::{CliConfig, Verbosity};
    pub use crate::util;
    pub use clap::{App, Arg, ArgMatches, SubCommand};
}

pub fn cli() -> App<'static, 'static> {
    SubCommand::with_name("remote")
        .about("Manage gerrit remote servers.")
        .template("{about}\n\nUSAGE:\n    {usage}\n\n{all-args}")
        .subcommands(vec![add::cli(), show::cli(), remove::cli()])
}

pub fn exec(config: &mut CliConfig, args: Option<&ArgMatches>) -> Result<(), failure::Error> {
    let args = args.unwrap();
    match args.subcommand() {
        ("add", subargs) => add::exec(config, subargs),
        ("show", subargs) => show::exec(config, subargs),
        ("", _) => show::show_table(config, args.occurrences_of("verbose").into()),
        ("remove", subargs) => remove::exec(config, subargs),
        _ => Ok(()),
    }
}

/**************************************************************************************************/
mod show {
    use super::prelude::*;
    use std::io::Write;

    pub fn cli() -> App<'static, 'static> {
        SubCommand::with_name("show")
            .about("Show information about remote.")
            .template("{about}\n\nUSAGE:\n    {usage}\n\n{all-args}")
            .arg(Arg::with_name("remote").multiple(true).help("Remote name."))
    }

    pub fn exec(config: &mut CliConfig, args: Option<&ArgMatches>) -> Result<(), failure::Error> {
        let args = args.unwrap();
        let verbose: Verbosity = args.occurrences_of("verbose").into();
        match args.values_of("remote") {
            Some(remotes) => show_remote(config, remotes.into_iter(), verbose),
            None => show_table(config, verbose),
        }
    }

    pub fn show_table(config: &CliConfig, verbose: Verbosity) -> Result<(), failure::Error> {
        let mut name_maxlen = 0;
        let mut url_maxlen = 0;
        // compute format variables
        for remote in config.user_cfg.settings.remotes.iter() {
            if remote.0.len() > name_maxlen {
                name_maxlen = remote.0.len();
            }
            if remote.1.url.len() > url_maxlen {
                url_maxlen = remote.1.url.len();
            }
        }
        // print remotes table
        for remote in config.user_cfg.settings.remotes.iter() {
            let mut stdout = config.stdout.lock();
            write!(stdout, "{0}", remote.0)?;
            if verbose.ge(&Verbosity::Verbose) {
                write!(
                    stdout,
                    "{0:1$} - {2} [{3}]",
                    "",
                    name_maxlen - remote.0.len(),
                    remote.1.url,
                    remote.1.port.unwrap_or(8080)
                )?;
            }
            if verbose.ge(&Verbosity::High) {
                write!(stdout, "{0:1$}", "", url_maxlen - remote.1.url.len())?;
                if let Some(username) = &remote.1.username {
                    write!(stdout, " ({})", username)?
                }
            }
            writeln!(stdout, "")?;
        }
        Ok(())
    }

    pub fn show_remote(
        config: &CliConfig, remotes: clap::Values, _verbose: Verbosity,
    ) -> Result<(), failure::Error> {
        for name in remotes {
            let mut stdout = config.stdout.lock();
            if let Some(remote) = config.user_cfg.settings.remotes.get(name.into()) {
                writeln!(stdout, "* remote: {}\n  url: {}", name, remote.url)?;
                if let Some(port) = remote.port {
                    writeln!(stdout, "  port: {}", port)?;
                }
                if let Some(username) = &remote.username {
                    writeln!(stdout, "  login: {}", username)?
                }
                writeln!(stdout, "")?;
            } else {
                return Err(failure::err_msg(format!("no such remote '{}'.", name)));
            }
        }
        Ok(())
    }
}

/**************************************************************************************************/
mod add {
    use super::prelude::*;

    pub fn cli() -> App<'static, 'static> {
        SubCommand::with_name("add")
            .about("Add a new remote.")
            .template("{about}\n\nUSAGE:\n    {usage}\n\n{all-args}")
            .setting(clap::AppSettings::DeriveDisplayOrder)
            .arg(
                Arg::with_name("name")
                    .required(true)
                    .help("Remote unique identifier."),
            )
            .arg(
                Arg::with_name("url")
                    .required(true)
                    .validator(util::validate::is_url_http_https)
                    .help("Remote URL including protocol. e.g. 'https://mygerrit.com'."),
            )
            .arg(
                Arg::with_name("port")
                    .takes_value(true)
                    .validator(util::validate::is_u16_range)
                    .help("Port to use on connection with server."),
            )
            .arg(
                Arg::with_name("username")
                    .long("username")
                    .short("u")
                    .takes_value(true)
                    .value_name("ID")
                    .help("Username for login."),
            )
            .arg(
                Arg::with_name("password")
                    .long("password")
                    .short("p")
                    .takes_value(true)
                    .value_name("STRING")
                    .min_values(0)
                    .help(
                        "HTTP password. Can be generated in gerrit user settings menu.\n\
                         Pass only the flag without value to be prompted for (recommended).\n\
                         Note: this password is saved in plain text in the configuration file.",
                    ),
            )
    }

    pub fn exec(config: &mut CliConfig, args: Option<&ArgMatches>) -> Result<(), failure::Error> {
        let args = args.unwrap();

        let name = args.value_of("name").unwrap();
        let url = args.value_of("url").unwrap();
        let port = match args.value_of("port") {
            Some(p) => Some(p.parse::<u16>().unwrap()),
            None => None,
        };
        let username = match args.value_of("username") {
            Some(u) => Some(u.to_owned()),
            None => None,
        };
        let http_password = match args.value_of("password") {
            Some(p) => Some(p.to_owned()),
            None => None,
        };

        if config.user_cfg.settings.remotes.contains_key(name) {
            return Err(failure::err_msg(format!(
                "remote '{}' already exists.",
                name
            )));
        }

        config.user_cfg.settings.remotes.insert(
            name.into(),
            Remote {
                url: url.to_owned(),
                port,
                username,
                http_password,
            },
        );

        config.user_cfg.store()?;
        Ok(())
    }
}

/**************************************************************************************************/
mod remove {
    use super::prelude::*;
    use std::io::Write;

    pub fn cli() -> App<'static, 'static> {
        SubCommand::with_name("remove")
            .visible_alias("rm")
            .about("Remove a remote from config.")
            .template("{about}\n\nUSAGE:\n    {usage}\n\n{all-args}")
            .arg(
                Arg::with_name("remote")
                    .required(true)
                    .multiple(true)
                    .help("Remote name."),
            )
    }

    pub fn exec(config: &mut CliConfig, args: Option<&ArgMatches>) -> Result<(), failure::Error> {
        let args = args.unwrap();
        let remotes = args.values_of("remote").unwrap();

        for remote in remotes.into_iter() {
            let mut stdout = config.stdout.lock();
            match config.user_cfg.settings.remotes.remove(remote) {
                Some(_) => writeln!(stdout, "removed: {}", remote)?,
                None => writeln!(stdout, "fatal: no such remote: {}", remote)?,
            };
        }

        config.user_cfg.store()?;
        Ok(())
    }
}
