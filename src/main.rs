use notify_rust::{Notification, Timeout};
use regex::Regex;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Deserialize)]
struct Config {
    ping_dst: String,
    check_freq: u32,
    warning_threshold: u32,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Config {
            ping_dst: "8.8.8.8".to_string(),
            check_freq: 30,
            warning_threshold: 80,
        }
    }
}

fn main() {
    let re = Regex::new(r"(?m).*rtt min/avg/max/mdev = (?P<min>[0-9\.]{6})/(?P<avg>[0-9\.]{6})/(?P<max>[0-9\.]{6}).*ms$").unwrap();

    let config = match xdg::BaseDirectories::new() {
        Ok(xdg) => match xdg.find_config_file("netfluff.toml") {
            Some(config) => match File::open(config) {
                Ok(mut f) => {
                    let mut s = String::new();
                    f.read_to_string(&mut s).unwrap();
                    match toml::from_str(&s) {
                        Ok(t) => t,
                        Err(e) => {
                            eprintln!("Cannot parse config file: {} - using defaults", e);
                            Config::default()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Cannot open config file: {} - using defaults", e);
                    Config::default()
                }
            },
            None => {
                eprintln!("Cannot find config file - using defaults");
                Config::default()
            }
        },
        Err(e) => {
            eprintln!("Cannot find config file: {} - using defaults", e);
            Config::default()
        }
    };
    loop {
        match Command::new("ping")
            .arg("-c")
            .arg("5")
            .arg("-i")
            .arg(".2")
            .arg(&config.ping_dst)
            .output()
        {
            Ok(output) => {
                let std_out = String::from_utf8_lossy(&output.stdout).into_owned();

                let cap = match re.captures(&std_out) {
                    Some(c) => c,
                    None => {
                        continue;
                    }
                };
                let max = cap["max"].parse().unwrap_or(0f32);
                let min = cap["min"].parse().unwrap_or(0f32);
                let avg = cap["avg"].parse().unwrap_or(0f32);

                if avg > config.warning_threshold as f32 {
                    match Notification::new()
                        .summary("Network degradation")
                        .body(&format!(
                            "Latency is getting worse!\nmin: {} | avg: {} | max: {}",
                            min, avg, max
                        ))
                        .icon(
                            "/usr/share/icons/Adwaita/96x96/emotes/face-sick-symbolic.symbolic.png",
                        )
                        .timeout(Timeout::Milliseconds(10000))
                        .show()
                    {
                        Ok(_) => {}
                        Err(e) => eprintln!("Unable to generate notification: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("Could not read check output: {}", e),
        }

        thread::sleep(Duration::from_secs(config.check_freq.into()));
    }
}
