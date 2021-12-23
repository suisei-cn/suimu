use anyhow::Result;
use serde::Deserialize;
use structopt::clap::crate_version;

#[derive(Deserialize, Debug)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize, Debug)]
struct ReleaseInfo {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

pub fn check_update() -> Result<()> {
    let user_agent = concat!("suimu/", crate_version!());
    let client = reqwest::blocking::Client::builder()
        .user_agent(user_agent)
        .build()?;
    let info = client
        .get("https://api.github.com/repos/suisei-cn/suimu/releases/latest")
        .send()?
        .json::<ReleaseInfo>()?;
    let curr_version = concat!("v", crate_version!());
    log::info!("Current version: {}", curr_version);
    if info.tag_name != curr_version {
        log::info!("Latest version: {}", info.tag_name);
        log::info!("Download the latest version:");
        for i in info.assets {
            log::info!("{} - {}", i.name, i.browser_download_url);
        }
    } else {
        log::info!("This is the latest version.");
    }
    Ok(())
}
