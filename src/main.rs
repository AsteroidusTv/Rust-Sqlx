use std::error::Error;
use sqlx::Row;

#[derive(Debug)]
struct Book {
    pub title: String,
    pub author: String,
    pub isbn: String,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "postgres://dbuser:mysecretpassword@localhost:5432/bookstore";
    let pool = sqlx::postgres::PgPool::connect(url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

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
