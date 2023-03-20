use std::sync::Arc;

use anyhow::Result;
use futures::future::join_all;
use prost::Message;
use reqwest::cookie::Jar;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::{HeaderMap, ACCEPT_RANGES, CONTENT_LENGTH};
use thiserror::Error;

use super::{DanmakuSegment, Video};

const API_INFO: &'static str = "https://api.bilibili.com/x/web-interface/view?bvid=";
const API_PLAY: &'static str = "https://api.bilibili.com/x/player/playurl";
const API_BULLET: &'static str = "http://api.bilibili.com/x/v2/dm/web/seg.so";
const API_USERINFO: &'static str = "https://api.bilibili.com/x/web-interface/nav";
const URL: &'static str = "https://www.bilibili.com/video/";
const UA: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36";

#[derive(Error, Debug)]
pub enum DownloadError<'a> {
    #[error("获取视频信息 {0} 失败")]
    GetVideoInfoFail(&'a str),
    #[error("获取弹幕包失败 cid: {0}, segment: {1}")]
    GetDanmakuSegmentFail(u64, u64),
}

#[derive(Debug, Clone)]
pub struct Downloader {
    client: reqwest::Client,
    task_num: u8,
}

fn extract_bv(bv_or_url: String) -> String {
    if bv_or_url.contains(URL) {
        todo!()
    } else {
        bv_or_url
    }
}

fn extract_content_lenth(headers: &HeaderMap) -> Result<u64> {
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

fn extract_format(headers: &HeaderMap) -> String {
    return match headers.get(CONTENT_TYPE) {
        Some(content_type) => match content_type.to_str().unwrap_or("video/mp4") {
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
        },
        None => "mp4",
    }
    .to_string();
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

fn build_cookie_jar() -> Option<Arc<Jar>> {
    if let Ok(cookie_str) = std::fs::read_to_string("cookie.txt") {
        let url = "https://bilibli.com".parse::<reqwest::Url>().unwrap();
        let jar = Jar::default();
        jar.add_cookie_str(cookie_str.as_str(), &url);
        return Some(Arc::new(jar));
    }
    None
}

impl Downloader {
    pub fn new() -> Result<Self> {
        let mut has_cookies = false;
        let client = if let Some(cookies) = build_cookie_jar() {
            has_cookies = true;
            reqwest::Client::builder()
                .user_agent(UA)
                .cookie_provider(Arc::clone(&cookies))
                .cookie_store(true)
                .build()?
        } else {
            reqwest::Client::builder().user_agent(UA).build()?
        };
        let downloader = Downloader {
            client,
            task_num: 8,
        };
        if has_cookies {
            downloader.check_login()?
        }
        Ok(downloader)
    }

    fn check_login(&self) -> Result<()> {
        Ok(())
    }

    pub async fn build_video(&self, bv_or_url: String) -> Result<Video> {
        let bv = extract_bv(bv_or_url);
        let url = format!("{}{}", API_INFO, bv);
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
        let response = &request_json(
            self.client
                .get(API_PLAY)
                .query(&[("bvid", bv.as_str()), ("cid", cid.to_string().as_str())]),
        )
        .await?["data"];
        let url = response["durl"]
            .as_array()
            .ok_or_else(|| DownloadError::GetVideoInfoFail("durl"))?[0]["url"]
            .as_str()
            .ok_or_else(|| DownloadError::GetVideoInfoFail("url"))?
            .to_string();
        let response = self
            .client
            .head(&url)
            .header("referer", "https://www.bilibili.com/")
            .send()
            .await?;
        let format = extract_format(response.headers());
        let content_lenth = extract_content_lenth(response.headers()).unwrap_or(0);
        Ok(Video {
            bv,
            cid,
            url,
            title,
            format,
            duration,
            content_lenth,
        })
    }

    pub async fn download_chunks(self: Arc<Self>, video: Arc<Video>) -> Result<()> {
        let chunk_size = video.content_lenth / self.task_num as u64;
        let mut range_list = vec![];
        let mut start = 0;
        let mut end = 0;

        while end < video.content_lenth {
            end += chunk_size;
            if end > video.content_lenth {
                end = video.content_lenth;
            }
            range_list.push((start, end));
            start = end + 1;
        }

        let mut handler_list = vec![];
        for (index, range) in range_list.into_iter().enumerate() {
            println!("download chunk {} from {} to {}", index, range.0, range.1);
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

    pub async fn download_chunk(
        self: Arc<Self>,
        video: Arc<Video>,
        range: (u64, u64),
        index: u8,
    ) -> Result<()> {
        let mut response = self
            .client
            .get(video.url.as_str())
            .header("referer", "https://www.bilibili.com/")
            .header("range", format!("bytes={}-{}", range.0, range.1))
            .send()
            .await?;
        let mut offset = 0;
        while let Some(bytes) = response.chunk().await? {
            let bv = video.bv.as_str();
            let filepath = format!("{}/{}_{}", bv, bv, index);
            let len = bytes.len() as u64;
            write_bytes_to_file(filepath.as_str(), &bytes, offset).await?;
            offset += len;
        }
        Ok(())
    }

    pub async fn download_danmaku(
        self: Arc<Self>,
        video: Arc<Video>,
    ) -> Result<Vec<DanmakuSegment>> {
        let mut fut = vec![];
        let bags = (video.duration + 359) / 360;
        for i in 0..bags {
            let downloader = self.clone();
            let video = video.clone();
            fut.push(downloader.download_danmaku_segment(video, i + 1));
        }
        futures::future::try_join_all(fut).await
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bv() {
        assert_eq!("xxx", extract_bv("xxx".to_string()));
        assert_eq!(
            "xxx",
            extract_bv("https://www.bilibili.com/video/xxx".to_string())
        );
    }
}
