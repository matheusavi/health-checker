use chrono::{Local, Timelike};
use config::{Config, ConfigError};
use discord::Discord;

use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
mod check_backup;
mod check_health;
mod discord;

const SETTINGS_FILE: &str = "./Settings";
const BACKUP_KEY: &str = "backup_path";
const HEALTHCHECK_KEY: &str = "healthcheck_urls";
const DISCORD_TOKEN_KEY: &str = "discord_token";
const DISCORD_CHANNEL_KEY: &str = "discord_channel";

const BACKUP_TIME: u32 = 5;

#[tokio::main]
async fn main() {
    let config = get_config().expect("Program was not able to locate config file");

    let backup_path = config
        .get_string(BACKUP_KEY)
        .expect("Program was not able to locate {BACKUP_KEY} key")
        .clone();

    let health_urls = config
        .get_array(HEALTHCHECK_KEY)
        .expect("Program was not able to locate {HEALTHCHECK_KEY} key")
        .clone();

    let token = config
        .get_string(DISCORD_TOKEN_KEY)
        .expect("Program was not able to locate {DISCORD_CHANNEL_KEY} key")
        .clone();

    let channel: i64 = config
        .get_int(DISCORD_CHANNEL_KEY)
        .expect("Program was not able to locate {DISCORD_CHANNEL_KEY} key")
        .clone();

    let discord = Arc::new(Discord::new(
        &token,
        channel
            .try_into()
            .expect("Unable to parse the channel id for discord"),
    ));

    for url in health_urls {
        let url = url
            .clone()
            .into_string()
            .expect("Failed to parse {url} for healthcheck");

        let discord = discord.clone();
        tokio::spawn(async move {
            loop {
                let result = check_health::get(&url).await;

                match result {
                    Ok(_) => println!("{url} healthy"),
                    Err(error) => {
                        let message = format!("{url} unhealthy {:?}", error);
                        println!("{}", message);
                        discord.send_discord_message(&message).await;
                    }
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
                let backup_done_today = check_backup::backup_done_today(&backup_path);
                if backup_done_today {
                    let message = "The database was not backed up today.";
                    println!("{}", message);
                    discord.send_discord_message(message).await;
                }

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
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen to ctrl+c");
    println!("Terminating application");
}

fn get_config() -> Result<Config, ConfigError> {
    Config::builder()
        .add_source(config::File::with_name(SETTINGS_FILE))
        .build()
}
