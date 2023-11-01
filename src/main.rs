pub mod question;
pub mod receipt;
pub mod local_store;
pub mod local_store_mem;
pub mod user;
pub mod record;
pub mod request;

use std::str::FromStr;

use warp::Filter;

use crate::question::{Question, QuestionId};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and_then(get_questions);

    let routes = get_items;

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;

    Ok(())
}

async fn get_questions() -> Result<impl warp::Reply, warp::Rejection> {
    let question: Question = Question::new(
        QuestionId::from_str("1").unwrap(),
        "First Question".to_string(),
        "Content of question".to_string(),
        Some(vec!["rust".to_string(), "warp".to_string()]),
    );

    Ok(warp::reply::json(&question))
}
