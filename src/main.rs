use std::error::Error;
use sqlx::Row;
use gtk4::prelude::*;
use gtk4::glib::clone;

use gtk4::glib;

#[derive(Debug)]
struct Book {
    pub title: String,
    pub author: String,
    pub isbn: String,
}

async fn create(book: &Book, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    // Check if the book with the same ISBN already exists
    let query = "SELECT COUNT(*) FROM book WHERE isbn = $1";
    let count: i64 = sqlx::query_scalar(query)
        .bind(&book.isbn)
        .fetch_one(pool)
        .await?;

    if count > 0 {
        return Err("A book with the same ISBN already exists".into());
    }

    // Insert the book into the database
    let query = "INSERT INTO book (title, author, isbn) VALUES ($1, $2, $3)";

    sqlx::query(query)
        .bind(&book.title)
        .bind(&book.author)
        .bind(&book.isbn)
        .execute(pool)
        .await?;

    Ok(())
}

async fn remove_book_by_isbn(pool: &sqlx::PgPool, isbn: &str) -> Result<(), Box<dyn Error>> {
    // Utiliser une requête DELETE pour retirer le livre par ISBN de la base de données
    let query = "DELETE FROM book WHERE isbn = $1";
    sqlx::query(query).bind(isbn).execute(pool).await?;

    Ok(())
}

async fn remove_book_by_title(pool: &sqlx::PgPool, title: &str) -> Result<(), Box<dyn Error>> {
    // Utiliser une requête DELETE pour retirer le livre par ISBN de la base de données
    let query = "DELETE FROM book WHERE title = $1";
    sqlx::query(query).bind(title).execute(pool).await?;

    Ok(())
}

async fn read(conn: &sqlx::PgPool) -> Result<Vec<Book>, Box<dyn Error>> {
    let query = "SELECT title, author, isbn FROM book";
    let query_result = sqlx::query(query).fetch_all(conn).await?;

    let books: Result<Vec<Book>, Box<dyn Error>> = query_result
        .into_iter()
        .map(|row| {
            let title = row.try_get("title").map_err(Into::<Box<dyn Error>>::into);
            let author = row.try_get("author").map_err(Into::<Box<dyn Error>>::into);
            let isbn = row.try_get("isbn").map_err(Into::<Box<dyn Error>>::into);

            let book = Book {
                title: title?,
                author: author?,
                isbn: isbn?,
            };
            
            Ok(book)
        })
        .collect();

    books
}

