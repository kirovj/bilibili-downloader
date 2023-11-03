use std::sync::Arc;

use anyhow::Result;
use futures::future::join_all;
use prost::Message;
use reqwest::header::{self, CONTENT_RANGE, CONTENT_TYPE};
use reqwest::header::{HeaderMap, ACCEPT_RANGES, CONTENT_LENGTH};
use thiserror::Error;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{create_file, mix_video_audio};

use super::{DanmakuSegment, Video};

const API_INFO: &'static str = "https://api.bilibili.com/x/web-interface/view?bvid=";
const API_PLAY: &'static str = "https://api.bilibili.com/x/player/playurl";
const API_BULLET: &'static str = "http://api.bilibili.com/x/v2/dm/web/seg.so";
const API_USERINFO: &'static str = "https://api.bilibili.com/x/web-interface/nav";
const UA: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36";

#[derive(Error, Debug)]
pub enum DownloadError<'a> {
    #[error("登陆失败")]
    LoginFail,
    #[error("获取视频信息 {0} 失败")]
    GetVideoInfoFail(&'a str),
    #[error("获取弹幕包失败 cid: {0}, segment: {1}")]
    GetDanmakuSegmentFail(u64, u64),
}

#[derive(Debug, Clone)]
pub struct Downloader {
    client: reqwest::Client,
    task_num: u8,
    pub dir: String,
}

fn extract_content_range(headers: &HeaderMap) -> Result<u64> {
    if let Some(range) = headers.get(CONTENT_RANGE) {
        let _range = String::from(range.to_str()?);
        let mut splits = _range.split("/");
        let _ = splits.next();
        if let Some(len) = splits.next() {
            return Ok(len.parse::<u64>()?);
        }
    }
    Ok(0 as u64)
}

fn extract_content_len(headers: &HeaderMap) -> Result<u64> {
    let check_accept = match headers.get(ACCEPT_RANGES) {
        Some(accept_ranges) => accept_ranges.to_str()?.contains("bytes"),
        None => false,
    };

    let content_length = if check_accept {
        match headers.get(CONTENT_LENGTH) {
            Some(lenth) => lenth.to_str()?.parse::<usize>()?,
            None => 0,
        }
    } else {
        0
    };
    Ok(content_length as u64)
}

fn extract_format(content_type: &str) -> String {
    match content_type {
        "video/mp4" => "mp4",
        "video/x-flv" => "flv",
        "application/x-mpegURL" => "m3u8",
        "video/MP2T" => "ts",
        "video/3gpp" => "3gpp",
        "video/quicktime" => "mov",
        "video/x-msvideo" => "avi",
        "video/x-ms-wmv" => "wmv",
        "audio/x-wav" => "wav",
        "audio/x-mp3" => "mp3",
        "audio/mp4" => "mp4",
        "application/ogg" => "ogg",
        "image/jpeg" => "jpeg",
        "image/png" => "png",
        "image/tiff" => "tiff",
        "image/gif" => "gif",
        "image/svg+xml" => "svg",
        _ => "mp4",
    }
    .to_string()
}

fn extract_format_from_header(headers: &HeaderMap) -> String {
    match headers.get(CONTENT_TYPE) {
        Some(content_type) => extract_format(content_type.to_str().unwrap_or("")),
        None => "mp4".to_string(),
    }
}

