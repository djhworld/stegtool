extern crate steg;
use clap::{App, Arg, ArgMatches};
use flexi_logger::Logger;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use steg::decoder::Decoder;
use steg::encoder::Encoder;

fn main() {
    let app = App::new("stegtool")
        .version("0.1")
        .author("Daniel Harper")
        .about("Steganography tool")
        .subcommand(
            App::new("decode")
                .about("decodes data from an image (if available!)")
                .arg(
                    Arg::new("debug")
                        .long("debug")
                        .required(false)
                        .about("print debug output (to stderr)"),
                )
                .arg(
                    Arg::new("input-image")
                        .long("image")
                        .short('i')
                        .value_name("FILE")
                        .required(true)
                        .about("the image (PNG) to decode data from"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .required(false)
                        .value_name("FILE")
                        .default_value("stdout")
                        .about("where to write the decoded data to"),
                ),
        )
        .subcommand(
            App::new("encode")
                .about("encodes data into a cover image")
                .arg(
                    Arg::new("debug")
                        .long("debug")
                        .required(false)
                        .about("print debug output"),
                )
                .arg(
                    Arg::new("cover-image")
                        .long("cover-image")
                        .short('c')
                        .value_name("FILE")
                        .required(true)
                        .about("the cover image (PNG) to encode data into"),
                )
                .arg(
                    Arg::new("input-data")
                        .short('i')
                        .long("input-data")
                        .value_name("FILE")
                        .required(false)
                        .default_value("stdin")
                        .about("the data to hide in the cover image"),
                )
                .arg(
                    Arg::new("output-image")
                        .short('o')
                        .long("output-image")
                        .required(false)
                        .value_name("FILE")
                        .default_value("stdout")
                        .about("the output image (PNG) to write to."),
                )
                .arg(
                    Arg::new("compress")
                        .short('z')
                        .long("compress")
                        .required(false)
                        .about("compress the input-data before hiding"),
                )
                .arg(
                    Arg::new("granularity")
                        .short('g')
                        .long("granularity")
                        .required(false)
                        .default_value("lsb")
                        .possible_values(&["lsb", "two-bits", "four-bits"])
                        .about("what granularity to use when encoding the input data into the output image.\n\nfour-bits will allow you to pack more data into an image, but results might be visible.\n\nlsb represents least significant bit encoding, which gives best hiding performance but uses a lot of data (8 output bytes for every 1 input byte)\n"),
                ),
        );

    let matches = app.get_matches();

    if let Err(err) = run(matches) {
        eprintln!("failure: {}", err);
        std::process::exit(1);
    }
}

fn run(matches: ArgMatches) -> Result<(), std::io::Error> {
    match matches.subcommand() {
        Some(("encode", cmd)) => {
            let cover_image_file = File::open(cmd.value_of("cover-image").unwrap())?;
            let cover_image = BufReader::new(cover_image_file);
            let mut input = stdin_or_file_reader(cmd.value_of("input-data").unwrap())?;
            let mut output_image = stdout_or_file_writer(cmd.value_of("output-image").unwrap())?;
            let compress = if cmd.is_present("compress") {
                steg::CompressInput::Gzip
            } else {
                steg::CompressInput::None
            };

            setup_logging(cmd.is_present("debug"));

            let byte_split_level = match cmd.value_of("granularity") {
                Some("four-bits") => steg::ByteSplitGranularity::FourBits,
                Some("two-bits") => steg::ByteSplitGranularity::TwoBits,
                Some("lsb") => steg::ByteSplitGranularity::OneBit,
                _ => panic!("unsupported grandularity!"),
            };

            let encoder = Encoder::new(compress, byte_split_level);
            encoder.encode(cover_image, &mut input, &mut output_image)
        }
        Some(("decode", cmd)) => {
            let input_image_file = File::open(cmd.value_of("input-image").unwrap())?;
            let mut input_image = BufReader::new(input_image_file);
            let mut output = stdout_or_file_writer(cmd.value_of("output").unwrap())?;

            setup_logging(cmd.is_present("debug"));

            let decoder = Decoder::new();
            decoder.decode(&mut input_image, &mut output)
        }
        Some(_) => Err(err_to_io_error("unsupported subcommand")),
        None => Err(err_to_io_error("no subcommand provided")),
    }
}

fn setup_logging(debug: bool) {
    if debug {
        Logger::with_str("debug")
            .start()
            .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));
    }
}

fn stdout_or_file_writer(path: &str) -> Result<Box<dyn Write>, std::io::Error> {
    match path {
        "stdout" | "" | "-" => {
            let st = Box::leak(Box::new(std::io::stdout()));
            let stdout = st.lock();
            Ok(Box::new(BufWriter::new(Box::new(stdout))))
        }
        _ => {
            let file = File::create(path)?;
            Ok(Box::new(BufWriter::new(file)))
        }
    }
}

fn stdin_or_file_reader(path: &str) -> Result<Box<dyn Read>, std::io::Error> {
    match path {
        "stdin" | "" | "-" => {
            let st = Box::leak(Box::new(std::io::stdin()));
            let stdin = st.lock();
            Ok(Box::new(BufReader::new(Box::new(stdin))))
        }
        _ => {
            let file = File::open(path)?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}

fn err_to_io_error<E>(error: E) -> std::io::Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    std::io::Error::new(std::io::ErrorKind::Other, error.into())
}
