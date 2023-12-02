use bilibili_downloader::*;
use std::sync::Arc;
use tokio::fs;

/// default task num
const TASK_NUM: &'static str = "7";

async fn run(bv: &str, task_num: u8) {
    let mut downloader = Downloader::new(task_num).unwrap();
    if let Ok(video) = downloader.build_video(bv.to_string()).await {
        let video = Arc::new(video);
        let title = video.title.as_str();
        let dir = &format!("{}_{}", bv, title.to_string());

        match fs::create_dir(dir.as_str()).await {
            Ok(_) => {
                downloader.dir = dir.to_owned();
            }
            _ => {
                downloader.dir = bv.to_string();
                if let Err(e) = fs::create_dir(bv).await {
                    println!("create video dir fail: {:?}", e);
                    return;
                }
            }
        }

        println!("download {} start, title: `{}`", video.bv, video.title);
        let downloader = Arc::new(downloader);
        if video.content_len > 0 {
            downloader.start(video.clone()).await.unwrap();
        } else {
            println!("download {} fail, video size is 0", video.title);
        }
    } else {
        println!("build video fail");
    }
}

#[tokio::main]
async fn main() {
    let matches = clap::App::new("Bilibili Video Downloader")
        .version(clap::crate_version!())
        .author("Kirovj")
        .about("Don't use it illegally, I don't take any responsibility.")
        .arg(
            clap::Arg::with_name("bv")
                .help("Bilibili video bv id")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::with_name("tasknum")
                .short("t")
                .long("tasknum")
                .help("Async task num for downloader")
                .required(true)
                .takes_value(true)
                .default_value(TASK_NUM),
        )
        .get_matches();

    let task_num = match matches.value_of("tasknum").unwrap().parse::<u8>() {
        Ok(num) => {
            if num > 10 {
                println!("task num over 10, please use 1 ~ 10 instead");
                return;
            }
            num
        }
        _ => {
            println!("{:?}", matches.value_of("bv").unwrap());
            return;
        }
    };

    match matches.value_of("bv") {
        Some(bv) => run(bv, task_num).await,
        _ => {
            println!("bv id is empty!");
            return;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run() {
        run("", 7).await;
    }
}
