use std::thread::sleep;

use anyhow::anyhow;
use clap::Parser;
use cliclack::{input, intro, log, note, outro, select, spinner, Confirm};
use dialoguer::Editor;

/// The initialization command
/// This command cleans the problem directories and sets up a blank repository. Run this after
/// you've cloned the repo to set up your own LeetCode problem manager.
#[derive(Parser, Debug)]
pub struct InitCommand {
    /// Force override of existing directory
    #[arg(short, long, default_value = "false")]
    force: bool,

    /// Do not initialize a git repository
    #[arg(long, name = "no-git", default_value = "false")]
    no_git: bool,
}

impl InitCommand {
    pub fn run(&self) {
        println!("Initializing new project...\n");

        if self.force {
            self._run()
        } else {
            let confirm = Confirm::new("Are you sure you want to delete existing problems?")
                .initial_value(true)
                .interact()
                .unwrap();

            if confirm {
                self._run();
            }
        }
    }

    fn _run(&self) {
        let mut spinner = spinner();
        spinner.start("Deleting existing problems...");
        sleep(std::time::Duration::from_secs(2));

        if let Err(e) = reset_project() {
            spinner.error(format!("Failed to reset project: {}", e));
            return;
        }
        spinner.stop("Existing problems deleted.");
    }
}

// Erase every file in src/problem/ and src/solution and create an empty mod.rs file in each
fn reset_project() -> anyhow::Result<()> {
    let problem_dir = std::path::Path::new("src/problem");
    let solution_dir = std::path::Path::new("src/solution");

    if problem_dir.exists() {
        // log("Removing problem directory...");
        std::fs::remove_dir_all(problem_dir)?;
    }

    if solution_dir.exists() {
        // log("Removing solution directory...");
        std::fs::remove_dir_all(solution_dir)?;
    }

    // log("Creating problem directory...");
    std::fs::create_dir(problem_dir)?;

    // log("Creating solution directory...");
    std::fs::create_dir(solution_dir)?;

    // log("Creating mod.rs in problem directory...");
    std::fs::write(problem_dir.join("mod.rs"), "")?;

    // log("Creating mod.rs in solution directory...");
    std::fs::write(solution_dir.join("mod.rs"), "")?;

    Ok(())
}
