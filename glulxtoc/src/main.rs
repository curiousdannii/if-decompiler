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
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Instant;

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

    /// Inform debug file
    #[structopt(long, parse(from_os_str))]
    debug_file: Option<PathBuf>,

    /// Disassembler mode - only disassemble, do not optimise or generate structured code
    #[structopt(short, long)]
    disassemble: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Find where we are running from
    let mut workspace_dir = env::current_exe()?;
    workspace_dir.pop();
    workspace_dir.pop();
    workspace_dir.pop();

    // Process arguments
    let args = Cli::from_args();
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
    let start = Instant::now();
    let data = std::fs::read(storyfile_path)?;

    // Read the debug file if specified
    let debug_function_data = match args.debug_file {
        Some(path) => {
            let start_parse_debug_file = Instant::now();
            let file = File::open(path)?;
            let result = Some(parse_debug_file(BufReader::new(file))?);
            println!("Time parsing the debug file: {:?}", start_parse_debug_file.elapsed());
            result
        },
        None => None,
    };

    // Decompile the storyfile
    let start_disassemble = Instant::now();
    let mut decompiler = if_decompiler::glulx::GlulxState::new(data.into_boxed_slice(), debug_function_data);
    decompiler.decompile_rom();
    let duration = start_disassemble.elapsed();
    println!("Time disassembling the storyfile: {:?}", duration);

    // Output the C files
    let ramstart = decompiler.read_addr(8);
    let mut output = output::GlulxOutput {
        disassemble_mode: args.disassemble,
        have_warned_about_dynamic_branches: false,
        name,
        out_dir,
        ramstart,
        state: decompiler,
        workspace_dir,
    };
    output.output()?;

    let duration = start.elapsed();
    println!("Total decompilation time: {:?}", duration);

    Ok(())
}

// Parse an Inform debug file
fn parse_debug_file(str: BufReader<File>) -> quick_xml::Result<BTreeMap<u32, DebugFunctionData>> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_reader(str);
    reader.trim_text(true);
    let mut result = BTreeMap::default();
    let mut buf = Vec::new();
    let mut in_routine = false;
    let mut text = String::new();
    let mut addr = 0;
    let mut len = 0;
    let mut name = String::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name() == b"routine" {
                    in_routine = true;
                }
                text.clear();
            },
            Ok(Event::Text(e)) => {
                if in_routine {
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
            },
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        };
        buf.clear();
    };

    Ok(result)
}