// use std::thread::sleep;

// use anyhow::anyhow;
use clap::Parser;
// use cliclack::{input, intro, log, note, outro, select, spinner, Confirm};
use cliclack::{input, log};
// use dialoguer::Editor;
use regex::Regex;

use crate::common::deal::deal_problem;
use crate::common::fetch::{self, get_initialized_problems, get_user_problems};

/// The initialization command
/// This command cleans the problem directories and sets up a blank repository. Run this after
/// you've cloned the repo to set up your own LeetCode problem manager.
#[derive(Parser, Debug)]
pub struct PullCommand {
    /// The problem ID to fetch
    #[arg(short, long)]
    id: Option<u32>,

    /// Force override of existing problem
    #[arg(long, default_value = "false")]
    force: bool,
}

impl PullCommand {
    pub async fn run(&self) {
        let _problems = get_user_problems().await;

        let mut initialized = get_initialized_problems();

        let id = match &self.id {
            Some(id) => *id,
            None => {
                let pid: String = input("Enter a problem id!")
                    .placeholder("1")
                    .validate(|input: &String| {
                        let re = Regex::new(r"^[0-9]*$").unwrap();
                        if !re.is_match(input) {
                            return Err("Invalid problem id - must be an integer");
                        }
                        Ok(())
                    })
                    .interact()
                    .expect("Failed to get problem id");

                pid.parse::<u32>()
                    .unwrap_or_else(|_| panic!("Not a number: {}", pid))
            }
        };

        if initialized.contains(&id) {
            println!("The problem you chose has already been initialized in problem/");
            return;
        }

        log::info(format!("Fetching problem #{}", id)).expect("Failed to log");
        let problem = fetch::get_problem(id).await.unwrap_or_else(|| {
            panic!(
                "Error: failed to get problem #{}\
                (The problem may be paid-only or may not exist).",
                id
            )
        });
        let code = problem.code_definition.iter().find(|&d| d.value == *"rust");
        if code.is_none() {
            println!("Problem {} has no rust version.", &id);
            initialized.push(problem.question_id);
            return;
        }

        let code = code.unwrap();

        deal_problem(&problem, code, true);
    }
}
