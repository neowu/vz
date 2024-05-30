use clap::Parser;
use clap::Subcommand;
use command::create::Create;
use command::generate_zsh_completion::GenerateZshCompletion;
use command::ipsw::Ipsw;
use command::run::Run;
use util::exception::Exception;

mod command;
mod config;
mod util;
mod vm;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Manage virtual machines")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
#[command(arg_required_else_help(true))]
pub enum Command {
    Run(Run),
    Create(Create),
    Ipsw(Ipsw),
    GenerateZshCompletion(GenerateZshCompletion),
}

#[tokio::main]
async fn main() -> Result<(), Exception> {
    tracing_subscriber::fmt().with_thread_ids(true).init();
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Run(command)) => command.execute().await,
        Some(Command::Create(command)) => command.execute().await,
        Some(Command::Ipsw(command)) => command.execute().await,
        Some(Command::GenerateZshCompletion(command)) => command.execute(),
        None => panic!("not implemented"),
    }
}
