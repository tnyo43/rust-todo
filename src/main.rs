use actix_web::{get, App, Responder, HttpResponse, HttpServer};

#[get("/")]
async fn index() -> impl Responder {
    let response_body = "Hello, World!";

    HttpResponse::Ok().body(response_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
