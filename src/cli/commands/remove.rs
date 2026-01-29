//! ms remove - Remove a skill from the index
//!
//! This command removes a skill from the local index (database and search).
//! It does not delete the source files.

use clap::Args;
use colored::Colorize;

use crate::app::AppContext;
use crate::error::Result;

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// ID of the skill to remove
    pub id: String,

    /// Skip confirmation
    #[arg(long, short = 'y')]
    pub yes: bool,
}

pub fn run(ctx: &AppContext, args: &RemoveArgs) -> Result<()> {
    let skill_opt = ctx.db.get_skill(&args.id)?;

    if skill_opt.is_none() {
        println!("{} Skill not found: {}", "✘".red(), args.id);
        return Ok(());
    }
    let skill = skill_opt.unwrap();

    if !args.yes {
        println!(
            "About to remove skill '{}' ({}) from the index.",
            skill.name.bold(),
            skill.id.cyan()
        );
        println!("This will NOT delete the source file at: {}", skill.source_path);
        
        // Simple confirmation prompt since we don't have the fancy prompts public here
        // or we can reuse what prune uses if it's public. 
        // For now, let's keep it simple or implement a quick prompt.
        match prompt_yes_no("Are you sure? [y/N] ") {
            Ok(true) => (),
            Ok(false) => {
                println!("Aborted.");
                return Ok(());
            }
            Err(_) => return Ok(()),
        }
    }

    // 1. Remove from SQLite
    ctx.db.delete_skill(&args.id)?;

    // 2. Remove from Search Index
    ctx.search.delete_skill(&args.id)?;
    ctx.search.commit()?;

    println!("{} Removed skill: {}", "✓".green(), args.id);

    Ok(())
}

fn prompt_yes_no(prompt: &str) -> std::io::Result<bool> {
    use std::io::{self, Write};
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    Ok(input == "y" || input == "yes")
}