// When the application is launched…
fn on_activate(application: &gtk4::Application) {
    // … create a new window …
    let window = gtk4::ApplicationWindow::new(application);
    
    // Set title to the windows
    window.set_title(Some("Bookstore"));

    // Create a vertical box layout to hold the widgets
    let box_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    // Create text
    let label = gtk4::Label::new(Some("Hello, world!"));
    
    // Create input title
    let text_entry_title = gtk4::Entry::new();
    text_entry_title.set_placeholder_text(Some("title"));

    // Create input author
    let text_entry_author = gtk4::Entry::new();
    text_entry_author.set_placeholder_text(Some("Author"));
    
    // Create input isbn
    let text_entry_isbn = gtk4::Entry::new();
    text_entry_isbn.set_placeholder_text(Some("Isbn"));

    // Remove input
    let remove_title = gtk4::Entry::new();
    remove_title.set_placeholder_text(Some("title"));
    let remove_isbn = gtk4::Entry::new(); 
    remove_isbn.set_placeholder_text(Some("Isbn"));

    // Create a button
    let button_add = gtk4::Button::with_label("Enregistrer le livre");
    let button_show = gtk4::Button::with_label("Afficher les livres");
    let button_remove = gtk4::Button::with_label("Retirer le livre");

    button_add.connect_clicked(clone!(@weak text_entry_title, @weak text_entry_author, @weak text_entry_isbn, @weak label => move |_| {
        let text_title = text_entry_title.text();
        let text_author = text_entry_author.text();
        let text_isbn = text_entry_isbn.text();

        if text_title.is_empty() || text_author.is_empty() || text_isbn.is_empty() {
            label.set_text("Error: one or more entries are empty");
        } else {
                text_entry_title.set_text("");
                text_entry_author.set_text("");
                text_entry_isbn.set_text("");

            let book = Book {
                title: text_title.to_string(),
                author: text_author.to_string(),
                isbn: text_isbn.to_string(),
            };

            // Execute the asynchronous function within the GTK main loop
            gtk4::glib::MainContext::default().spawn_local(async move {
                let url = "postgres://dbuser:mysecretpassword@localhost:5432/bookstore";
                let pool = sqlx::postgres::PgPool::connect(url).await.expect("Failed to connect to the database pool");
                match create(&book, &pool).await {
                    Ok(_) => {
                        println!("Entered text: {:?}, {:?}, {:?}", text_title, text_author, text_isbn); // Debug
                        label.set_text("The book successfully entered");
                    }
                    Err(e) => {
                        println!("Error inserting book: {:?}", e);
                    }
                }
            });
        }
    }));
    button_show.connect_clicked( |_| {
        gtk4::glib::MainContext::default().spawn_local(async move {
            let url = "postgres://dbuser:mysecretpassword@localhost:5432/bookstore";
            let pool = sqlx::postgres::PgPool::connect(url).await.expect("Failed to connect to the database pool");

            let books = match read(&pool).await {
                Ok(books) => books,
                Err(err) => {
                    eprintln!("Error reading books: {}", err);
                    return Err(());
                }
            };

            println!("All Books:");
            for (index, book) in books.iter().enumerate() {
                println!("Book {}: {}", index + 1, book.title);
                println!("Author: {}", book.author);
                println!("ISBN: {}", book.isbn);
                println!();
            }

            Ok(())
        });
    });

    button_remove.connect_clicked(clone!(@weak remove_title, @weak remove_isbn => move |_|{
        gtk4::glib::MainContext::default().spawn_local(async move {
            let url = "postgres://dbuser:mysecretpassword@localhost:5432/bookstore";
            let pool = sqlx::postgres::PgPool::connect(url).await.expect("Failed to connect to the database pool");

            let text_title = remove_title.text();
            let text_isbn = remove_isbn.text();

            if text_title.is_empty() && text_isbn.is_empty() {
                println!("You must enter at least one value!");
            } else if !text_title.is_empty() && !text_isbn.is_empty() {
                println!("You cannot remove by both title and ISBN");
            } else if !text_title.is_empty() {
                match remove_book_by_title(&pool, &text_title).await {
                    Ok(_) => {
                        println!("Book removed successfully!");
                        remove_title.set_text("")
                    }
                    Err(e) => {
                        println!("Error removing book: {:?}", e);
                    }
                }
            } else {
                match remove_book_by_isbn(&pool, &text_isbn).await {
                    Ok(_) => {
                        println!("Book removed successfully!");
                        remove_isbn.set_text("")
                    }
                    Err(e) => {
                        println!("Error removing book: {:?}", e);
                    }
                }
            };
        }); 
    }));


    // Set margin to box 
    box_layout.set_margin_start(10);
    box_layout.set_margin_end(10);
    box_layout.set_margin_top(10);
    box_layout.set_margin_bottom(10);

    // Add the widgets to the box layout
    box_layout.append(&label);
    box_layout.append(&text_entry_title);
    box_layout.append(&text_entry_author);
    box_layout.append(&text_entry_isbn);
    box_layout.append(&button_add);
    box_layout.append(&button_show);
    box_layout.append(&remove_title);
    box_layout.append(&remove_isbn);
    box_layout.append(&button_remove);

    // Show box on window
    window.set_child(Some(&box_layout));

    window.present();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "postgres://dbuser:mysecretpassword@localhost:5432/bookstore";
    let pool = sqlx::postgres::PgPool::connect(url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    // Create a new application with the builder pattern
    let app = gtk4::Application::builder()
        .application_id("com.github.gtk-rs.examples.basic")
        .build();
    app.connect_activate(move |app| on_activate(app));

    // Run the application
    app.run();

    let books = read(&pool).await?;

    println!("All Books:");
    for (index, book) in books.iter().enumerate() {
        println!("Book {}: {}", index + 1, book.title);
        println!("Author: {}", book.author);
        println!("ISBN: {}", book.isbn);
        println!();
    }
    Ok(())
}
