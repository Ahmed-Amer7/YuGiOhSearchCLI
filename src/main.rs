use std::io::Write;
use image::GenericImageView;
use termimage::ops;
use colored::Colorize;
use reqwest::blocking::get;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

#[derive(Debug, Deserialize, Serialize)]
struct Card{
    name: Option<String>,
    id: Option<i32>,
    atk: Option<i32>,
    def: Option<i32>,
    archetype: Option<String>,
    card_images: Vec<CardImages>,
    desc: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CardImages {
    image_url: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
struct Response {
    data: Vec<Card>
}

fn main() {
    let mut keep_going = true;

    while keep_going {
        let mut chosen_card = String::new();

        print!("{}", "Enter A Card Name: ".green());
        std::io::stdout().flush().unwrap();

        std::io::stdin().read_line(&mut chosen_card).expect("Enter a valid Yu-Gi-Oh! Card Name");

        let res = match call_api(&chosen_card) {
            Some(res) => {
                res
            },
            None => {
                continue;
            }
        };

        select_card(res);

        let mut again = String::new();

        print!("{}", "Do You Want to Search Another Card? Type y For Yes: ".green());
        std::io::stdout().flush().unwrap();

        std::io::stdin().read_line(&mut again).expect("y or n");

        match again.trim() {
            "y" | "Y" => {},
            _ => { keep_going = false; }
        }

    }
}

fn call_api(chosen_card: &String) -> Option<Response> {

    let url = format!("https://db.ygoprodeck.com/api/v7/cardinfo.php?fname={}", chosen_card.trim());
    let res = get(&url).unwrap();

    if !res.status().is_success() {
        println!("That's an Invalid Card Name, Try Again");
        return None;
        // return Err("Failed to fetch card data".into());
    }

    let res: Response = res.json().unwrap();

    if res.data.is_empty() {
        println!("No cards found for '{}'", chosen_card.trim());
        return None;
        // return Err("No card data returned".into());
    }

    for (index, card) in res.data.iter().enumerate() {
        let idx_part = format!("[{}]", index).red();
        let name_part = format!("{}", serde_json::to_string_pretty(&card.name).unwrap().blue());
        println!("{idx_part} {name_part}");
    };

    return Some(res);
    // Ok(res)
}

fn select_card(cards: Response) -> () {
    let mut chosen = String::new();

    print!("{}", "Select A Card Index: ".green());
    std::io::stdout().flush().unwrap();

    std::io::stdin().read_line(&mut chosen).expect("Entered an Invalid Number!");
    
    let chosen = match chosen.trim().parse::<usize>() {
        Ok(num) => { 
            if num < cards.data.len() {
                num
            } else {
                println!("Entered Invalid Index, Searching for First Card");
                0
            }
        },
        Err(_) => {
            println!("Entered Invalid Index, Searching for First Card");
            0
        }
    };

    print_details(&cards.data[chosen]);

    print_image(&cards.data[chosen]);
}

fn print_details(card: &Card) {
    let name = format!("{}", "Name:").red();
    let atk = format!("{}", "ATK:").red();
    let def = format!("{}", "DEF:").red();
    println!("{name} {}", card.name.clone().unwrap().bright_blue());
    println!("{atk} {}", card.atk.clone().unwrap().to_string().bright_blue());
    println!("{def} {}", card.def.clone().unwrap().to_string().bright_blue());
}

fn print_image(card: &Card) {
    let image_url = card.card_images[0].image_url.clone().unwrap();
    let bytes = get(image_url).unwrap().bytes().unwrap();

    let mut temp = NamedTempFile::new().unwrap();
    temp.write_all(&bytes).unwrap();

    let size = (20, 15);
    let preserve_aspect = true;

    let format = ops::guess_format(&("image".to_string(), temp.path().to_owned())).unwrap();
    let img = ops::load_image(&("image".to_string(), temp.path().to_owned()), format).unwrap();

    let img_s = ops::image_resized_size(img.dimensions(), size, preserve_aspect);
    let resized = ops::resize_image(&img, img_s);

    ops::write_ansi_truecolor(&mut std::io::stdout(), &resized);

    let desc = format!("{}", "Desc:").red();
    println!("{desc} {}", card.desc.clone().unwrap().bright_blue());
}