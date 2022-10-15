use downloader;

fn download(bid: &str, bullet: bool) -> () {
    let add = downloader::add(1, 2);
    println!("add {}", add);
    println!("download {}, {}", bid, bullet);
}

fn main() -> () {
    let matches = clap::App::new("Bilibili Video Downloader")
        .version(clap::crate_version!())
        .author("Kirovj")
        .about("Don't use it illegally, I don't take any responsibility.")
        .arg(
            clap::Arg::with_name("target")
                .short("t")
                .long("target")
                .help("Video bid")
                .required(true)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("bullet")
                .short("b")
                .long("bullet")
                .help("Need bullet comment default false"),
        )
        .get_matches();

    let bullet = matches.is_present("bullet");

    if let Some(bid) = matches.value_of("target") {
        download(bid, bullet);
    }

    panic!("Target is empty!")
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // pub fn test_download() -> Result<(), error::Brror> {

    // }
}
