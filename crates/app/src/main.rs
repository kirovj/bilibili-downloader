use downloader::Downloader;


#[tokio::main]
async fn main() -> () {
    let downloader = Downloader::new().unwrap();

    downloader.download("BV1BU4y1r7wg".to_string()).await;


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
