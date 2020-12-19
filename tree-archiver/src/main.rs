use std::{
    convert::TryInto,
    ffi::OsStr,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    time::SystemTime,
};

use chrono::{DateTime, Datelike, Local, Timelike};
use zip::{write::FileOptions, DateTime as ZipDateTime, ZipWriter};

fn systime_to_ziptime(systime: SystemTime) -> ZipDateTime {
    let datetime = DateTime::<Local>::from(systime).naive_local();
    let datetime = ZipDateTime::from_date_and_time(
        datetime.year().try_into().unwrap(),
        datetime.month().try_into().unwrap(),
        datetime.day().try_into().unwrap(),
        datetime.hour().try_into().unwrap(),
        datetime.minute().try_into().unwrap(),
        datetime.second().try_into().unwrap(),
    )
    .unwrap();
    datetime
}

fn should_ignore(filename: &OsStr) -> bool {
    filename == "$RECYCLE.BIN" || filename == "System Volume Information"
}

fn archive(
    realpath: &mut PathBuf,
    zip: &mut ZipWriter<BufWriter<File>>,
    zippath: &mut Vec<String>,
) -> std::io::Result<()> {
    for result in realpath.read_dir()? {
        let child = result?;
        let metadata = child.metadata()?;
        if should_ignore(&child.file_name()) {
            continue;
        }

        zippath.push(child.file_name().into_string().unwrap());
        if metadata.is_file() {
            let modified = systime_to_ziptime(metadata.modified()?);
            let file_opts = FileOptions::default().last_modified_time(modified);
            zip.start_file(zippath.join("/"), file_opts)?;
        } else {
            realpath.push(child.file_name());
            archive(realpath, zip, zippath)?;
            realpath.pop();
        }
        zippath.pop();
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    if args.len() != 2 {
        eprintln!("Usage: tree-archiver <path> <zipfile>");
        return Ok(());
    }

    let mut realpath = PathBuf::from(args.next().unwrap());
    let zippath = args.next().unwrap();
    let mut zip = ZipWriter::new(BufWriter::new(File::create(zippath)?));
    let mut zippath = Vec::new();

    archive(&mut realpath, &mut zip, &mut zippath)?;

    zip.finish()?.flush()?;
    Ok(())
}
