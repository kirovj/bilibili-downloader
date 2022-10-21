use std::fs;
use std::sync::Arc;

use downloader::{Downloader, Video};
use futures::future::join_all;

const TASK_NUM: u8 = 8;

async fn download_chunks(downloader: Arc<Downloader>, video: Arc<Video>) {
    let chunk_size = video.content_lenth / TASK_NUM as u64;
    let mut range_list = vec![];
    let mut start = 0;
    let mut end = 0;

    while end <= video.content_lenth {
        end += chunk_size;
        range_list.push((start, end));
        start = end + 1;
    }

    let mut handler_list = vec![];

    for (index, range) in range_list.into_iter().enumerate() {
        println!("download chunk {} from {} to {}", index, range.0, range.1);
        let downloader = downloader.clone();
        let video = video.clone();
        let handler =
            tokio::spawn(async move { downloader.download_chunk(video, range, index as u8).await });
        handler_list.push(handler);
    }
    join_all(handler_list).await;
}

async fn download_entry(bv: &str) -> bool {
    let downloader = Arc::new(Downloader::new().unwrap());

    let video = downloader
        .build_video(bv.to_string())
        .await
        .map_or_else(|e| panic!("{}", e), |v| Arc::new(v));

    match fs::create_dir(bv) {
        Ok(_) => {
            println!("download {} start, title: `{}`", video.bv, video.title);
            if video.content_lenth > 0 {
                download_chunks(downloader, video).await;
            } else {
                todo!()
            }
        }
        Err(_) => panic!("create video dir failed"),
    }
    true
}

#[tokio::main]
async fn main() -> () {
    let bv = "BV1Q14y1L76r";
    let _ = download_entry(bv).await;

    // let matches = clap::App::new("Bilibili Video Downloader")
    //     .version(clap::crate_version!())
    //     .author("Kirovj")
    //     .about("Don't use it illegally, I don't take any responsibility.")
    //     .arg(
    //         clap::Arg::with_name("target")
    //             .short("t")
    //             .long("target")
    //             .help("Video bid")
    //             .required(true)
    //             .takes_value(true),
    //     )
    //     .arg(
    //         clap::Arg::with_name("bullet")
    //             .short("b")
    //             .long("bullet")
    //             .help("Need bullet comment default false"),
    //     )
    //     .get_matches();

    // let bullet = matches.is_present("bullet");

    // if let Some(bid) = matches.value_of("target") {
    //     download(bid, bullet);
    // }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // pub fn test_download() -> Result<(), error::Brror> {

    // }
}
