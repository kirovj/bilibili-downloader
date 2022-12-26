use bilibili_downloader::Downloader;
use bilibili_downloader::Video;
use std::sync::Arc;
use tokio::{
    fs,
    io::{self, AsyncReadExt, AsyncWriteExt},
};

const TASK_NUM: u8 = 8;

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
        Err(_) => {
            println!("cannot create file named by title, use bv instead");
            create_file(format!("{}/{}.{}", video.bv, video.bv, video.format))
                .await
                .map_or_else(|_| panic!("create result file fail"), |f| f)
        }
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
    let downloader = Downloader::new().unwrap();
    if let Ok(video) = downloader.build_video(bv.to_string()).await {
        let video = Arc::new(video);
        match fs::create_dir(bv).await {
            Ok(_) => {
                println!("download {} start, title: `{}`", video.bv, video.title);
                let downloader = Arc::new(downloader);
                if video.content_lenth > 0 {
                    downloader.download_chunks(video.clone()).await.unwrap();
                    let _ = merge_chunk_files(video.clone()).await;
                } else {
                    todo!()
                }
            }
            Err(_) => panic!("create video dir failed"),
        }
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
    use std::{fs::OpenOptions, io::Write};

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_download_bullet() -> Result<()> {
        let bv = "bvid";
        let downloader = Downloader::new().unwrap();
        let video = downloader.build_video(bv.to_string()).await.unwrap();
        let downloader = Arc::new(downloader);
        let danmuku_list = downloader.download_danmaku(Arc::new(video)).await?;
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(format!("{}_danmuku.txt", bv))
            .expect("cannot open file");
        for danmuku_seg in danmuku_list {
            for danmuku in danmuku_seg.elems {
                if let Ok(json) = serde_json::to_string(&danmuku) {
                    file.write_all(format!("{}\n", json).as_bytes())?;
                }
            }
        }
        Ok(())
    }
}
