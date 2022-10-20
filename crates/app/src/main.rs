use std::sync::Arc;

use downloader::{Downloader, Video};
use futures::future::join_all;

const TASK_NUM: usize = 5;

async fn download(downloader: Arc<Downloader>, video: Video) -> bool {
    let chunk_size = video.content_lenth / TASK_NUM;
    let mut range_list = vec![];
    let mut start = 0;
    let mut end = 0;

    while end <= video.content_lenth {
        end += chunk_size;
        range_list.push((start, end));
        start = end + 1;
    }

    let mut handler_list = vec![];

    for range in range_list {
        println!("{:?}", range);
        let downloader = downloader.clone();
        let video = video.clone();
        let handler =
            tokio::spawn(async move { downloader.download_chunk(video, range.0, range.1).await });
        handler_list.push(handler);
    }
    join_all(handler_list).await;
    true
}

#[tokio::main]
async fn main() -> () {
    let downloader = Arc::new(Downloader::new().unwrap());

    let video = downloader.build_video("BV1Q14y1L76r".to_string()).await;

    if let Err(e) = video {
        panic!("{:?}", e);
    }

    let video = video.unwrap();
    println!("download {} start, title: `{}`", video.bv, video.title);
    if video.content_lenth > 0 {
        download(downloader, video).await;
    } else {
        todo!()
    }

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
