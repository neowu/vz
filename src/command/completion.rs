use std::io;

use clap::Args;
use clap::CommandFactory;
use clap_complete::Shell;
use clap_complete::generate;

use crate::Cli;

#[derive(Args)]
pub struct Completion;

const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

impl Completion {
    pub fn execute(&self) {
        let shell = Shell::from_env().expect("unknown shell");
        generate(shell, &mut Cli::command(), CARGO_PKG_NAME, &mut io::stdout());
        // only support dynmaic vm name completion for fish
        // clap dynamic completion is incomplete, better have shell native file completion
        if matches!(shell, Shell::Fish) {
            for subcommand in ["run", "stop", "edit", "install"] {
                println!(
                    r#"complete -c {CARGO_PKG_NAME} -x -n "__fish_seen_subcommand_from {subcommand}" -a "({CARGO_PKG_NAME} _complete vm_name)""#
                );
            }
        }
    }
}
