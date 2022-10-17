const URL: &'static str = "https://www.bilibili.com/video/";
const UA: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36";

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[derive(Debug)]
pub struct Downloader {
    url: String,
    client: reqwest::Client,
}

impl Downloader {
    pub fn new(bv_or_url: String) -> Self {
        let client = reqwest::Client::builder().user_agent(UA).build().unwrap();
        let url = if bv_or_url.contains(URL) {
            bv_or_url
        } else {
            format!("{}{}", URL, bv_or_url)
        };
        Downloader { url, client }
    }

    pub fn download() -> () {
        todo!()
    }

    pub fn url(&self) -> &str {
        self.url.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn new() {
        let downloader1 = Downloader::new(String::from("test_bv"));
        let downloader2 = Downloader::new(String::from("https://www.bilibili.com/video/test_bv"));
        assert!(downloader1.url().contains(URL));
        assert!(downloader2.url().contains(URL));
    }
}
