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
    
    // Create input name
    let text_entry_name = gtk4::Entry::new();
    text_entry_name.set_placeholder_text(Some("Name"));

    // Create input author
    let text_entry_author = gtk4::Entry::new();
    text_entry_author.set_placeholder_text(Some("Author"));
    
    // Create input isbn
    let text_entry_isbn = gtk4::Entry::new();
    text_entry_isbn.set_placeholder_text(Some("Isbn"));

    // Create a button
    let button = gtk4::Button::with_label("Enregistrer le livre");

    button.connect_clicked(clone!(@weak text_entry_name, @weak text_entry_author, @weak text_entry_isbn, @weak label => move |_| {
        let text_name = text_entry_name.text();
        let text_author = text_entry_author.text();
        let text_isbn = text_entry_isbn.text();

        if text_name.is_empty() || text_author.is_empty() || text_isbn.is_empty() {
            label.set_text("Error: one or more entries are empty");
        } else {
            println!("Entered text: {:?}, {:?}, {:?}", text_name, text_author, text_isbn);
            label.set_text("Hello, moon!");

            let book = Book {
                title: text_name.to_string(),
                author: text_author.to_string(),
                isbn: text_isbn.to_string(),
            };

            // Execute the asynchronous function within the GTK main loop
            gtk4::glib::MainContext::default().spawn_local(async move {
                let url = "postgres://dbuser:mysecretpassword@localhost:5432/bookstore";
                let pool = sqlx::postgres::PgPool::connect(url).await.expect("Failed to connect to the database pool");

                match create(&book, &pool).await {
                    Ok(_) => {
                        println!("Book inserted successfully!");
                    }
                    Err(e) => {
                        println!("Error inserting book: {:?}", e);
                    }
                }
            });
        }
    }));

    // Set margin to box 
    box_layout.set_margin_start(10);
    box_layout.set_margin_end(10);
    box_layout.set_margin_top(10);
    box_layout.set_margin_bottom(10);

    // Add the widgets to the box layout
    box_layout.append(&label);
    box_layout.append(&text_entry_name);
    box_layout.append(&text_entry_author);
    box_layout.append(&text_entry_isbn);
    box_layout.append(&button);

    // Show box on window
    window.set_child(Some(&box_layout));

    window.present();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "postgres://dbuser:mysecretpassword@localhost:5432/bookstore";
    let pool = sqlx::postgres::PgPool::connect(url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let books = read(&pool).await?;

    

    // Create a new application with the builder pattern
    let app = gtk4::Application::builder()
        .application_id("com.github.gtk-rs.examples.basic")
        .build();
    app.connect_activate(move |app| on_activate(app));

    // Run the application
    app.run();

    println!("All Books:");
    for (index, book) in books.iter().enumerate() {
        println!("Book {}: {}", index + 1, book.title);
        println!("Author: {}", book.author);
        println!("ISBN: {}", book.isbn);
        println!();
    }
    Ok(())
}
