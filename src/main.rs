use clap::Parser;
use clap::Subcommand;
use command::complete::Complete;
use command::completion::Completion;
use command::create::Create;
use command::install::Install;
use command::ipsw::Ipsw;
use command::list::List;
use command::resize::Resize;
use command::run::Run;
use command::stop::Stop;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod command;
mod config;
mod util;
mod vm;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "manage virtual machines")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
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
    #[command(about = "generate shell completion")]
    Completion(Completion),
    #[command(name = "_complete", hide = true)]
    Complete(Complete),
}

fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_line_number(true)
                .with_thread_ids(true)
                .with_filter(LevelFilter::INFO),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::List(command) => command.execute(),
        Command::Create(command) => command.execute(),
        Command::Run(command) => command.execute(),
        Command::Stop(command) => command.execute(),
        Command::Ipsw(command) => command.execute(),
        Command::Resize(command) => command.execute(),
        Command::Install(command) => command.execute(),
        Command::Complete(command) => command.execute(),
        Command::Completion(command) => command.execute(),
    }
}
