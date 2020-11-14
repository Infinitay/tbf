use log::{info, debug, error};
use clap::{load_yaml, crate_authors, crate_description, crate_version, App};
use rayon::prelude::*;
use env_logger::Env;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::prelude::*;
use std::io::stdin;
use regex::Regex;
use reqwest::blocking;
use url::Url;
use indicatif::ParallelProgressIterator;
use std::convert::TryFrom;

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

fn check_availability(url: &str) {
    let parsed = Url::parse(url).unwrap();
    let segments = parsed.path_segments().map(|c| c.collect::<Vec<_>>()).unwrap();
    if segments[segments.len()-1] == "index-dvr.m3u8" {
        let url_normal = format!("https://vod-secure.twitch.tv{}1.ts", parsed.path().split_at(61).0);
        let url_muted = format!("https://vod-secure.twitch.tv{}1-muted.ts", parsed.path().split_at(61).0);
        let res_normal = blocking::get(url_normal.as_str()).unwrap();
        let res_muted = blocking::get(url_muted.as_str()).unwrap();
        if res_normal.status() == 200 || res_muted.status() == 200 {
            info!("This VOD is available for watching/downloading! :)")
        } else {
            info!("This VOD has been deleted from Twitch servers :(")
        }
    }
}

fn parse_timestamp(timestamp: &str) -> i64 {
    let re_unix = Regex::new(r"^\d*$").unwrap();
    let re_utc = Regex::new("UTC").unwrap();

    if re_unix.is_match(timestamp) {
        timestamp.parse::<i64>().unwrap()
    } else {
        if re_utc.is_match(timestamp) {
            NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S UTC").unwrap().timestamp()
        } else {
            let meme = DateTime::parse_from_rfc3339(timestamp);
            match meme {
                Ok(result) => result.timestamp(),
                Err(_) => NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S").unwrap().timestamp(),
            }
        }
    }
}

fn bruteforcer(username: &str, vod: i64, initial_from_stamp: &str, initial_to_stamp: &str, verbose: bool) {
    let mut log_level = "info";
    if verbose { log_level = "debug" };

    env_logger::init_from_env(
        Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, log_level));

    let number1 = parse_timestamp(initial_from_stamp);
    let number2 = parse_timestamp(initial_to_stamp);

    let final_url_check = AtomicBool::new(false);
    let final_url_atomic = Arc::new(Mutex::new(String::new()));
    let mut initial_url_vec: Vec<String> = Vec::new();
    let client = blocking::Client::new();
    info!("Starting!");
    for number in number1..number2+1 {
        let mut hasher = Sha1::new();
        hasher.input_str(format!("{}_{}_{}", username, vod, number).as_str());
        let hex = hasher.result_str();
        initial_url_vec.push(format!("https://vod-secure.twitch.tv/{}_{}_{}_{}/chunked/index-dvr.m3u8", &hex[0..20], username, vod, number));
    }
    debug!("Finished making urls.");
    let vec_len_u64 = u64::try_from(initial_url_vec.len()).unwrap();
    initial_url_vec.par_iter().progress_count(vec_len_u64).for_each( |url| {
        if !final_url_check.load(Ordering::SeqCst) {
            let final_url_atomic = Arc::clone(&final_url_atomic);
            let res = client.get(&url.clone()).send().expect("Error");
            if res.status() == 200 {
                final_url_check.store(true, Ordering::SeqCst);
                let mut final_url = final_url_atomic.lock().unwrap();
                *final_url = url.to_string();
                debug!("Got it! - {:?}", url);
            } else if res.status() == 403 {
                debug!("Still going - {:?}", url);
            } else {
                error!("You might be getting throttled (or your connection is dead)! Status code: {}", res.status());
            }
        }
    });

    let final_url = &*final_url_atomic.lock().unwrap();
    if !final_url.is_empty() {
        info!("Got the URL! - {}", final_url);
        check_availability(final_url);
    } else {
        info!("Couldn't find anything :(");
    }
}

