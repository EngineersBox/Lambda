use std::fs::{File, OpenOptions};
use std::{fs, io, thread};
use std::sync::Mutex;
use std::io::Write;

use slog::{Drain, Duplicate, Fuse, Logger, Record};
use slog_async::{Async, OverflowStrategy};
use slog_json::Json;
use slog_term::{FullFormat, TermDecorator, ThreadSafeTimestampFn, RecordDecorator, CountingWriter};
use regex::Regex;
use lazy_static::lazy_static;

macro_rules! get_current_thread_id {
    () => {
        o!("thread-id" => format!("{:?}", thread::current().id()))
    }
}

lazy_static! {
    static ref MODULE_SEPARATOR_REGEX: Regex = Regex::new(r"::").expect("Could not compile module separator regex");
}

///
/// Format the message according to the following standard:
/// `[YY-mm-dd HH:MM:SS.SSS] [MESSAGE] <LEVEL>: <MESSAGE>[, ...<KEY>: <VALUE>]`
///
/// # Arguments
/// * fn_timestamp: Method to get the current timestamp
/// * rd: RecordDecorator to write formatted message to
/// * record: Record to retrieve current logger data from (E.g. module, location, etc)
/// * use_file_location: Whether to specify the destination file
///
/// # Returns
/// `Result<Bool>`: `true` indicating message should be logged, `false` to skip
///
pub fn print_msg_header(fn_timestamp: &dyn ThreadSafeTimestampFn<Output = io::Result<()>>,
                        mut rd: &mut dyn RecordDecorator,
                        record: &Record,
                        use_file_location: bool) -> io::Result<bool> {
    rd.start_whitespace()?;
    write!(rd, "[")?;

    rd.start_timestamp()?;
    fn_timestamp(&mut rd)?;

    rd.start_whitespace()?;
    write!(rd, "] [")?;

    rd.start_value()?;
    let split_module: Vec<String> = MODULE_SEPARATOR_REGEX
        .split(record.module())
        .map(String::from)
        .collect::<Vec<String>>();
    write!(
        rd,
        "{}",
        split_module.get(split_module.len() - 1).unwrap(),
    )?;

    rd.start_whitespace()?;
    write!(rd, "] ")?;

    rd.start_level()?;
    write!(rd, "{}", record.level().as_short_str())?;

    if use_file_location {
        rd.start_location()?;
        write!(
            rd,
            "[{}:{}:{}]",
            record.location().file,
            record.location().line,
            record.location().column
        )?;
    }

    rd.start_whitespace()?;
    write!(rd, ": ")?;

    rd.start_msg()?;
    let mut count_rd = CountingWriter::new(&mut rd);
    write!(count_rd, "{}", record.msg())?;
    Ok(count_rd.count() != 0)
}

///
/// Retrieve the current date time in the following format:
/// `YY-mm-dd HH:MM:SS.SSS`
///
/// # Arguments
/// * io: Writer to forward timestamp to
///
/// # Returns
/// `Result<()>`: Empty result
///
pub fn timestamp_utc(io: &mut dyn io::Write) -> io::Result<()> {
    write!(io,
           "{}",
           chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
    )
}

///
/// Initialise a logger with a given prefix for the log file. Log file name will be
/// in the following format:
/// `<PREFIX>_<TIMESTAMP>.log`
///
/// # Arguments
/// * prefix: A string prefix for the log file name
///
/// # Returns
/// * Logger: A logger instance with two drains for STDOUT and JSON file writer
///
pub fn initialize_logging(prefix: String) ->  Logger {
    let log_path: String = String::from("logs/");
    let directory_creation_message: &str;
    match fs::create_dir(log_path.as_str()) {
        Ok(_) => { directory_creation_message = "Created logging directory"; },
        Err(_) => { directory_creation_message = "Logging directory already exists, skipping";}
    }

    let log_file_path: String = format!("{}{}{}",(log_path + prefix.as_str()).as_str(),chrono::Utc::now().to_string(),".log");
    let file: File = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_file_path.as_str())
        .unwrap();

    let decorator: TermDecorator = TermDecorator::new()
        .force_color()
        .build();

    type FuseFFTD = Fuse<FullFormat<TermDecorator>>;
    type FuseJF = Fuse<Json<File>>;
    type FuseMD = Fuse<Mutex<Duplicate<FuseFFTD, FuseJF>>>;

    // Define drain for STDOUT logging
    let d1: FuseFFTD = FullFormat::new(decorator)
        .use_custom_timestamp(timestamp_utc)
        .use_custom_header_print(print_msg_header)
        .build()
        .fuse();
    // Define drain for JSON file writing
    let d2: FuseJF = Json::default(file).fuse();
    // Define mutex for drain access to assure thread safety
    let both: FuseMD = Mutex::new(Duplicate::new(d1, d2)).fuse();
    // Create async access for for logging with Blocking strategy to queue up asynced methods
    let both: Fuse<Async> = Async::new(both)
        .overflow_strategy(OverflowStrategy::Block)
        .build()
        .fuse();
    let log: Logger = Logger::root(both, o!());

    info!(log.new(get_current_thread_id!()), "{}", directory_creation_message);
    return log;
}
