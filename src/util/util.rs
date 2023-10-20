use std::process::ExitStatus;

use tokio::process::Command;
use tokio::{
    fs,
    io::{self},
};

/// use ffmpeg to mix video and audio
pub async fn mix_video_audio(
    video_path: &str,
    audio_path: &str,
    output_path: &str,
) -> io::Result<ExitStatus> {
    Command::new("ffmpeg")
        .arg("-i")
        .arg(video_path)
        .arg("-i")
        .arg(audio_path)
        .arg("-c:v")
        .arg("copy")
        .arg("-c:a")
        .arg("aac")
        .arg("-strict")
        .arg("experimental")
        .arg(output_path)
        .status()
        .await
}

/// create file with filepath
pub async fn create_file(filepath: &str) -> io::Result<fs::File> {
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(filepath)
        .await
}
