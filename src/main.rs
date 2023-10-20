use bilibili_downloader::*;
use std::sync::Arc;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};

const TASK_NUM: u8 = 8;

async fn merge_chunk_files(video: Arc<Video>) -> () {
    let title = video.title.as_str();
    let bv = video.bv.as_str();
    let format = video.format.as_str();
    let video_path = format!("{}/video.{}", bv, format);
    let video_path = video_path.as_str();
    let mut file = create_file(video_path)
        .await
        .map_or_else(|_| panic!("create result file fail"), |f| f);

    for index in 0..TASK_NUM + 1 {
        let chunk_path = format!("{}/{}_{}", bv, bv, index);
        let chunk_path = chunk_path.as_str();
        match fs::File::open(chunk_path).await {
            Ok(mut chunk_file) => {
                println!("merge chunk file {}", index);
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
        fs::remove_file(chunk_path).await.unwrap_or_else(|why| {
            println!("remove chunk file fail: {:?}", why.kind());
        });
    }
    println!("merge chunk files finished");
    let _ = mix_video_audio(
        format!("{}/video.{}", bv, format).as_str(),
        format!("{}/audio.mp3", bv).as_str(),
        format!("{}/{}.{}", bv, title, format).as_str(),
    )
    .await
    .map_err(|why| {
        println!("remove chunk file fail: {:?}", why.kind());
    });
}

#[tokio::main]
async fn main() -> () {
    let bv = "";
    let downloader = Downloader::new(TASK_NUM).unwrap();
    if let Ok(video) = downloader.build_video(bv.to_string()).await {
        let video = Arc::new(video);
        match fs::create_dir(bv).await {
            Ok(_) => {
                println!("download {} start, title: `{}`", video.bv, video.title);
                let downloader = Arc::new(downloader);
                if video.content_len > 0 {
                    downloader
                        .clone()
                        .download_chunks(video.clone())
                        .await
                        .unwrap();
                    downloader
                        .clone()
                        .download_audio(video.clone())
                        .await
                        .unwrap();
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
        let bv = "";
        let downloader = Downloader::new(TASK_NUM).unwrap();
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
