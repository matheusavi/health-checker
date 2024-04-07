use std::{
    fs::{self, DirEntry},
    io,
};

use chrono::{DateTime, Local};

pub fn backup_done_today(backup_dir: &str) -> bool {
    match get_newer_file_in_directory(&backup_dir) {
        Ok(file) => {
            let creation_date = match get_file_modification_time(&file) {
                Ok(time) => time,
                Err(error) => {
                    println!("Error while trying to get the file modification time {error}");
                    return false;
                }
            };
            match get_file_name(&file) {
                Ok(name) => {
                    println!("The newest file found is {name}, modified in {creation_date}")
                }
                Err(error) => println!("Error while trying to get the file name {error}"),
            }
            let now: DateTime<Local> = Local::now();
            println!("{}-{}", creation_date.date_naive(), now.date_naive());
            creation_date.date_naive() == now.date_naive()
        }
        Err(error) => {
            println!("Error while trying to get the newest backup file {error}");
            false
        }
    }
}

fn get_newer_file_in_directory(dir: &str) -> Result<DirEntry, io::Error> {
    fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|f| f.path().is_file())
        .max_by_key(|entry| {
            entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
        })
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "No files found"))
}

fn get_file_modification_time(file: &DirEntry) -> Result<DateTime<Local>, io::Error> {
    let metadata = file.metadata()?;
    let modified_time = metadata.modified()?;
    let datetime: DateTime<Local> = modified_time.into();

    Ok(datetime)
}

fn get_file_name(file: &DirEntry) -> Result<String, io::Error> {
    file.file_name()
        .into_string()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Unable to get file name"))
}
