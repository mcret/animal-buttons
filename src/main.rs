use rppal::gpio::{Gpio, InputPin, Trigger};

use std::{env, fs};
use std::fs::{File};
use std::io::{BufReader};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use rodio::{Decoder, OutputStream, Sink};

fn main() -> ! {
    log::set_boxed_logger(Box::new(SimpleLogger::default()))
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Unable to establish logger");

    let mut args: Vec<u8> = env::args().collect().into_bytes();
    args.remove(0);

    info!("Hello, pets!");

    //set up buttons
    let (_stream, stream_handle) = OutputStream::try_default()
        .expect("Unable to get default output stream");
    let aud_path = Path::new("audio");
    let gpio = Gpio::new().expect("Unable to create new gpio");
    let mut pins: Vec<InputPin> = Vec::new();
    for dir in args
    {
        let sink = Sink::try_new(&stream_handle)
            .expect(&*format!("Unable to sink for pin {}", dir));
        let path_buf = aud_path.join(dir.to_string());
        let file = fs::read_dir(path_buf.clone())
            .expect(&*format!("Unable to read directory {:?}", path_buf))
            .next()
            .expect(&*format!("Unable to find file in {:?}", path_buf))
            .expect("I don't know what goes here");

        info!("configuring in {}", dir);
        info!("with file {:?}", file.file_name());

        let mut pin = gpio.get(dir)
            .expect(&*format!("unable to get pin {}", dir)).into_input_pullup();
        let mut debouncer = Debouncer::new(sink, file.path(), dir);
        pin.set_async_interrupt(Trigger::RisingEdge, move |_| debouncer.foo())
            .expect(&*format!("Unable to set interrupt on pin {}", dir));
        pins.push(pin);
    }

    loop {

    }
}

struct Debouncer
{
    last_trigger: Instant,
    min_duration: Duration,
    sink: Sink,
    file: PathBuf,
    dir: u8,
}

impl Debouncer
{
    fn new(sink: Sink, file: PathBuf, dir: u8) -> Debouncer
    {
        Debouncer
        {
            last_trigger: Instant::now(),
            min_duration: Duration::from_secs(1),
            sink,
            file,
            dir,
        }
    }

    fn foo(&mut self)
    {
        if self.last_trigger.elapsed() < self.min_duration
        {
            return;
        }
        self.last_trigger = Instant::now();
        let reader = BufReader::new(File::open(&self.file)
            .expect(&*format!("Unable to open file {:?}", self.dir)));
        let source = Decoder::new(reader)
            .expect(&*format!("Unable to create encoder for {:?}", self.dir));

        info!("Callback for button {}:\t{:?}", self.dir, self.file.as_path());
        self.sink.append(source);
    }
}
