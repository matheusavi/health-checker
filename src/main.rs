use chrono::{Local, Timelike};
use config::{Config, ConfigError};
use std::time::Duration;
use tokio::time::sleep;
mod check_backup;
mod check_health;

const SETTINGS_FILE: &str = "./Settings";
const BACKUP_KEY: &str = "backup_path";
const HEALTHCHECK_KEY: &str = "healthcheck_urls";
const BACKUP_TIME: u32 = 5;

#[tokio::main]
async fn main() {
    let config = get_config().expect("Program was not able to locate config file");

    let backup_path = config
        .get_string(BACKUP_KEY)
        .expect("Program was not able to locate BACKUP_KEY key")
        .clone();

    let health_urls = config
        .get_array(HEALTHCHECK_KEY)
        .expect("Program was not able to locate HEALTHCHECK_KEY key")
        .clone();

    for url in health_urls {
        let url = url
            .clone()
            .into_string()
            .expect("Failed to parse {url} for healthcheck");

        tokio::spawn(async move {
            loop {
                let result = check_health::get(&url).await;

                match result {
                    Ok(_) => println!("{url} healthy"),
                    Err(error) => println!("{url} unhealthy {:?}", error),
                }

                sleep(Duration::from_secs(5 * 60)).await;
            }
        });
    }

    tokio::spawn(async move {
        loop {
            let now = Local::now().naive_local();
            let next_backup_time = if now.hour() >= BACKUP_TIME {
                println!("It's past {BACKUP_TIME}, checking if the database was backed up");
                println!("{}", check_backup::backup_done_today(&backup_path));

                now.date()
                    .succ_opt()
                    .unwrap()
                    .and_hms_opt(BACKUP_TIME, 1, 0)
            } else {
                now.date().and_hms_opt(BACKUP_TIME, 1, 0)
            };

            let sleep_duration =
                Duration::from_secs((next_backup_time.unwrap() - now).num_seconds() as u64);

            println!(
                "Checking the backup again at {}",
                next_backup_time.unwrap().format("%Y-%m-%d %H:%M:%S")
            );

            sleep(sleep_duration).await;
        }
    });

    handle_termination().await;
}

#[cfg(unix)]
async fn handle_termination() {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigint = signal(SignalKind::interrupt()).expect("Failed to set up SIGINT handler");
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to set up SIGTERM handler");

    tokio::select! {
        _ = sigint.recv() => println!("Received SIGINT"),
        _ = sigterm.recv() => println!("Received SIGTERM"),
    }
    println!("Terminating application");
}

#[cfg(windows)]
async fn handle_termination() {
    tokio::signal::ctrl_c().await.expect("Failed to listen to ctrl+c");
    println!("Terminating application");
}

fn get_config() -> Result<Config, ConfigError> {
    Config::builder()
        .add_source(config::File::with_name(SETTINGS_FILE))
        .build()
}
