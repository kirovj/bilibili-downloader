use anyhow::Ok as AnyOk;
use anyhow::Result;
use reqwest::header::{HeaderMap, ACCEPT_RANGES, CONTENT_LENGTH};
use thiserror::Error;

const URL_INFO: &'static str = "https://api.bilibili.com/x/web-interface/view?bvid=";
const URL_PLAY: &'static str = "https://api.bilibili.com/x/player/playurl";
const URL: &'static str = "https://www.bilibili.com/video/";
const UA: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36";

#[derive(Error, Debug)]
pub enum DownloadError<'a> {
    #[error("获取视频信息 {0} 失败")]
    GetVideoInfoFail(&'a str),
}

#[derive(Debug)]
pub struct Video {
    pub bv: String,
    pub cid: String,
    pub url: String,
    pub title: String,
    pub format: String,
    pub support_chunk: bool,
}

#[derive(Debug)]
pub struct Downloader {
    client: reqwest::Client,
}

fn extract_bv(bv_or_url: String) -> String {
    if bv_or_url.contains(URL) {
        todo!()
    } else {
        bv_or_url
    }
}

fn is_support_chunk(headers: &HeaderMap) -> (bool, usize) {
    if let Some(accept) = headers.get(ACCEPT_RANGES) {
        if matches!(accept.to_str(), Ok(value) if value.contains("bytes")) {
            let content_length = headers.get(CONTENT_LENGTH);
            if let Some(length) = content_length {
                let len = length
                    .to_str()
                    .map(|len| len.parse::<usize>().unwrap_or(0))
                    .unwrap_or(0);
                if len > 0 {
                    return (true, len);
                }
            }
        }
    }
    (false, 0)
}

async fn request_json(builder: reqwest::RequestBuilder) -> Result<serde_json::Value> {
    Ok(builder.send().await?.json::<serde_json::Value>().await?)
}

impl Downloader {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder().user_agent(UA).build()?;
        Ok(Downloader { client })
    }

    async fn build_video(&self, bv_or_url: String) -> Result<Video> {
        let bv = extract_bv(bv_or_url);
        let url = format!("{}{}", URL_INFO, bv);
        let response = &request_json(self.client.get(url)).await?["data"];
        let title = response["title"]
            .as_str()
            .unwrap_or_else(|| bv.as_str())
            .to_string();
        let cid = response["cid"]
            .as_u64()
            .ok_or_else(|| DownloadError::GetVideoInfoFail("cid"))?
            .to_string();
        let response = &request_json(
            self.client
                .get(URL_PLAY)
                .query(&[("bvid", bv.as_str()), ("cid", cid.as_str())]),
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
        let chunk_support_info = is_support_chunk(response.headers());
        AnyOk(Video {
            bv,
            cid,
            url,
            title,
            format: String::from("mp4"),
            support_chunk: chunk_support_info.0,
        })
    }

    pub async fn download(&self, bv_or_url: String) -> Result<()> {
        let video = self.build_video(bv_or_url).await;
        match video {
            Err(e) => println!("{:?}", e),
            _ => {
                println!("video: {:#?}", video.unwrap());
            }
        }
        AnyOk(())
    }

    async fn plain_downloader() -> () {
        todo!()
    }

    async fn chunk_download() -> () {
        todo!()
    }

    async fn write_bytes() -> () {
        todo!()
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
