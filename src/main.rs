use rppal::gpio::{Gpio, InputPin, Trigger};

use std::{fs, io};
use std::fs::{DirEntry, File};
use std::io::{BufReader};
use std::path::Path;

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use rodio::{Decoder, OutputStream, Sink};

fn main() -> ! {
    log::set_boxed_logger(Box::new(SimpleLogger::default()))
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Unable to establish logger");

    info!("Hello, pets!");

    //set up buttons
    let (_stream, stream_handle) = OutputStream::try_default().expect("Unable to get default output stream");
    let aud_path = Path::new("audio");
    let gpio = Gpio::new().expect("Unable to create new gpio");
    for dir in 1..10
    {
        let sink = Sink::try_new(&stream_handle).expect(&*format!("Unable to sink for pin {}", dir));
        let path_buf = aud_path.join(dir.to_string());
        let file = fs::read_dir(path_buf.clone())
            .expect(&*format!("Unable to read directory {:?}", path_buf))
            .next()
            .expect(&*format!("Unable to find file in {:?}", path_buf))
            .expect("I don't know what goes here");

        let mut pin = gpio.get(dir).expect(&*format!("unable to get pin {}", dir)).into_input_pullup();
        pin.set_async_interrupt(Trigger::FallingEdge, move |_| foo(&sink, file, dir))
            .expect(&*format!("Unable to set interrupt on pin {}", dir));
    }

    loop {

    }
}

fn foo(sink: &Sink, file: DirEntry, dir: u8)
{
    let reader = BufReader::new(File::open(file.path()).expect(&*format!("Unable to open file {:?}", file)));
    let source = Decoder::new(reader).expect(&*format!("Unable to create encoder for {:?}", file));

    info!("Callback for button {}", dir);
    (*sink).append(source);
}