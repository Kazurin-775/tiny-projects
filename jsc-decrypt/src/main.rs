use std::{ffi::OsStr, io::Read, path::PathBuf};

use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(short)]
    key: Option<String>,

    #[structopt(short)]
    force: bool,
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let options = Options::from_args();
    let output = options.output.unwrap_or_else(|| {
        let mut output = options.input.clone();
        if output.extension() != Some(OsStr::new("jsc")) {
            log::warn!(
                "Unexpected file extension: {:?}, continuing",
                output.extension(),
            );
        }
        assert!(output.set_extension("js"));
        output
    });

    if output.exists() && !options.force {
        log::error!(
            "The output file {:?} already exists, use -f to overwrite",
            output,
        );
        return;
    }

    log::info!("Processing input file {:?}", options.input);
    let encrypted = std::fs::read(options.input).expect("read input file");

    let decrypted = if let Some(key) = &options.key {
        log::debug!("Decrypting xxtea data with key");
        xxtea::decrypt(&encrypted, key)
    } else {
        log::warn!("No key specified, skipping decrypt");
        encrypted
    };

    let unzipped = if &decrypted[0..2] == b"\x1f\x8b" {
        log::debug!("Gzip header detected, unzipping");
        let mut buf = Vec::new();
        let mut decoder = flate2::read::GzDecoder::new(&decrypted as &[u8]);
        decoder.read_to_end(&mut buf).expect("gunzip");
        buf
    } else {
        log::debug!("Decrypted data does not seem to be compressed");
        decrypted
    };

    log::info!("Writing output to {:?}", output);
    std::fs::write(output, unzipped).expect("write output file");
    log::info!("Done");
}
