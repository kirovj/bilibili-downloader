mod consts;
mod downloader;

fn main() -> Result<(), ureq::Error> {
    let b_id = "bvidexample";
    let url = consts::URL.to_string() + b_id;
    let r = ureq::get(url.as_str())
        .set("user-agent", consts::UA)
        .call()?
        .into_string()?;
    println!("{}", r);
    Ok(())
}
