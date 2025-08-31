use chrono::Local;
use clap::{Command, arg};
use std::{io::Write, usize};
use tokio::io::Error;
mod amslib;
use amslib::AmsData;
#[macro_use]
extern crate prettytable;

#[tokio::main]
async fn main() {
    // Start by clearing the screen
    clearscreen::clear().expect("Failed to clear the screen...");
    // Read the arguments provided
    let matches = Command::new("ams")
        .version("1.0")
        .about("Fetches and displays IT jobadverts from arbetsformedlingen.se in Örebro")
        .arg(
            arg!(--numadverts <VALUE>)
                .required(false)
                .default_value("15"),
        )
        .get_matches();

    let num_adverts = matches.get_one::<String>("numadverts").unwrap();
    let today = Local::now();

    // Call amslib to retrieve and print the table
    let data: AmsData = amslib::run(&num_adverts, &today).await;
    println!("To open a advert url, input the 'id' and press Enter, or 'q' to quit.");
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
        } else {
            println!("Type 'q' or 'quit' to exit.");
        }
    }
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
