

use actix_web::{get, App, HttpResponse, HttpServer, ResponseError, web::{self, Data}, http::header, post};
use askama::Template;
use serde::Deserialize;
use thiserror::Error;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

#[derive(Deserialize)]
struct AddParams {
    text: String,
}

struct TodoEntry {
    id: u32,
    text: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    entries: Vec<TodoEntry>,
}

#[derive(Error, Debug)]
enum MyError {
    #[error("Failed to render HTML")]
    AskamaError(#[from] askama::Error),

    #[error("Failed to get connection")]
    ConnectionPoolError(#[from] r2d2::Error),

    #[error("Failed SQL execution")]
    SQLiteError(#[from] rusqlite::Error),
}

impl ResponseError for MyError {}

#[post("/add")]
async fn add_todo(params: web::Form<AddParams>, db: web::Data<r2d2::Pool<SqliteConnectionManager>>) -> Result<HttpResponse, MyError> {
    let conn = db.get()?;
    conn.execute("INSERT INTO todo (text) VALUES (?)", &[&params.text])?;
    Ok(
        HttpResponse::SeeOther()
            .append_header((header::LOCATION, "/"))
            .finish()
    )
}

#[get("/")]
async fn index(db: web::Data<Pool<SqliteConnectionManager>>) -> Result<HttpResponse, MyError> {
    let conn = db.get()?;
    let mut statement = conn.prepare("SELECT id, text FROM todo;")?;
    let rows = statement.query_map(params![], |row| {
        let id = row.get(0)?;
        let text = row.get(1)?;
        Ok(TodoEntry { id, text })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }

    let html = IndexTemplate { entries };
    let response_body = html.render()?;

    Ok(HttpResponse::Ok().content_type("text/html").body(response_body))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let manager = SqliteConnectionManager::file("todo.db");
    let pool = Pool::new(manager).expect("Failed to initialize the connection pool.");
    let conn = pool.get().expect("Failed to get the connection from the pool.");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS todo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL
        );"
        , params![]
    )
        .expect("Failed to create a table `todo`.");
    
    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(add_todo)
            .app_data(Data::new(pool.clone()))
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
