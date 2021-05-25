/*

glulxtoc - Decompile a Glulx file into C code
=============================================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::{BufReader, Cursor, Write};
use std::path::PathBuf;
use std::time::Instant;
use std::thread;

use bytes::Buf;
use quick_xml;
use structopt::StructOpt;

use if_decompiler;
use if_decompiler::DebugFunctionData;

mod output;

#[derive(StructOpt)]
#[structopt(name = "glulxtoc", about = "Decompile a Glulx file into C code")]
struct Cli {
    /// The path of the Glulxe storyfile
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    /// Output folder
    #[structopt(long, parse(from_os_str))]
    out_dir: Option<PathBuf>,

    /// Stack size (MB) (for the glulxtoc app, not the stack of the Glulx file being decompiled)
    #[structopt(short, long)]
    stack_size: Option<u32>,

    /// Inform debug file
    #[structopt(long, parse(from_os_str))]
    debug_file: Option<PathBuf>,

    /// Disassembler mode - only disassemble, do not optimise or generate structured code
    #[structopt(short, long)]
    disassemble: bool,

    /// Stop disassembling if you reach a string
    #[structopt(long)]
    stop_on_string: bool,

    /// Safe function overrides
    #[structopt(long, use_delimiter = true)]
    safe_function_overrides: Option<Vec<u32>>,

    /// Unsafe function overrides
    #[structopt(long, use_delimiter = true)]
    unsafe_function_overrides: Option<Vec<u32>>,
}

fn main() -> Result<(), Box<std::io::Error>> {
    // Process arguments
    let args: Cli = Cli::from_args();

    let child = thread::Builder::new()
        .name("run".into())
        .stack_size((args.stack_size.unwrap_or(8) * 1024 * 1024) as usize)
        .spawn(move || -> Result<(), Box<std::io::Error>> { run(args)?; Ok(()) })
        .unwrap();

    child.join().unwrap()?;

    Ok(())
}

fn run(args: Cli) -> Result<(), Box<std::io::Error>> {
    // Find where we are running from
    let mut workspace_dir = env::current_exe()?;
    workspace_dir.pop();
    workspace_dir.pop();
    workspace_dir.pop();

    // Start processing args
    let mut storyfile_path = env::current_dir()?;
    storyfile_path.push(args.path);
    let name = storyfile_path.file_stem().expect("storyfile should not be relative").to_str().unwrap().to_string();

    let out_dir = match args.out_dir {
        Some(path) => path,
        None => {
            let mut path = storyfile_path.clone();
            let mut name = path.file_name().unwrap().to_os_string();
            name.push(".decompiled");
            path.pop();
            path.push(name);
            path
        }
    };

    // Read the storyfile
    println!("Starting to decompile {:?}", storyfile_path);
    let start = Instant::now();
    let data = std::fs::read(storyfile_path)?;
    let data_length = data.len();

    // Start parsing the file
    fn get_file_header(data: &[u8]) -> (u32, u32) {
        let mut cursor = Cursor::new(data);
        let magic = cursor.get_u32();
        cursor.set_position(8);
        let iff_type = cursor.get_u32();
        (magic, iff_type)
    }
    let (magic, iff_type) = get_file_header(&data);

    // Check for a blorb
    let image = if magic == 0x464F524D /* FORM */ && iff_type == 0x49465253 /* IFRS */ {
        parse_blorb(&data)
    }
    // A bare Glulx file
    else if magic == 0x476C756C /* Glul */ {
        Some(&*data)
    }
    else {
        panic!("Unrecognised file format");
    };

    // Read the debug file if specified
    let debug_function_data = match args.debug_file {
        Some(path) => {
            print!("Parsing the debug file...");
            io::stdout().flush().unwrap();
            let start_parse_debug_file = Instant::now();
            let file = File::open(path)?;
            let result = Some(parse_debug_file(BufReader::new(file)).expect("Error parsing XML"));
            println!(" completed in {:?}", start_parse_debug_file.elapsed());
            result
        },
        None => None,
    };

    // Decompile the storyfile
    print!("Disassembling the storyfile...");
    io::stdout().flush().unwrap();
    let start_disassemble = Instant::now();
    let mut decompiler = if_decompiler::glulx::GlulxState::new(debug_function_data, args.safe_function_overrides, args.stop_on_string, args.unsafe_function_overrides);
    decompiler.decompile_rom(image.unwrap());
    let duration = start_disassemble.elapsed();
    println!(" completed in {:?}", duration);

    // Output the C files
    let mut output = output::GlulxOutput::new(args.disassemble, data_length as u32, name, out_dir, decompiler, workspace_dir);
    output.output(&data)?;

    let duration = start.elapsed();
    println!("Total decompilation time: {:?}", duration);

    Ok(())
}

// Parse a blorb file
// TODO: parse debug data from blorb
fn parse_blorb<'a>(data: &'a [u8]) -> Option<&'a [u8]> {
    let length = data.len() as u64;
    let mut cursor = Cursor::new(data);
    cursor.set_position(12);
    let mut glulx_chunk = None;
    while cursor.position() < length {
        let chunk_type = cursor.get_u32();
        let chunk_length = cursor.get_u32();
        let chunk_end = cursor.position()as usize + chunk_length as usize;
        match chunk_type {
            0x474C554C /* GLUL */ => {
                glulx_chunk = Some(&data[cursor.position() as usize..chunk_end]);
            },
            _ => {},
        }
        cursor.set_position(chunk_end as u64);
    }
    if glulx_chunk.is_none() {
        panic!("Blorb file does not have a GLUL chunk");
    }
    glulx_chunk
}

// Parse an Inform debug file
fn parse_debug_file(str: BufReader<File>) -> quick_xml::Result<BTreeMap<u32, DebugFunctionData>> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_reader(str);
    reader.trim_text(true);
    let mut result = BTreeMap::default();
    let mut buf = Vec::new();
    let mut in_routine = false;
    let mut process_text = false;
    let mut text = String::new();
    let mut addr = 0;
    let mut len = 0;
    let mut name = String::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"byte-count" | b"identifier" | b"value" => {
                        process_text = true;
                        text.clear();
                    },
                    b"routine" => {
                        in_routine = true;
                    },
                    _ => {},
                };
            },
            Ok(Event::Text(e)) => {
                if in_routine && process_text {
                    text.push_str(&e.unescape_and_decode(&reader)?);
                }
            },
            Ok(Event::End(ref e)) => {
                match e.name() {
                    b"byte-count" => {
                        if in_routine {
                            len = text.parse::<u32>().unwrap();
                        }
                    },
                    b"identifier" => {
                        if in_routine && name == "" {
                            name = text.clone();
                        }
                    },
                    b"routine" => {
                        in_routine = false;
                        result.insert(addr, DebugFunctionData {
                            addr,
                            len,
                            name: name.clone(),
                        });
                        name.clear();
                    },
                    b"value" => {
                        if in_routine {
                            addr = text.parse::<u32>().unwrap();
                        }
                    },
                    _ => {},
                };
                process_text = false;
            },
            Ok(Event::Eof) => break,
            Err(e) => panic!("XML error in debug file at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        };
        buf.clear();
    };

    Ok(result)
}