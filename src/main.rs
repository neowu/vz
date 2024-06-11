use clap::Parser;
use clap::Subcommand;
use command::create::Create;
use command::generate_zsh_completion::GenerateZshCompletion;
use command::install::Install;
use command::ipsw::Ipsw;
use command::list::List;
use command::resize::Resize;
use command::run::Run;
use command::stop::Stop;
use util::exception::Exception;

mod command;
mod config;
mod util;
mod vm;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "manage virtual machines")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
#[command(arg_required_else_help(true))]
pub enum Command {
    #[command(name = "ls", about = "list vm status")]
    List(List),
    #[command(about = "create vm")]
    Create(Create),
    #[command(about = "run vm")]
    Run(Run),
    #[command(about = "stop vm")]
    Stop(Stop),
    #[command(
        about = "get macOS restore image ipsw url",
        long_about = "get macOS restore image ipsw url, download ipsw file manually, then use in create command with --ipsw"
    )]
    Ipsw(Ipsw),
    #[command(about = "increase disk image size")]
    Resize(Resize),
    #[command(about = "install macOS")]
    Install(Install),
    #[command(about = "generate zsh completion")]
    GenerateZshCompletion(GenerateZshCompletion),
}

fn main() -> Result<(), Exception> {
    tracing_subscriber::fmt().with_thread_ids(true).init();
    let cli = Cli::parse();
    match cli.command {
        Some(Command::List(command)) => command.execute(),
        Some(Command::Create(command)) => command.execute(),
        Some(Command::Run(command)) => command.execute(),
        Some(Command::Stop(command)) => command.execute(),
        Some(Command::Ipsw(command)) => command.execute(),
        Some(Command::Resize(command)) => command.execute(),
        Some(Command::Install(command)) => command.execute(),
        Some(Command::GenerateZshCompletion(command)) => command.execute(),
        None => panic!("not implemented"),
    }
}
