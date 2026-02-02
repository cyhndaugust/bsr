use anyhow::Result;
use self_update::cargo_crate_version;

pub fn handle() -> Result<()> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("cyhndaugust")
        .repo_name("bsr")
        .bin_name("bsr")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    println!("Update status: `{}`!", status.version());
    Ok(())
}
