use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Increase verbosity, repeat for more verbosity, default is 3 (info)
    #[arg(
        short = 'v',
        long,
        action = clap::ArgAction::Count,
        global = true,
        default_value_t = 3
    )]
    pub verbose: u8,
}
