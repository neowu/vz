use clap_complete::dynamic::CompletionCandidate;

use crate::config::vm_dir;

pub mod create;
pub mod install;
pub mod ipsw;
pub mod list;
pub mod resize;
pub mod run;
pub mod stop;

fn complete_vm_name() -> Vec<CompletionCandidate> {
    vm_dir::vm_dirs().into_iter().map(|dir| CompletionCandidate::new(dir.name())).collect()
}
