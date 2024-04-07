use chrono::{Local, Timelike};
use config::{Config, ConfigError};
use std::{collections::HashMap, time::Duration};
use tokio::time::sleep;
mod check_backup;

const SETTINGS_FILE: &str = "./Settings";
const BACKUP_KEY: &str = "backup_path";
const BACKUP_TIME: u32 = 5;

#[tokio::main]
async fn main() {
    let config = get_config().expect("Program was not able to locate config file");

    let backup_path = config
        .get(BACKUP_KEY)
        .expect("Program was not able to locate backup_path key")
        .clone();

    //make a foreach with the multiple URLs, verify each one every 5 min

    tokio::spawn(async move {
        loop {
            let res = reqwest::get("http://localhost:5121/health").await.unwrap();
            println!("Status: {}", res.status());
            println!("Headers:\n{:#?}", res.headers());

            let body = res.text().await.unwrap();
            println!("Body:\n{}", body);

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
    })
    .await
    .expect("Failed to verify the backup");
}

fn get_config() -> Result<HashMap<String, String>, ConfigError> {
    Config::builder()
        .add_source(config::File::with_name(SETTINGS_FILE))
        .build()?
        .try_deserialize::<HashMap<String, String>>()
}
