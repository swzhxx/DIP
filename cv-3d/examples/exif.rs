use anyhow::Result;
use std::{fs, io};
fn main() -> Result<()> {
    let file = fs::File::open("./images/DSC_0480.jpg")?;
    let mut bufreader = io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif_info = exifreader.read_from_container(&mut bufreader)?;
    for f in exif_info.fields() {
        println!(
            "{} {} {}",
            f.tag,
            f.ifd_num,
            f.display_value().with_unit(&exif_info)
        );
    }

    Ok(())
}
