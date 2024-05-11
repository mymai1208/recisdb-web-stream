use crate::MAX_BUFFER_SIZE;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::mpsc::Sender,
};
use tracing::{debug, info};

pub fn launch(
    driver: &str,
    channel: &str,
    is_encoding: bool,
    sender: Sender<Vec<u8>>,
) -> Result<(), std::io::Error> {
    info!(
        "Launching recisdb with driver: {} and channel: {}",
        driver, channel
    );

    let _driver = driver.to_string();
    let _channel = channel.to_string();

    let mut command = if is_encoding {
        Box::new(
            Command::new("./ffmpeg_pipe.sh")
                .arg(_driver)
                .arg(_channel)
                .stdout(std::process::Stdio::piped())
                .spawn()?,
        )
    } else {
        Box::new(
            Command::new("recisdb")
                .arg("tune")
                .arg("--device")
                .arg(_driver)
                .arg("--no-strip")
                .arg("--channel")
                .arg(_channel)
                .arg("-")
                .stdout(std::process::Stdio::piped())
                .spawn()?,
        )
    };

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

        info!("killing recisdb with PID: {:?}", command.id());

        if command.try_wait().is_err() {
            command.kill().await.unwrap();
        }

        sender.closed().await;
    });

    Ok(())
}
