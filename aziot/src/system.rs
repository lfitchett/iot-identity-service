use structopt::StructOpt;

use aziot_common::{get_status, get_system_logs, restart, SERVICE_DEFINITIONS};

#[derive(StructOpt)]
pub enum SystemOptions {
    Restart(RestartOptions),
    Status(StatusOptions),
    Logs(LogsOptions),
}

#[derive(StructOpt)]
#[structopt(about = "Restart the services")]
pub struct RestartOptions {}

#[derive(StructOpt)]
#[structopt(about = "Report the status of system")]
pub struct StatusOptions {}

#[derive(StructOpt)]
#[structopt(about = "Get logs for the services")]
pub struct LogsOptions {
    /// Extra args to be passed to journalctl
    #[structopt(last = true)]
    args: Vec<String>,
}

pub fn system(options: SystemOptions) {
    match options {
        SystemOptions::Restart(_) => restart(SERVICE_DEFINITIONS),
        SystemOptions::Status(_) => get_status(SERVICE_DEFINITIONS),
        SystemOptions::Logs(opts) => logs(&opts),
    }
}

fn logs(options: &LogsOptions) {
    let services: Vec<&str> = SERVICE_DEFINITIONS.iter().map(|s| s.service).collect();
    let args: Vec<&str> = options.args.iter().map(|a| &**a).collect();

    get_system_logs(&services, &args);
}
