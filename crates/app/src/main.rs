use std::sync::Arc;
use tokio::{
    fs,
    io::{self, AsyncReadExt, AsyncWriteExt},
};

use downloader::{Downloader, Video};
use futures::future::join_all;

const TASK_NUM: u8 = 8;

async fn download_chunks(downloader: Arc<Downloader>, video: Arc<Video>) {
    let chunk_size = video.content_lenth / TASK_NUM as u64;
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
        let downloader = downloader.clone();
        let video = video.clone();
        let handler =
            tokio::spawn(async move { downloader.download_chunk(video, range, index as u8).await });
        handler_list.push(handler);
    }
    join_all(handler_list).await;
}

async fn download_entry(bv: &str) -> Arc<Video> {
    let downloader = Arc::new(Downloader::new().unwrap());

    let video = downloader
        .build_video(bv.to_string())
        .await
        .map_or_else(|e| panic!("{}", e), |v| Arc::new(v));

    match fs::create_dir(bv).await {
        Ok(_) => {
            let clone = video.clone();
            println!("download {} start, title: `{}`", clone.bv, clone.title);
            if clone.content_lenth > 0 {
                download_chunks(downloader, clone).await;
            } else {
                todo!()
            }
            return video;
        }
        Err(_) => panic!("create video dir failed"),
    }
}

async fn create_file(filepath: String) -> io::Result<fs::File> {
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(filepath)
        .await
}

async fn merge_chunk_files(video: Arc<Video>) -> () {
    let mut file = match create_file(format!("{}/{}.{}", video.bv, video.title, video.format)).await
    {
        Ok(f) => f,
        Err(_) => create_file(format!("{}/{}.{}", video.bv, video.bv, video.format))
            .await
            .map_or_else(|_| panic!("create result file fail"), |f| f),
    };

    for index in 0..TASK_NUM + 1 {
        let chunk_path = format!("{}/{}_{}", video.bv, video.bv, index);
        match fs::File::open(chunk_path).await {
            Ok(mut chunk_file) => {
                println!("merge file {}", index);
                let size = chunk_file.metadata().await.map_or_else(
                    |_| panic!("read chunk file size fail"),
                    |metadata| metadata.len(),
                );
                let mut buf = vec![0; size as usize];
                chunk_file
                    .read_exact(&mut buf)
                    .await
                    .unwrap_or_else(|_| panic!("read chunk file fail"));
                file.write_all(&buf)
                    .await
                    .map_or_else(|_e| panic!("read chunk file fail"), |_| {});
            }
            Err(_) => continue,
        }
    }
    println!("merge file finished");
}

#[tokio::main]
async fn main() -> () {
    let bv = "BV1Q14y1L76r";
    let video = download_entry(bv).await;
    let _ = merge_chunk_files(video).await;

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
