#[macro_use]
extern crate rocket;
use std::sync::Mutex;
// This trait is required when dealing with streams.
use async_std::stream::StreamExt;

use golgi::{
    api::get_subset::{SubsetQuery, SubsetQueryOptions},
    messages::{SsbMessageContent, SsbMessageContentType, SsbMessageValue},
    GolgiError, Sbot,
};
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::serde::{json::Json, Serialize};
use rocket::State;

#[derive(FromForm)]
struct Message<'r> {
    r#message_text: &'r str,
}

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

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Feed {
    posts: Vec<Post>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Post {
    author: String,
    timestamp: f64,
    hash: String,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/posts")]
async fn get_posts() {
    // Initialize SSB data
    let mut sbot_client = Sbot::init(None, None).await.unwrap();

    let post_query = SubsetQuery::Type {
        op: "type".to_string(),
        string: "post".to_string(),
    };

    let post_query_opts = SubsetQueryOptions {
        descending: Some(true),
        keys: None,
        page_limit: Some(5),
    };

    let query_stream = sbot_client
        .get_subset_stream(post_query, Some(post_query_opts))
        .await
        .unwrap();

    // This prints nothing
    query_stream.for_each(|msg| {
        if let Ok(val) = msg {
            println!("{:#?}", val)
        }
    });
}

#[post("/post", data = "<message>")]
async fn new_post(message: Form<Message<'_>>) -> Option<String> {
    // Initialize SSB data
    let mut sbot_client = Sbot::init(None, None).await.unwrap();

    // We can also match on the returned `Result`, rather than using the `?` operator.
    match sbot_client.publish_post(message.message_text).await {
        Ok(post_ref) => Some(post_ref),
        Err(e) => {
            eprintln!("failed to publish post: {}", e);
            return None;
        }
    }
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
        .mount(
            "/api",
            routes![index, whoami, profile_update, get_posts, new_post],
        )
        .mount("/", FileServer::from("static/"))
        .launch()
        .await;
}
