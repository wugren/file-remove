use std::ops::Sub;
use std::path::Path;
use std::time::{Duration, SystemTime};
use clap::Arg;
use config::builder::DefaultState;
use config::ConfigBuilder;

fn main() {
    let matches = clap::Command::new("remove-file")
        .arg(
            Arg::new("path")
                .help("Set remove file path")
                .required(true),
        )
        .arg(
            Arg::new("days")
                .value_parser(clap::value_parser!(u32))
                .help("Set how many days old files to delete")
                .required(true),
        )
        .get_matches();

    let remove_path = matches.get_one::<String>("path").unwrap();
    let days= *matches.get_one::<u32>("days").unwrap();

    let exe_path = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let config = ConfigBuilder::<DefaultState>::default();
    let config_file = exe_path.join("file-remove.toml");
    if !config_file.exists() {
        println!("{} not exist", config_file.to_string_lossy().to_string());
        return;
    }

    let file = config::File::from(config_file.as_path());
    let config = config.add_source(file).build().unwrap();

    let white_list: Vec<String> = config.get("white_list").unwrap();
    if !white_list.contains(remove_path) {
        println!("{} is not whitelisted", remove_path);
        return;
    }

    if let Err(e) = delete_expired_files(Path::new(remove_path.as_str()), SystemTime::now().sub(Duration::from_secs(days as u64 * 24 * 3600))) {
        println!("err {}", e);
    }
}

fn delete_expired_files(dir: &Path, expiration_time: SystemTime) -> Result<bool, std::io::Error> {
    let read_dir = dir.read_dir()?;
    let mut is_empty = true;
    for entry in read_dir {
        if let Ok(entry) = entry {
            if entry.path().is_file() {
                let file_meta = entry.metadata()?;
                if file_meta.modified()? < expiration_time {
                    std::fs::remove_file(entry.path().as_path())?;
                    println!("remove file {}", entry.path().to_string_lossy().to_string());
                } else {
                    is_empty = false;
                }
            } else if entry.path().is_dir() {
                let is_sub_empty = delete_expired_files(entry.path().as_path(), expiration_time)?;
                if is_sub_empty {
                    std::fs::remove_dir(entry.path().as_path())?;
                    println!("remove dir {}", entry.path().to_string_lossy().to_string());
                } else {
                    is_empty = false;
                }
            }
        }
    }
    Ok(is_empty)
}
