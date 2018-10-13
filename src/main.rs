#[macro_use] extern crate structopt;
#[macro_use] extern crate failure;
extern crate byteorder;

mod vtx;
mod faces;

use structopt::StructOpt;
use failure::{Error, ResultExt};
use std::path::PathBuf;
use std::num::ParseIntError;
use std::io::{self, Write, BufWriter, BufReader};
use std::fs::{File, OpenOptions};

#[derive(Debug, StructOpt)]
/// Dump SM64 start screen vertex and face data
struct Opts {
    /// Input binary
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    /// Address of Face or Vtx data info struct
    #[structopt(parse(try_from_str = "hex_or_dec"))]
    addr: u64,
    /// Output file, or stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
    /// RAM address of start of file
    #[structopt(short = "r", long = "ram", parse(try_from_str = "hex_or_dec"))]
    ram: Option<u64>,
    /// Look for vertex data (linked in NodeGroup)
    #[structopt(short = "v", long = "vertex", conflicts_with = "face")]
    vtx: bool,
    /// Look for face data (linked in PlaneGroup)
    #[structopt(short = "f", long = "face", conflicts_with = "vertex")]
    face: bool,
}

fn main() {
    let opts = Opts::from_args();

    if let Err(e) = run(opts) {
        eprintln!("Error: {}", e);
        for c in e.iter_causes() {
            eprintln!("caused by: {}", c);
        }
        ::std::process::exit(1);
    }
}

fn run(opts: Opts) -> Result<(), Error> {
    let f = File::open(opts.input).context("reading input file")?;
    let rdr = BufReader::new(f);
    let wtr = get_file_or_stdout(opts.output).context("opening output for writing")?;
    let vram = opts.ram.unwrap_or(0);

    match (opts.vtx, opts.face) {
        (true, false) => vtx::dump(rdr, wtr, opts.addr, vram as u32)?,
        (false, true) => faces::dump(rdr, wtr, opts.addr, vram as u32)?,
        _ => bail!("Illegal combination of mode options"),
    };

    Ok(())
}

fn hex_or_dec<S>(n: S) -> Result<u64, ParseIntError>
    where S: AsRef<str>
{
    let n: &str = n.as_ref();
    let op = &n[0..2];
    
    if op == "0x" || op == "0X" { 
        u64::from_str_radix(&n[2..], 16)
    } else { 
        u64::from_str_radix(n, 10)
    }
}

fn get_file_or_stdout(out: Option<PathBuf>) -> Result<BufWriter<Box<Write>>, io::Error> {
    Ok(BufWriter::new(
        if let Some(f) = out {
            let f = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(f)?;
            Box::new(f) as Box<Write>
        } else {
            Box::new(io::stdout()) as Box<Write>
        }
    ))
}
