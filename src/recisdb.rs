use std::sync::{Arc, Mutex};

use crate::MAX_BUFFER_SIZE;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::mpsc::Sender,
};
use tracing::{debug, info, warn};

pub fn launch(
    driver: String,
    channel: &str,
    is_encoding: bool,
    sender: Sender<Vec<u8>>,
    tuners_arc: &Arc<Mutex<Vec<String>>>,
) -> Result<(), std::io::Error> {
    info!(
        "Launching recisdb with driver: {} and channel: {}",
        driver, channel
    );

    let mut command = if is_encoding {
        Box::new(
            Command::new("./ffmpeg_pipe.sh")
                .arg(&driver)
                .arg(channel)
                .stdout(std::process::Stdio::piped())
                .spawn()?,
        )
    } else {
        Box::new(
            Command::new("recisdb")
                .arg("tune")
                .arg("--device")
                .arg(&driver)
                .arg("--no-strip")
                .arg("--channel")
                .arg(channel)
                .arg("-")
                .stdout(std::process::Stdio::piped())
                .spawn()?,
        )
    };

    let active_tuners_arc: Arc<Mutex<Vec<String>>> = Arc::clone(tuners_arc);

    tokio::spawn(async move {
        info!("Launched recisdb with PID: {:?}", command.id());
        let mut buf: Vec<u8> = Vec::with_capacity(MAX_BUFFER_SIZE);

        info!("Reading from recisdb");

        let mut reader = BufReader::new(command.stdout.take().unwrap());

        while let Ok(length) = reader.read_until(b'-', &mut buf).await {
            if length == 0 {
                break;
            }

            let result = sender.send(buf.clone()).await;

            debug!("Read {} bytes from recisdb", buf.len());

            buf.clear();

            if result.is_err() {
                break;
            }
        }

        active_tuners_arc
            .lock()
            .unwrap()
            .retain(|tuner| !tuner.contains(&driver));

        info!("deleted active tuner: {}", driver);

        info!("killing recisdb with PID: {:?}", command.id());

        sender.closed().await;

        let result = command.kill().await;

        if result.is_err() {
            warn!("Failed to kill recisdb with PID: {:?}", command.id());
        }
    });

    Ok(())
}