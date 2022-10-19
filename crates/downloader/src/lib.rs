use reqwest::Client;
use anyhow::Result;

const URL: &'static str = "https://www.bilibili.com/video/";
const UA: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36";
const RE_TITLE: &'static str = "<h1 title=\"([^\"]*)\"";

#[derive(Debug)]
pub struct Video {
    pub url: String,
    pub title: Option<String>,
    pub format: Option<String>,
    pub support_chunk: bool,
}

impl Video {
    pub fn new(bv_or_url: String) -> Self {
        let url = if bv_or_url.contains(URL) {
            bv_or_url
        } else {
            format!("{}{}", URL, bv_or_url)
        };
        Video { url, title: None, format: None, support_chunk: false }
    }
}

#[derive(Debug)]
pub struct Downloader {
    video: Video,
    client: reqwest::Client,
}

async fn get_html(client: &Client, url: &str) -> Result<String> {
    let text = client.get(url).send().await?.text().await?;
    Ok(text)
}

fn search_title() -> String {
    todo!()
}

impl Downloader {
    pub fn new(video: Video) -> Result<Self> {
        let client = reqwest::Client::builder().user_agent(UA).build()?;
        Ok(Downloader { video, client })
    }

    pub fn download() -> () {
        todo!()
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
    fn new() {
        let v1 = Video::new(String::from("test_bv"));
        let v2 = Video::new(String::from("https://www.bilibili.com/video/test_bv"));
        assert!(v1.url.contains(URL));
        assert!(v2.url.contains(URL));
    }
}
