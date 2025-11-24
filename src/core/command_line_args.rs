use clap::Parser;

#[derive(Clone, Parser)]
#[command(version, about, long_about = None)]
pub struct CommandLineArgs {
    #[arg(short, long, value_parser = ["DEV", "DEBUG", "PRODUCTION", "SPEEDTEST"])]
    pub opmode: Option<String>,

    #[arg(long, help = "Reset the admin password when server starts")]
    pub reset_admin_password: bool,
}


pub fn get_command_line_args() -> CommandLineArgs {
     // Parse command line args
    CommandLineArgs::parse()
}
