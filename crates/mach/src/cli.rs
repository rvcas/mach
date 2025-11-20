use clap::Parser;

use crate::{BANNER, cmd, service::Services};

#[derive(Parser)]
#[clap(version, about, long_about = Some(BANNER))]
#[clap(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Option<cmd::Cmd>,
}

impl Default for Cli {
    fn default() -> Self {
        Self::parse()
    }
}

impl Cli {
    pub async fn exec(self) -> miette::Result<()> {
        let services = Services::bootstrap().await?;

        match self.cmd {
            Some(cmd) => cmd.exec(&services).await,
            None => {
                todo!("start the tui")
            }
        }
    }
}
