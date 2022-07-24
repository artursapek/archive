use chrono::NaiveDate;
use exif::{Reader, Tag};
use std::process::{Command, Output};
use walkdir::WalkDir;

/*
 *
 * archive [srcdir] [archivedir]
 * archive server archive
 *
 */

fn main() {
    let mut args = std::env::args();
    args.next();
    let src_dir = args.next().unwrap();
    dbg!(&src_dir);
    let archive_dir = args.next().unwrap();
    dbg!(&archive_dir);

    for entry in WalkDir::new(src_dir) {
        if let Ok(entry) = entry {
            let path = entry.path();
            let extension = path.extension();

            if let Some(ext) = path.extension() {
                match ext.to_ascii_lowercase().to_str() {
                    Some(ext) => {
                        match ext {
                            "png" | "jpg" | "jpeg" | "heic" => {
                                // Image

                                process_image(path);
                            }
                            "mov" => {
                                // Movie
                            }
                            other => {
                                println!("unhandled file extension: {}", path.display());
                            }
                        }
                    }
                    None => {
                        println!("unhandled file extension: {}", path.display());
                    }
                }
            }
        }
    }
}

enum Error {
    Io(std::io::Error),
    Exif(exif::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<exif::Error> for Error {
    fn from(err: exif::Error) -> Error {
        Error::Exif(err)
    }
}

fn process_image(path: &std::path::Path) -> Result<(), Error> {
    let file = std::fs::File::open(path)?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader)?;

    let date: Option<NaiveDate> = {
        let field = exif
            .get_field(Tag::DateTimeOriginal, exif::In::PRIMARY)
            .or(exif.get_field(Tag::GPSDateStamp, exif::In::PRIMARY));

        if let Some(field) = field {
            let s = field.display_value().to_string();

            let mut parts = s.split(' ').next().unwrap().split('-');
            let year = parts.next().unwrap();
            let month = parts.next().unwrap();
            let date = parts.next().unwrap();

            let year: i32 = year.parse().unwrap();
            let month: u32 = month.parse().unwrap();
            let date: u32 = date.parse().unwrap();

            Some(NaiveDate::from_ymd(year, month, date))
            //dbg!(s);
        } else {
            // As a last resort, look for kMDItemFSContentChangeDate using mdls

            let output = Command::new("mdls")
                .arg("-name")
                .arg("kMDItemFSContentChangeDate")
                .arg(path.display().to_string())
                .output()
                .expect("failed to execute process");

            match output.status.code() {
                Some(0) => {
                    // OK
                    let output = std::str::from_utf8(&output.stdout).unwrap();
                    let mut parts = output.split(' ');
                    parts.next();
                    parts.next();
                    let value = parts.next().unwrap();

                    let mut  parts = value.split('-');
                    let year = parts.next().unwrap();
                    let month = parts.next().unwrap();
                    let date = parts.next().unwrap();

                    let year: i32 = year.parse().unwrap();
                    let month: u32 = month.parse().unwrap();
                    let date: u32 = date.parse().unwrap();

                    Some(NaiveDate::from_ymd(year, month, date))
                }
                x => {
                    None
                    // Failed
                }
            }
        }
    };

    if let Some(date) = date {
        dbg!(date);
        // yay
    } else {
        println!("Failed to get date for {}", path.display());
    }

    Ok(())
}
