use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    os::unix::prelude::FromRawFd,
    path::PathBuf,
};

use arguably::ArgParser;

#[derive(Debug, Clone)]
struct Options {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl Options {
    pub fn parse() -> std::io::Result<Options> {
        let mut parser = ArgParser::new().helptext("Usage: runiq...").version("1.0");

        if let Err(err) = parser.parse() {
            err.exit();
        }

        let input = match parser.args.get(0) {
            Some(input_path) => Some(Self::parse_path(input_path)?),
            None => None,
        };

        let output = match parser.args.get(1) {
            Some(output_path) => Some(Self::parse_path(output_path)?),
            None => None,
        };

        Ok(Options {
            input: input,
            output: output,
        })
    }

    pub fn open_input(&self) -> std::io::Result<Box<dyn BufRead>> {
        let file = match self.input {
            Some(ref path) => File::open(path)?,
            None => unsafe { File::from_raw_fd(0) },
        };

        let reader = BufReader::with_capacity(4 * 1024, file);

        Ok(Box::new(reader))
    }

    pub fn open_output(&self) -> std::io::Result<io::BufWriter<File>> {
        let file = match self.output {
            Some(ref path) => File::open(path)?,
            None => unsafe { File::from_raw_fd(1) },
        };

        let buf_writer = BufWriter::with_capacity(64 * 1024, file);

        Ok(buf_writer)
    }

    fn parse_path(raw_path: &str) -> Result<PathBuf, io::Error> {
        let mut path_buf = PathBuf::new();
        path_buf.push(raw_path);

        if !path_buf.is_file() {
            return Err(io::Error::new(io::ErrorKind::Other, "Not a file"));
        }

        Ok(path_buf)
    }
}

#[allow(dead_code)]
fn with_iterator() -> std::io::Result<()> {
    let options = Options::parse()?;
    let input = options.open_input()?;
    let mut output = options.open_output()?;

    let mut lines_iterator = input.lines();

    let first_line = lines_iterator.next();
    if first_line.is_none() {
        return Ok(());
    }

    let mut buffer = first_line.unwrap()?;
    output.write(&buffer.as_bytes())?;
    output.write("\n".as_bytes())?;

    for line in lines_iterator {
        let line = line?;

        if line != buffer {
            buffer = line;
            output.write(&buffer.as_bytes())?;
            output.write("\n".as_bytes())?;
        }
    }

    Ok(())
}

fn buf_read() -> std::io::Result<()> {
    let options = Options::parse()?;
    let mut input = options.open_input()?;
    let mut output = options.open_output()?;

    let mut buffer = String::with_capacity(1024);

    let size = input.read_line(&mut buffer)?;

    if size == 0 {
        return Ok(());
    }

    let mut last_line = buffer.clone();

    output.write(&buffer.as_bytes())?;
    buffer.clear();

    while input.read_line(&mut buffer)? > 0 {
        if buffer != last_line {
            output.write(&buffer.as_bytes())?;
            last_line.clear();
            last_line.push_str(&buffer);
        }
        buffer.clear();
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    // with_iterator()?;
    buf_read()?;

    Ok(())
}
