use std::{collections::BTreeMap, fs::File, io::Write, path::PathBuf};

use pycrate_rs::nas::{NASMessage, ParseError};
use serde_json;
use pcap_file::{self, pcapng::{Block, PcapNgReader}};
use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long)]
    input_dir: PathBuf,

    #[arg(short, long)]
    output_dir: PathBuf,
}

fn ensure_output_dir(path: &PathBuf) -> std::io::Result<()> {
    match path.read_dir() {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(path)?;
            path.read_dir()?;
            Ok(())
        },
        Err(e) => Err(e),
    }
}

const GSMTAP_HDR_START: usize = 28;
const GSMTAP_HDR_END: usize = GSMTAP_HDR_START + 16;
const GSMTAP_TYPE_NAS: u8 = 18;

fn process_pcap(mut pcap_file: PcapNgReader<File>, output_file: &mut File) -> std::io::Result<()> {
    let mut msgs = BTreeMap::new();
    let mut i = -1;
    while let Some(Ok(block)) = pcap_file.next_block() {
        let data = match block {
            Block::EnhancedPacket(block) => block.data,
            Block::SimplePacket(block) => block.data,
            Block::Packet(block) => block.data,
            _ => continue,
        };
        i += 1;
        let gsmtap_hdr = &data[GSMTAP_HDR_START..GSMTAP_HDR_END];
        let gsmtap_type = gsmtap_hdr[2];
        let packet_data = &data[GSMTAP_HDR_END..];
        if gsmtap_type == GSMTAP_TYPE_NAS {
            match NASMessage::parse(packet_data) {
                Ok(packet) => { msgs.insert(i, Ok(packet)); },
                Err(ParseError::EncryptedNASMessage) => { msgs.insert(i, Err("(encrypted NAS message)".to_string())); },
                Err(err) => { msgs.insert(i, Err(format!("err on packet {}: {}", i, err))); },
            }
        }
    }
    let json_output = serde_json::to_string_pretty(&msgs)
        .expect("failed to serialize output");
    output_file.write_all(json_output.as_bytes())?;
    Ok(())
}

fn main() {
    let args = Cli::parse();
    env_logger::init();
    let input_dir = args.input_dir.read_dir()
        .expect("input_dir doesn't exit");
    ensure_output_dir(&args.output_dir)
        .expect("error opening/creating output dir");
    for maybe_entry in input_dir {
        let entry = maybe_entry.expect("can't read file");
        match entry.path().extension() {
            Some(ext) if ext == "pcap" => {},
            _ => continue,
        }
        let pcap_file = std::fs::File::open(entry.path())
            .expect("failed to open file");
        let pcap_reader = PcapNgReader::new(pcap_file)
            .expect("failed to read pcap file");
        let mut output_file_path = args.output_dir.clone();
        output_file_path.push(entry.path().file_stem().unwrap());
        output_file_path.set_extension("json");
        let mut output_file = std::fs::File::create(&output_file_path)
            .expect("failed to create output file");
        println!("{:?} -> {:?}", entry.path(), &output_file_path);
        process_pcap(pcap_reader, &mut output_file)
            .expect("failed to process pcap file");
    }
}