fn exact(username: &str, vod: i64, initial_stamp: &str, verbose: bool) {
    let mut log_level = "info";
    if verbose { log_level = "debug" };

    env_logger::init_from_env(
        Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, log_level));

    let number = parse_timestamp(initial_stamp);
    
    let mut hasher = Sha1::new();
    hasher.input_str(format!("{}_{}_{}", username, vod, number).as_str());
    let hex = hasher.result_str();
    let final_url = format!("https://vod-secure.twitch.tv/{}_{}_{}_{}/chunked/index-dvr.m3u8", &hex[0..20], username, vod, number);
    info!("Made the URL! - {}", final_url);
    check_availability(final_url.as_str());
}

fn interface() {
    let mut mode = String::new();

    println!("Please select the mode you want:");
    println!("[1] Exact mode - Combines all the parts (streamer's username, VOD/broadcast ID and a timestamp) into a proper m3u8 URL");
    println!("[2] Bruteforce mode - Goes over a range of timestamps, looking for a usable/working m3u8 URL");

    stdin().read_line(&mut mode).expect("Failed to read line.");
    trim_newline(&mut mode);

    match mode.as_str() {
        "1" => {
            let mut username = String::new();
            let mut vod = String::new();
            let mut initial_stamp = String::new();

            println!("Please enter the streamer's username:");
            stdin().read_line(&mut username).expect("Failed to read line.");
            trim_newline(&mut username);
            println!("Please enter the VOD/broadcast ID:");
            stdin().read_line(&mut vod).expect("Failed to read line.");
            trim_newline(&mut vod);
            println!("Please enter the timestamp:");
            stdin().read_line(&mut initial_stamp).expect("Failed to read line.");
            trim_newline(&mut initial_stamp);

            exact(username.as_str(), vod.parse::<i64>().unwrap(), initial_stamp.as_str(), false);
        },
        "2" => {
            let mut username = String::new();
            let mut vod = String::new();
            let mut initial_from_stamp = String::new();
            let mut initial_to_stamp = String::new();

            println!("Please enter the streamer's username:");
            stdin().read_line(&mut username).expect("Failed to read line.");
            trim_newline(&mut username);
            println!("Please enter the VOD/broadcast ID:");
            stdin().read_line(&mut vod).expect("Failed to read line.");
            trim_newline(&mut vod);
            println!("Please enter the first timestamp:");
            stdin().read_line(&mut initial_from_stamp).expect("Failed to read line.");
            trim_newline(&mut initial_from_stamp);
            println!("Please enter the last timestamp:");
            stdin().read_line(&mut initial_to_stamp).expect("Failed to read line.");
            trim_newline(&mut initial_to_stamp);

            bruteforcer(username.as_str(), vod.parse::<i64>().unwrap(), initial_from_stamp.as_str(), initial_to_stamp.as_str(), false);
        }
        _ => {}
    }
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .get_matches();

    match matches.subcommand_name() {
        Some("bruteforce") => {
            if let Some(matches) = matches.subcommand_matches("bruteforce") {
                let username = matches.value_of("username").unwrap();
                let vod = matches.value_of("id").unwrap().parse::<i64>().unwrap();
                let initial_from_stamp = matches.value_of("from").unwrap();
                let initial_to_stamp = matches.value_of("to").unwrap();

                let mut verbose = false;
                if matches.is_present("v") {
                    verbose = true;
                }

                bruteforcer(username, vod, initial_from_stamp, initial_to_stamp, verbose);
            }
        },
        Some("exact") => {
            if let Some(matches) = matches.subcommand_matches("exact") {
                let username = matches.value_of("username").unwrap();
                let vod = matches.value_of("id").unwrap().parse::<i64>().unwrap();
                let initial_stamp = matches.value_of("stamp").unwrap();

                let mut verbose = false;
                if matches.is_present("v") {
                    verbose = true;
                }

                exact(username, vod, initial_stamp, verbose);
            }
        },
        _ => interface()
    }
}