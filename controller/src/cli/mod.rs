use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, author)]
pub struct Cli {
    /// The level of verbosity.
    #[clap(short, long)]
    pub(crate) verbose: Option<usize>,
}