use std::fs::File;
use std::io::{Write, BufReader, BufRead};

struct Book {
    title: String,
    author: String,
    year: u16,
}

fn save_books(books: &Vec<Book>, filename: &str) {
   let mut file = File::create(filename).unwrap();

   for book in books.iter() {
        writeln!(file, "{} {} {}", book.title, book.author, book.year).unwrap();
   }
    
}

fn load_books(filename: &str) -> Vec<Book> {
     let mut book_list: Vec<Book> = Vec::new();

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();

        // Find the last space to separate year from the rest
        if let Some(dot) = line.rfind(' ') {
            let year_str = &line[dot+1..];
            let year: u16 = year_str.parse().unwrap_or(0);

            let rest = &line[..dot];
            if let Some(dot2) = rest.rfind(' ') {
                let title = rest[..dot].to_string();
                let author = rest[dot2+1..].to_string();

                book_list.push(Book { title, author, year });
            }
        }
    }

    book_list
}

fn main() {
    let books = vec![
        Book { title: "1984".to_string(), author: "George Orwell".to_string(), year: 1949 },
        Book { title: "To Kill a Mockingbird".to_string(), author: "Harper Lee".to_string(), year: 1960 },
    ];

    save_books(&books, "books.txt");
    println!("Books saved to file.");

    let loaded_books = load_books("books.txt");
    println!("Loaded books:");
    for book in loaded_books {
        println!("{} by {}, published in {}", book.title, book.author, book.year);
    }
}