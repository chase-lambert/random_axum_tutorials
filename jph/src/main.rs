mod book;
mod data;

use crate::book::Book;
use crate::data::DATA;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::thread;

use axum::http::{StatusCode, Uri};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{extract, http, response};
use base64::{engine::general_purpose, Engine as _};
use serde_json::{json, Value};
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// async fn print_data() {
//     thread::spawn(move || {
//         let data = DATA.lock().unwrap();
//         println!("data: {data:?}");
//     })
//     .join()
//     .unwrap()
// }

#[tokio::main]
async fn main() {
    // print_data().await;

    // tracing_subscriber::registry()
    //     .with(tracing_subscriber::fmt::layer())
    //     .init();

    let app = axum::Router::new()
        .route("/", get(hello))
        .route("/demo.html", get(get_demo_html))
        .route("/hello.html", get(hello_html))
        .route("/demo-status", get(demo_status))
        .route("/demo-uri", get(demo_uri))
        .route("/demo.png", get(get_demo_png))
        .route(
            "/foo",
            get(get_foo)
                .put(put_foo)
                .patch(patch_foo)
                .post(post_foo)
                .delete(delete_foo),
        )
        .route("/items/:id", get(get_items_id))
        .route("/items", get(get_items))
        .route("/demo.json", get(get_demo_json).put(put_demo_json))
        .route("/books", get(get_books).put(put_books))
        .route("/books/:id", get(get_books_id).delete(delete_books_id))
        .route(
            "/books/:id/form",
            get(get_books_id_form).post(post_books_id_form),
        )
        .fallback(fallback);

    let host = [127, 0, 0, 1];
    let port = 3000;
    let addr = SocketAddr::from((host, port));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        // .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

// async fn shutdown_signal() {
//     tokio::signal::ctrl_c()
//         .await
//         .expect("expect tokio signal ctrl-c");
//     println!("signal shutdown");
// }

pub async fn fallback(uri: Uri) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, format!("No route {}", uri))
}

pub async fn hello() -> String {
    "Hello, World!".into()
}

pub async fn get_demo_html() -> Html<&'static str> {
    "<h1>Hello</h1>".into()
}

async fn hello_html() -> Html<&'static str> {
    include_str!("hello.html").into()
}

pub async fn demo_status() -> (StatusCode, String) {
    (StatusCode::OK, "Everything is OK".to_string())
}

pub async fn demo_uri(uri: Uri) -> String {
    format!("The URI is: {uri:?}")
}

async fn get_demo_png() -> impl IntoResponse {
    let png = concat!(
        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB",
        "CAYAAAAfFcSJAAAADUlEQVR42mPk+89Q",
        "DwADvgGOSHzRgAAAAABJRU5ErkJggg=="
    );

    (
        response::AppendHeaders([(http::header::CONTENT_TYPE, "image/png")]),
        general_purpose::STANDARD.decode(png).unwrap(),
        // base64::decode(png).unwrap(),
    )
}

pub async fn get_foo() -> String {
    "GET foo".to_string()
}

pub async fn put_foo() -> String {
    "PUT foo".to_string()
}

pub async fn patch_foo() -> String {
    "PATCH foo".to_string()
}

pub async fn post_foo() -> String {
    "POST foo".to_string()
}

pub async fn delete_foo() -> String {
    "DELETE foo".to_string()
}

pub async fn get_items_id(extract::Path(id): extract::Path<String>) -> String {
    format!("Get items with path id: {:?}", id)
}

pub async fn get_items(extract::Query(params): extract::Query<HashMap<String, String>>) -> String {
    format!("Get items with query params: {params:?}")
}

pub async fn get_demo_json() -> extract::Json<Value> {
    json!({"a": "b"}).into()
}

pub async fn put_demo_json(extract::Json(data): extract::Json<serde_json::Value>) -> String {
    format!("Put demo JSON data: {data:?}")
}

pub async fn get_books() -> Html<String> {
    thread::spawn(move || {
        let data = DATA.lock().unwrap();
        let mut books = data.values().collect::<Vec<_>>().clone();
        books.sort_by(|a, b| a.title.cmp(&b.title));
        books
            .iter()
            .map(|&book| format!("<p>{}</p>\n", &book))
            .collect::<String>()
    })
    .join()
    .unwrap()
    .into()
}

pub async fn get_books_id(extract::Path(id): extract::Path<u32>) -> Html<String> {
    thread::spawn(move || {
        let data = DATA.lock().unwrap();
        match data.get(&id) {
            Some(book) => format!("<p>{}</p>\n", &book),
            None => format!("<p>Book id {} not found</p>", id),
        }
    })
    .join()
    .unwrap()
    .into()
}

pub async fn put_books(extract::Json(book): extract::Json<Book>) -> Html<String> {
    thread::spawn(move || {
        let mut data = DATA.lock().unwrap();
        data.insert(book.id, book.clone());
        format!("Put book: {}", &book)
    })
    .join()
    .unwrap()
    .into()
}

pub async fn get_books_id_form(extract::Path(id): extract::Path<u32>) -> Html<String> {
    thread::spawn(move || {
        let data = DATA.lock().unwrap();
        match data.get(&id) {
            Some(book) => format!(
                concat!(
                    "<form method=\"post\" action=\"/books/{}/form\">\n",
                    "<input type=\"hidden\" name=\"id\" value=\"{}\">\n",
                    "<p><input name=\"title\" value=\"{}\"></p>\n",
                    "<p><input name=\"author\" value=\"{}\"></p>\n",
                    "<input type=\"submit\" value=\"Save\">\n",
                    "</form>\n"
                ),
                &book.id, &book.id, &book.title, &book.author
            ),
            None => format!("<p>Book id {} not found</p>", id),
        }
    })
    .join()
    .unwrap()
    .into()
}

pub async fn post_books_id_form(form: extract::Form<Book>) -> Html<String> {
    let new_book: Book = form.0;
    thread::spawn(move || {
        let mut data = DATA.lock().unwrap();
        if data.contains_key(&new_book.id) {
            data.insert(new_book.id, new_book.clone());
            format!("<p>{}</p>\n", &new_book)
        } else {
            format!("Book id not found: {}", &new_book.id)
        }
    })
    .join()
    .unwrap()
    .into()
}

pub async fn delete_books_id(extract::Path(id): extract::Path<u32>) -> Html<String> {
    thread::spawn(move || {
        let mut data = DATA.lock().unwrap();
        if data.contains_key(&id) {
            data.remove(&id);
            format!("Delete book id: {}", &id)
        } else {
            format!("Book id not found: {}", &id)
        }
    })
    .join()
    .unwrap()
    .into()
}
