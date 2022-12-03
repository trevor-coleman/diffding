use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::path::PathBuf;

use config::{Config, File};

use crate::Options;

pub fn get_options() -> Result<Options, Box<dyn Error>> {
    let config_path = get_config_path();

    let settings = Config::builder()
        .add_source(File::from(config_path.join("config.toml")).required(false))
        .build()?;

    let settings = settings
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let config_options = Options {
        sound_path: match settings.get("sound") {
            Some(sound) => Some(PathBuf::from(config_path.join(sound))),
            None => None,
        },
        threshold: settings
            .get("threshold")
            .unwrap_or(&"".to_string())
            .parse::<i32>()
            .unwrap_or(100),
        loop_time: settings
            .get("interval")
            .unwrap_or(&"".to_string())
            .parse::<u64>()
            .unwrap_or(10),
        volume: settings
            .get("volume")
            .unwrap_or(&"".to_string())
            .parse::<f32>()
            .unwrap_or(1.0),
        snooze_length: settings
            .get("snooze_length")
            .unwrap_or(&"".to_string())
            .parse::<i64>()
            .unwrap_or(5),
    };

    let args: Vec<String> = env::args().collect();

    let options: Options = match args.len() {
        1 => config_options,
        2 => Options {
            loop_time: args[1].parse::<u64>().unwrap(),
            sound_path: config_options.sound_path,
            threshold: config_options.threshold,
            volume: config_options.volume,
            snooze_length: config_options.snooze_length,
        },
        _ => Options {
            loop_time: args[1].parse::<u64>().unwrap(),
            sound_path: config_options.sound_path,
            threshold: args[2].parse::<i32>().unwrap(),
            volume: config_options.volume,
            snooze_length: config_options.snooze_length,
        },
    };

    Ok(options)
}

fn get_config_path() -> PathBuf {
    let mut config_path = PathBuf::new();
    config_path.push(env::var("HOME").unwrap());
    config_path.push(".config");
    config_path.push("diffding");
    config_path
}
