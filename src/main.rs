mod consts;
mod downloader;
mod error;

fn test_download(bid: &str, bullet: bool) -> Result<(), error::Brror> {
    let url = consts::URL.to_string() + bid;
    let r = ureq::get(url.as_str())
        .set("user-agent", consts::UA)
        .call()?
        .into_string()?;
    println!("{}", r);
    Ok(())
}

fn main() -> Result<(), error::Brror> {
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
        return test_download(bid, bullet);
    }

    panic!("Target is empty!")
}
