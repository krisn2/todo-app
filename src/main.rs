use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use mongodb::{Client, Collection, bson::doc};
use futures_util::stream::TryStreamExt;
use uuid::Uuid;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Todo {
    id: Option<String>,
    task: String,
}

struct AppState {
    client: Mutex<Client>,
}

async fn get_todos(data: web::Data<AppState>) -> impl Responder {
    let collection: Collection<Todo> = data.client.lock().unwrap()
        .database("todo_app").collection("todos");

    // Find all documents with an empty filter
    let cursor = collection.find(doc! {}).await.unwrap();
    let todos: Vec<Todo> = cursor.try_collect().await.unwrap();

    HttpResponse::Ok().json(todos)
}



async fn add_todo(todo: web::Json<Todo>, data: web::Data<AppState>) -> impl Responder {
    let collection: Collection<Todo> = data.client.lock().unwrap()
        .database("todo_app").collection("todos");

    let mut todo = todo.into_inner();
    todo.id = Some(Uuid::new_v4().to_string());

    collection.insert_one(todo.clone()).await.unwrap();

    HttpResponse::Ok().json(todo)
}

async fn delete_todo(id: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let collection: Collection<Todo> = data.client.lock().unwrap()
        .database("todo_app").collection("todos");

    collection.delete_one(doc! { "id": id.into_inner() }).await.unwrap();

    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Client::with_uri_str("mongodb://localhost:27017").await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                client: Mutex::new(client.clone()),
            }))
            .route("/todos", web::get().to(get_todos))
            .route("/todos", web::post().to(add_todo))
            .route("/todos/{id}", web::delete().to(delete_todo))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
