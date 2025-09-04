use chrono::{DateTime, Local};
use clap::{Command, arg};
use std::{io::Write, usize};
use tokio::io::Error;
mod amslib;
use amslib::AmsData;
#[macro_use]
extern crate prettytable;

#[tokio::main]
async fn main() {
    // Read the arguments provided
    let matches = Command::new("ams")
        .version("1.0")
        .about("Fetches and displays IT jobadverts from arbetsformedlingen.se in Örebro")
        .arg(arg!(-n --num <VALUE>).required(false).default_value("15"))
        .arg(
            arg!(-c --clear <VALUE>)
                .required(false)
                .default_value("false")
                .default_missing_value("false"),
        )
        .get_matches();

    let num_adverts = matches.get_one::<String>("num").unwrap();
    let should_clear_screen = matches.get_one::<String>("clear").unwrap();

    match should_clear_screen.parse::<bool>().unwrap() {
        false => (),
        true => clearscreen::clear().expect("Failed to clear the screen..."),
    }

    // Call amslib to retrieve and print the table
    let current_date = Local::now();
    let mut data = call_amslib(num_adverts, &current_date).await;

    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let inp = read_input().expect("Problem reading input");
        // Check if input is a parsable number
        if inp.trim().parse::<usize>().is_ok() {
            let selected_index: usize = inp
                .trim()
                .parse()
                .expect("Failed to parse selected index-string to usize");
            // TODO: this could be handled the Rust way with match?
            if selected_index <= data.ads.len() + 1 && selected_index != 0 {
                // Open the link
                let combined_url = "https://arbetsformedlingen.se/platsbanken/annonser/".to_owned()
                    + &data.ads[selected_index - 1].id;
                // TODO: Check result here if the browser could not be opened.
                open_browser_link(combined_url).expect("Problem opening browser!");
            } else {
                println!("No ad matches that id.")
            }
        // Otherwise check if it is the quit sequence
        } else if inp.trim() == "q" || inp.trim() == "quit" {
            std::process::exit(1);
        } else if inp.trim() == "r" || inp.trim() == "reload" {
            // Clear the screen and refetch the data, mutating 'data'
            clearscreen::clear().expect("Failed to clear the screen...");
            let current_date = Local::now();
            data = call_amslib(num_adverts, &current_date).await;
        } else {
            println!("Type 'q' or 'quit' to exit.");
        }
    }
}

async fn call_amslib(num_adverts: &String, current_date: &DateTime<Local>) -> AmsData {
    let res: Result<AmsData, reqwest::Error> = amslib::run(&num_adverts, &current_date).await;
    if res.is_err() {
        println!("Nework connectivity issues, server not reachable...");
        std::process::exit(1);
    }
    println!("To open a advert url, input the 'id' and press Enter, or 'q' to quit.");
    println!("Reload the adverts with 'r' or 'reload'");
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
