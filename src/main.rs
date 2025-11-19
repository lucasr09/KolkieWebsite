#[macro_use] extern crate rocket;

use rocket::serde::{json::Json, Deserialize, Serialize};

#[get("/")]
fn index() -> &'static str {
    "Hallo, wereld!"
}

#[get("/hello/<name>")]
fn hello(name: String) -> String {
    format!("Hallo, {}!", name)
}

#[derive(Serialize, Deserialize)]
struct Message {
    content: String,
}

#[post("/message", format = "json", data = "<message>")]
fn send_message(message: Json<Message>) -> String {
    format!("Bericht ontvangen: {}", message.content)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, hello, send_message])
}