#[cfg(target_family = "windows")]
async fn write_bytes_to_file(
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

#[cfg(target_family = "unix")]
async fn write_bytes_to_file(
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

async fn request_json(builder: reqwest::RequestBuilder) -> Result<serde_json::Value> {
    Ok(builder.send().await?.json::<serde_json::Value>().await?)
}

impl Downloader {
    pub fn new(task_num: u8) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "referer",
            header::HeaderValue::from_static("https://www.bilibili.com/"),
        );
        if let Ok(c) = std::fs::read_to_string("cookie") {
            let val = header::HeaderValue::from_str(c.as_str())?;
            headers.insert("cookie", val);
        }
        let client = reqwest::Client::builder()
            .user_agent(UA)
            .default_headers(headers)
            .build()?;
        let dir = "".to_string();
        let downloader = Downloader {
            client,
            task_num,
            dir,
        };
        Ok(downloader)
    }

    /// Validate cookie
    pub async fn check_login(&self) -> Result<()> {
        let r = &request_json(self.client.get(API_USERINFO)).await?;
        match r["data"]["isLogin"].as_bool() {
            Some(true) => {
                println!("Login success.");
                Ok(())
            }
            _ => {
                println!("Login fail.");
                Err(DownloadError::LoginFail.into())
            }
        }
    }

    /// Buile video
    pub async fn build_video(&self, bv: String) -> Result<Video> {
        self.check_login().await?;
        let url = format!("{}{}", API_INFO, bv);
        println!("video info: {}", url);
        let response = &request_json(self.client.get(url)).await?["data"];
        let title = response["title"]
            .as_str()
            .unwrap_or_else(|| bv.as_str())
            .to_string();
        let cid = response["cid"]
            .as_u64()
            .ok_or_else(|| DownloadError::GetVideoInfoFail("cid"))?;
        let duration = response["duration"]
            .as_u64()
            .ok_or_else(|| DownloadError::GetVideoInfoFail("duration"))?;
        println!("video play info: {}?bvid={}&cid={}", API_PLAY, bv, cid);
        let response = &request_json(self.client.get(API_PLAY).query(&[
            ("bvid", bv.as_str()),
            ("cid", cid.to_string().as_str()),
            // add this to fetch 1080P or better
            ("fnval", "2000"),
        ]))
        .await?["data"];
        let video_url;
        let audio_url;
        let format;
        let content_len;

        if let Some(data) = response["dash"].as_object() {
            let video_data = &data
                .get("video")
                .ok_or_else(|| DownloadError::GetVideoInfoFail("url"))?[0];
            video_url = video_data["baseUrl"].as_str().unwrap_or("").to_string();
            audio_url = data
                .get("audio")
                .ok_or_else(|| DownloadError::GetVideoInfoFail("url"))?[0]["baseUrl"]
                .as_str()
                .unwrap_or("")
                .to_string();
            format = extract_format(video_data["mimeType"].as_str().unwrap_or(""));
            let response = self
                .client
                .get(&video_url)
                .header("range", "bytes=0-1024")
                .send()
                .await?;
            content_len = extract_content_range(response.headers()).unwrap_or(0);
        } else {
            video_url = response["durl"]
                .as_array()
                .ok_or_else(|| DownloadError::GetVideoInfoFail("durl"))?[0]["url"]
                .as_str()
                .ok_or_else(|| DownloadError::GetVideoInfoFail("url"))?
                .to_string();
            audio_url = String::new();
            let response = self.client.head(&video_url).send().await?;
            format = extract_format_from_header(response.headers());
            content_len = extract_content_len(response.headers()).unwrap_or(0);
        }
        println!("video format: {}, content length: {}", format, content_len);
        Ok(Video {
            bv,
            cid,
            video_url,
            audio_url,
            title,
            format,
            duration,
            content_len,
        })
    }

    /// Download video chunks
    pub async fn download_chunks(self: Arc<Self>, video: Arc<Video>) -> Result<()> {
        let chunk_size = video.content_len / self.task_num as u64;
        println!("downloading video, chunk size {}...", chunk_size);
        let mut range_list = vec![];
        let mut start = 0;
        let mut end = 0;

        while end < video.content_len {
            end += chunk_size;
            if end > video.content_len {
                end = video.content_len;
            }
            range_list.push((start, end));
            start = end + 1;
        }
        let mut handler_list = vec![];
        for (index, range) in range_list.into_iter().enumerate() {
            let downloader = self.clone();
            let video = video.clone();
            let handler =
                tokio::spawn(
                    async move { downloader.download_chunk(video, range, index as u8).await },
                );
            handler_list.push(handler);
        }
        join_all(handler_list).await;
        Ok(())
    }

    /// Download single video chunk
    pub async fn download_chunk(
        self: Arc<Self>,
        video: Arc<Video>,
        range: (u64, u64),
        index: u8,
    ) -> Result<()> {
        println!("download chunk {} from {} to {}", index, range.0, range.1);
        let mut response = self
            .client
            .get(video.video_url.as_str())
            .header("range", format!("bytes={}-{}", range.0, range.1))
            .send()
            .await?;
        let mut offset = 0;
        while let Some(bytes) = response.chunk().await? {
            let bv = video.bv.as_str();
            let filepath = format!("{}/{}_{}", self.dir, bv, index);
            let len = bytes.len() as u64;
            write_bytes_to_file(filepath.as_str(), &bytes, offset).await?;
            offset += len;
        }
        Ok(())
    }

    /// Download audio if exists
    pub async fn download_audio(self: Arc<Self>, video: Arc<Video>) -> Result<()> {
        if video.audio_url.len() == 0 {
            return Ok(());
        }
        println!("downloading audio...");
        let response = self.client.get(video.audio_url.as_str()).send().await?;
        if let Ok(bytes) = response.bytes().await {
            let filepath = format!("{}/audio.mp3", self.dir);
            write_bytes_to_file(filepath.as_str(), &bytes, 0).await?;
            println!("downloading audio success at {}", filepath);
        }
        Ok(())
    }

    /// Merge chunk files and mix video and audio
    pub async fn build_final_video(self: Arc<Self>, video: Arc<Video>) -> Result<()> {
        let format = video.format.as_str();
        let video_path = format!("{}/video.{}", self.dir, format);
        let mut file = create_file(video_path.as_str()).await?;
        let indexes = self.task_num + 1;

        for index in 0..indexes {
            let chunk_path = format!("{}/{}_{}", self.dir, video.bv.as_str(), index);
            let mut chunk_file = fs::File::open(chunk_path.as_str()).await?;
            println!("merge chunk file {}", index);
            let size = chunk_file.metadata().await?.len();
            let mut buf = vec![0; size as usize];
            chunk_file.read_exact(&mut buf).await?;
            file.write_all(&buf).await?;
            fs::remove_file(chunk_path).await?;
        }
        println!("merge chunk files finished");
        let _ = mix_video_audio(
            format!("{}/video.{}", self.dir, format).as_str(),
            format!("{}/audio.mp3", self.dir).as_str(),
            format!("{}/{}.{}", self.dir, video.title.as_str(), format).as_str(),
        )
        .await?;
        Ok(())
    }

    /// Download danmaku
    pub async fn download_danmaku(self: Arc<Self>, video: Arc<Video>) -> Result<()> {
        println!("downloading danmaku...");
        let mut file =
            create_file(format!("{}/{}_danmuku.txt", self.dir, video.bv).as_str()).await?;
        let mut fut = vec![];
        let bags = (video.duration + 359) / 360;
        for i in 0..bags {
            let downloader = self.clone();
            let video = video.clone();
            fut.push(downloader.download_danmaku_segment(video, i + 1));
        }
        let danmuku_list = futures::future::try_join_all(fut).await?;
        for danmuku_seg in danmuku_list {
            for danmuku in danmuku_seg.elems {
                if let Ok(json) = serde_json::to_string(&danmuku) {
                    file.write_all(format!("{}\n", json).as_bytes()).await?;
                }
            }
        }
        Ok(())
    }

    /// Download danmaku segment
    pub async fn download_danmaku_segment(
        self: Arc<Self>,
        video: Arc<Video>,
        seg_index: u64,
    ) -> Result<DanmakuSegment> {
        let response = self
            .client
            .get(API_BULLET)
            .query(&[
                ("oid", video.cid),
                ("segment_index", seg_index),
                ("type", 1),
            ])
            .send()
            .await?;
        let content = response.bytes().await?;
        match super::DanmakuSegment::decode(content) {
            Ok(v) => Ok(v),
            Err(_) => Err(DownloadError::GetDanmakuSegmentFail(video.cid, seg_index).into()),
        }
    }

    /// Start download mission
    pub async fn start(self: Arc<Self>, video: Arc<Video>) -> Result<()> {
        self.clone().download_chunks(video.clone()).await?;
        self.clone().download_audio(video.clone()).await?;
        self.clone().build_final_video(video.clone()).await?;
        self.clone().download_danmaku(video).await?;
        Ok(())
    }
}
