mod server;

use crate::service::Services;

#[derive(clap::Args)]
pub struct Args;

impl Args {
    pub async fn exec(self, services: &Services) -> miette::Result<()> {
        server::run(services.clone()).await
    }
}
