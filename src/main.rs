use anyhow::Result;

use std::env;
use std::fs::File;
use std::hash::Hasher;
use std::io::prelude::*;
use twox_hash::XxHash32;

#[allow(dead_code)]
#[derive(Debug)]
struct Flg {
    version: u8,
    b_indep: bool,
    b_checksum: bool,
    c_size: bool,
    c_checksum: bool,
    dict_id: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Bd {
    block_maxsize: u8,
}

impl From<u8> for Flg {
    fn from(byte: u8) -> Flg {
        Flg {
            version: byte >> 6,
            b_indep: byte >> 5 & 1 == 1,
            b_checksum: byte >> 4 & 1 == 1,
            c_size: byte >> 3 & 1 == 1,
            c_checksum: byte >> 2 & 1 == 1,
            dict_id: byte & 1 == 1,
        }
    }
}

impl From<u8> for Bd {
    fn from(byte: u8) -> Bd {
        Bd {
            block_maxsize: byte >> 4 & 0b111,
        }
    }
}

// HEADER FORMAT - field name with size in bytes below. () indicates optional
//
// FLG  BD  (Content Size)  (Dictionary ID)  HC
// 1    1   0-8             0-4              1

fn header(filename: &str) -> Result<()> {
    let mut file = File::open(filename)?;
    let mut buf: Vec<u8> = Vec::new();
    let res = file.read_to_end(&mut buf)?;
    println!("Read {res} bytes");
    let magic = u32::from_le_bytes(buf[0..4].try_into()?);
    println!("Magic: {magic:08X}");
    assert_eq!(magic, 0x184d2204);
    let flag: Flg = buf[4].into();
    println!("Flag: {flag:?}");
    let bd: Bd = buf[5].into();
    println!("Bd: {bd:?}");
    let (hc, hash, mut datablocks, datasize) = if !flag.c_size {
        let hc = buf[6];
        let mut t = XxHash32::with_seed(0);
        t.write(&buf[4..6]);
        let t = t.finish();
        let datablocks: [u8; 4] = buf[7..11].try_into().unwrap();
        (hc, t, datablocks, None)
    } else {
        let hc = buf[14];
        let mut t = XxHash32::with_seed(0);
        t.write(&buf[4..14]);
        let t = t.finish();
        let datasize: u64 = u64::from_le_bytes(buf[5..13].try_into().unwrap());
        let datablocks: [u8; 4] = buf[15..19].try_into().unwrap();
        (hc, t, datablocks, Some(datasize))
    };
    if let Some(datasize) = datasize {
        println!("uncompressed data len {datasize}")
    }
    println!("HeaderChecksum: {hc:02X}");

    println!("xxhash {hash:08X}");
    // lz4 only uses the 2nd byte
    let t = hash.to_le_bytes()[1];
    println!("hash byte {t:02X}");
    let compressed = datablocks[3] >> 7 != 0;
    datablocks[3] &= 0b0111_1111;
    println!("datablocks {:?}", datablocks);
    println!("data is compressed? {compressed}");
    let datasize = u32::from_le_bytes(datablocks);
    println!("datasize: {datasize}");

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() > 1 {
        args[1].clone()
    } else {
        "hello.txt.lz4".to_string()
    };
    dbg!(args);
    header(filename.as_str()).unwrap();
    println!("Hello, world!");
}
