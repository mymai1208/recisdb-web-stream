use std::{
    io::Result,
    sync::{Arc, Mutex},
};

use axum::{
    body::{Body, HttpBody}, extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::get, Json, Router
};
use futures::StreamExt;
use serde::Serialize;
use tokio::{
    net::TcpListener,
    sync::mpsc::{self, Receiver, Sender},
};
use tokio_stream::wrappers::ReceiverStream;
use tracing::info;

use crate::{recisdb, CONFIG, MAX_BUFFER_SIZE};

#[derive(Debug, Clone, PartialEq, Serialize)]
struct ChannelJson {
    name: String,
    #[serde(rename = "channelId")]
    channel_id: String,
    #[serde(rename = "type")]
    r#type: String,
}

pub async fn init_server() {
    let tuners_arc = Arc::new(Mutex::new(vec![]));

    let app: Router = Router::new()
        .route("/stream/:channel_id", get(stream_handler))
        .with_state(tuners_arc)
        .route("/channels", get(get_channels_handler));
    let listener = TcpListener::bind("0.0.0.0:4000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn stream_handler(
    Path(channel_id): Path<String>,
    State(tuner_arc): State<Arc<Mutex<Vec<String>>>>,
) -> impl IntoResponse {
    let config = CONFIG.lock().unwrap();

    let channel = config
        .channels
        .iter()
        .filter(|channel| channel.channel == channel_id)
        .take(1)
        .next();

    if channel.is_none() {
        info!("Channel not found");
        let mut response = Body::new("Channel not found".to_string()).into_response();

        *response.status_mut() = StatusCode::NOT_FOUND;

        return response;
    }

    let (sender, receiver): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel(MAX_BUFFER_SIZE);

    let mut active_tuners = tuner_arc.lock().unwrap();

    let driver = config
        .tuners
        .iter()
        .filter(|tuner| tuner.types.contains(&channel.unwrap().r#type))
        .find(|tuner| !active_tuners.contains(&tuner.name));

    if driver.is_none() {
        let mut response = Body::new("Driver not found".to_string()).into_response();

        *response.status_mut() = StatusCode::NOT_FOUND;

        info!("Driver is full");

        return response;
    }

    let args: Vec<&str> = driver.unwrap().command.split_ascii_whitespace().collect();

    if recisdb::launch(args[5].to_string(), &channel_id, false, sender, &tuner_arc).is_err() {
        return Body::new("Failed to launch recisdb".to_string()).into_response();
    }

    active_tuners.push(args[5].to_string());

    let stream = ReceiverStream::new(receiver).map(Result::Ok);

    let stream_body = Body::from_stream(stream);

    if !stream_body.is_end_stream() {
        return stream_body.into_response();
    }

    return Body::new("Stream ended".to_string()).into_response();
}

async fn get_channels_handler() -> Json<Vec<ChannelJson>> {
    let config = CONFIG.lock().unwrap();

    return Json(
        config
            .channels
            .iter()
            .map(|json| ChannelJson {
                name: json.name.trim_start_matches('\u{feff}').to_owned(),
                channel_id: json.channel.to_owned(),
                r#type: json.r#type.to_owned(),
            })
            .collect(),
    );
}
