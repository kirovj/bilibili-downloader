use bilibili_downloader::*;
use std::sync::Arc;
use tokio::fs;

const TASK_NUM: u8 = 7;

#[tokio::main]
async fn main() -> () {
    let bv = "";
    let mut downloader = Downloader::new(TASK_NUM).unwrap();
    if let Ok(video) = downloader.build_video(bv.to_string()).await {
        let video = Arc::new(video);
        let title = video.title.as_str();
        downloader.dir = title.to_string();

        if let Err(_) = fs::create_dir(title).await {
            downloader.dir = bv.to_string();
            if let Err(e) = fs::create_dir(bv).await {
                panic!("create video dir fail: {:?}", e);
            }
        }
        println!("download {} start, title: `{}`", video.bv, video.title);
        let downloader = Arc::new(downloader);
        if video.content_len > 0 {
            downloader.start(video.clone()).await.unwrap();
        } else {
            todo!()
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
