use anyhow::Result;
use colored::*;
use self_update::cargo_crate_version;

pub fn handle() -> Result<()> {
    println!("{}", "Checking for updates...".cyan());

    let status = self_update::backends::github::Update::configure()
        .repo_owner("cyhndaugust")
        .repo_name("bsr")
        .bin_name("bsr")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .no_confirm(true)
        .build()?
        .update()?;

    if status.updated() {
        println!(
            "\n{} {} {} {}",
            "Successfully updated to version".green(),
            status.version().bold().white(),
            "from".green(),
            cargo_crate_version!().bold().white()
        );
        println!("{}", "✨ Enjoy the new features!".yellow());
    } else {
        println!(
            "\n{} {}",
            "You are already using the latest version:".green(),
            status.version().bold().white()
        );
    }

    Ok(())
}
