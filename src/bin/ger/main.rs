#[macro_use]
extern crate clap;
extern crate ansi_term;
extern crate chrono;
extern crate exitfailure;
extern crate failure;
extern crate gerlib;
mod cli;

fn main() -> Result<(), exitfailure::ExitFailure> {
    let result = cli::cli(&mut std::env::args_os(), &mut std::io::stdout())?;
    Ok(result)
}
