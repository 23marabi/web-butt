#[macro_use]
extern crate rocket;
use std::sync::Mutex;

use golgi::{
    api::get_subset::{SubsetQuery, SubsetQueryOptions},
    messages::SsbMessageContent,
    GolgiError, Sbot,
};
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::serde::{json::Json, Serialize};
use rocket::State;

#[derive(FromForm)]
struct ProfileUpdate<'r> {
    r#username: &'r str,
    r#description: &'r str,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Whoami {
    username: String,
    description: String,
    pubkey: String,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/update", data = "<profileupdate>")]
async fn profile_update(profileupdate: Form<ProfileUpdate<'_>>) -> String {
    // Initialize SSB data
    let mut sbot_client = Sbot::init(None, None).await.unwrap();

    let name_msg_reference = sbot_client
        .publish_name(profileupdate.username)
        .await
        .unwrap();
    let description_msg_reference = sbot_client
        .publish_description(profileupdate.description)
        .await
        .unwrap();

    return "Profile Updated!".to_string();
}

#[get("/whoami")]
async fn whoami() -> Json<Whoami> {
    // Initialize SSB data
    let mut sbot_client = Sbot::init(None, None).await.unwrap();

    let id = sbot_client.whoami().await.unwrap();
    let profile_info = sbot_client.get_profile_info(&id).await.unwrap();
    let name = match profile_info.get("name") {
        Some(s) => s,
        None => {
            println!("Could not get name");
            ""
        }
    };
    let description = match profile_info.get("description") {
        Some(s) => s,
        None => {
            println!("Could not get description");
            ""
        }
    };

    Json(Whoami {
        username: name.to_string(),
        description: description.to_string(),
        pubkey: id,
    })
}

#[rocket::main]
async fn main() {
    let result = rocket::build()
        .mount("/api", routes![index, whoami, profile_update])
        .mount("/", FileServer::from("static/"))
        .launch()
        .await;
}
