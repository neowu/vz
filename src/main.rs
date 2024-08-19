use std::io;

use anyhow::Context;
use anyhow::Result;
use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;
use clap_complete::dynamic::CompleteArgs;
use clap_complete::generate;
use clap_complete::Shell;
use command::create::Create;
use command::install::Install;
use command::ipsw::Ipsw;
use command::list::List;
use command::resize::Resize;
use command::run::Run;
use command::stop::Stop;

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
    Completion,
    #[command(hide = true)]
    Complete(CompleteArgs),
}

fn main() -> Result<()> {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();
    let cli = Cli::parse();
    match cli.command {
        Command::List(command) => command.execute(),
        Command::Create(command) => command.execute(),
        Command::Run(command) => command.execute(),
        Command::Stop(command) => command.execute(),
        Command::Ipsw(command) => command.execute(),
        Command::Resize(command) => command.execute(),
        Command::Install(command) => command.execute(),
        Command::Completion => {
            const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
            let shell = Shell::from_env().context("unknown shell")?;
            generate(shell, &mut Cli::command(), CARGO_PKG_NAME, &mut io::stdout());
            // only support dynmaic vm name completion for fish
            // clap dynamic completion is incomplete, better have shell native file completion
            if matches!(shell, Shell::Fish) {
                for subcommand in ["run", "stop", "install"] {
                    println!(
                        r#"complete -c {CARGO_PKG_NAME} -x -n "__fish_seen_subcommand_from {subcommand}" -a "({CARGO_PKG_NAME} complete fish -- (commandline --current-process --tokenize --cut-at-cursor) (commandline --current-token))""#
                    );
                }
            }
            Ok(())
        }
        Command::Complete(command) => {
            command.complete(&mut Cli::command());
            Ok(())
        }
    }
}
