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
async fn get_posts() -> Json<Feed> {
    // Initialize SSB data
    let mut sbot_client = Sbot::init(None, None).await.unwrap();
    let ssb_id = sbot_client.whoami().await.unwrap();

    let history_stream = sbot_client.create_history_stream(ssb_id).await.unwrap();

    // Iterate through the elements in the stream and use `map` to convert
    // each `SsbMessageValue` element into a tuple of
    // `(String, SsbMessageContentType)`. This is an example of stream
    // conversion.
    let type_stream = history_stream.map(|msg| match msg {
        Ok(val) => {
            let message_type = val.get_message_type()?;
            let structure = Post {
                author: val.author,
                timestamp: val.timestamp,
                hash: val.hash,
            };
            let tuple: (Post, SsbMessageContentType) = (structure, message_type);
            Ok(tuple)
        }
        Err(err) => Err(err),
    });

    // Pin the stream to the stack to allow polling of the `future`.
    futures::pin_mut!(type_stream);

    println!("looping through type stream");

    let mut posts: Vec<Post> = Vec::new();
    // Iterate through each element in the stream and match on the `Result`.
    // In this case, each element has type
    // `Result<(String, SsbMessageContentType), GolgiError>`.
    while let Some(res) = type_stream.next().await {
        match res {
            Ok(value) => {
                if value.1 == SsbMessageContentType::Post {
                    println!(
                        "author: {}, timestamp: {}, signature: {}",
                        value.0.author, value.0.timestamp, value.0.hash
                    );
                    posts.push(value.0);
                } else {
                    println!("{:?}", value.1);
                }
            }
            Err(err) => {
                println!("err: {:?}", err);
            }
        }
    }

    Json(Feed { posts })
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
