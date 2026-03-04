use chrono::{DateTime, Local};
use clap::{Command, arg};
use std::io::Write;
use std::sync::Arc;
use tokio::io::Error;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio::{task, time};

mod amslib;
use amslib::AmsData;

#[macro_use]
extern crate prettytable;

#[tokio::main]
async fn main() {
    let matches = Command::new("ams")
        .version("1.0")
        .about("Fetches and displays IT job adverts from arbetsformedlingen.se in Örebro")
        .arg(arg!(-n --num <VALUE>).required(false).default_value("15"))
        .arg(
            arg!(-m --minutesautofetch <VALUE>)
                .required(false)
                .default_value("0"),
        )
        .arg(
            arg!(-c --clear <VALUE>)
                .required(false)
                .default_value("false")
                .default_missing_value("false"),
        )
        .get_matches();

    // Clone values so they can be moved into async task safely
    let num_adverts: String = matches.get_one::<String>("num").unwrap().clone();

    let should_clear_screen: bool = matches.get_one::<String>("clear").unwrap().parse().unwrap();

    let refetch_in_minutes: u64 = matches
        .get_one::<String>("minutesautofetch")
        .unwrap()
        .parse()
        .expect("Error parsing periodic minutes argument");

    if should_clear_screen {
        clearscreen::clear().expect("Failed to clear the screen...");
    }

    let current_date = Local::now();

    let data = Arc::new(Mutex::new(call_amslib(&num_adverts, &current_date).await));

    if refetch_in_minutes > 0 {
        let data_clone = Arc::clone(&data);
        let num_clone = num_adverts.clone();

        task::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(refetch_in_minutes * 60));

            loop {
                interval.tick().await;

                clearscreen::clear().ok();

                let current_date = Local::now();
                let new_data = call_amslib(&num_clone, &current_date).await;

                let mut locked = data_clone.lock().await;
                *locked = new_data;
            }
        });
    }

    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let inp = read_input().expect("Problem reading input");
        let inp = inp.trim().to_lowercase();

        if let Ok(selected_index) = inp.parse::<usize>() {
            let locked = data.lock().await;

            if selected_index > 0 && selected_index <= locked.ads.len() {
                let combined_url = "https://arbetsformedlingen.se/platsbanken/annonser/".to_owned()
                    + &locked.ads[selected_index - 1].id;

                open_browser_link(combined_url).expect("Problem opening browser!");
            } else {
                println!("No ad matches that id.");
            }
        } else if inp == "q" || inp == "quit" {
            std::process::exit(0);
        } else if inp == "r" || inp == "reload" {
            clearscreen::clear().ok();

            let current_date = Local::now();
            let new_data = call_amslib(&num_adverts, &current_date).await;

            let mut locked = data.lock().await;
            *locked = new_data;
        } else {
            println!("Type a number to open an advert.");
            println!("Type 'r' or 'reload' to refresh.");
            println!("Type 'q' or 'quit' to exit.");
        }
    }
}

async fn call_amslib(num_adverts: &String, current_date: &DateTime<Local>) -> AmsData {
    let res: Result<AmsData, reqwest::Error> = amslib::run(num_adverts, current_date).await;

    if res.is_err() {
        println!("Network connectivity issues, server not reachable...");
        std::process::exit(1);
    }

    println!("To open an advert url, input the number and press Enter.");
    println!("Reload the adverts with 'r' or 'reload'");
    println!("Quit with 'q' or 'quit'");

    res.unwrap()
}

fn read_input() -> Result<String, Error> {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut buffer)?;
    Ok(buffer)
}

fn open_browser_link(url: String) -> Result<(), Error> {
    println!("Opening link {}", url);
    open::that(url)?;
    Ok(())
}
