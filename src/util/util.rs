use std::process::ExitStatus;

use tokio::process::Command;
use tokio::{
    fs,
    io::{self},
};

/// Use ffmpeg to mix video and audio
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

/// Create file with filepath
pub async fn create_file(filepath: &str) -> io::Result<fs::File> {
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(filepath)
        .await
}

/// Write bytes to file in windows
#[cfg(target_family = "windows")]
pub async fn write_bytes_to_file(
    filepath: &str,
    bytes: &[u8],
    offset: u64,
) -> Result<usize, std::io::Error> {
    use std::fs;
    use std::os::windows::fs::FileExt;
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(filepath)?;
    file.seek_write(&bytes, offset)
}

/// Write bytes to file in unix
#[cfg(target_family = "unix")]
pub async fn write_bytes_to_file(
    filepath: &str,
    bytes: &[u8],
    offset: u64,
) -> Result<usize, std::io::Error> {
    use std::fs;
    use std::os::unix::fs::FileExt;
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(filepath)?;
    file.write_at(bytes, offset)
}
