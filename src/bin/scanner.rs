extern crate nes_emulator;

use nes_emulator::cart::Rom;

use std::path::Path;
use std::env;
use std::fs;
use std::fs::*;
use std::io;
use std::collections::HashSet;

fn visit_dirs(dir: &Path, cb: &mut FnMut(&DirEntry)) -> io::Result<()> {
    if try!(fs::metadata(dir)).is_dir() {
        for entry in try!(fs::read_dir(dir)) {
            let entry = try!(entry);
            if try!(fs::metadata(entry.path())).is_dir() {
                try!(visit_dirs(&entry.path(), cb));
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn scan_dirs() -> io::Result<()> {
    let args = env::args();
    let path_str = args.skip(1).next().unwrap();
    let path = Path::new(&path_str);
    let mut mappers: HashSet<u8> = HashSet::new();
    let result = visit_dirs(path,
                            &mut |file: &DirEntry| {
                                let path = file.path();
                                match path.extension().and_then(|ext| ext.to_str()) {
                                    Some("NES") | Some("nes") => {
                                        println!("{}", path.to_str().unwrap());
                                        match Rom::read(&path) {
                                            Ok(ref rom) => {
                                                println!("PRG Size: {}, CHR Size: {}, PRG_RAM \
                                                          Size:{}, trainer size: {}",
                                                         rom.prg_rom.len(),
                                                         rom.chr_rom.len(),
                                                         rom.prg_ram.len(),
                                                         rom.trainer.len());
                                                println!("SRAM:{}, Screen Mode: {:?}, PC10: {}, \
                                                          VS: {}, Mapper: {}",
                                                         rom.sram(),
                                                         rom.screen_mode(),
                                                         rom.pc10(),
                                                         rom.vs(),
                                                         rom.mapper());
                                                mappers.insert(rom.mapper());
                                            }
                                            Err(ref err) => println!("{}", err),
                                        };
                                        println!("");
                                    }
                                    _ => (),
                                };
                            });
    let mut mappers: Vec<u8> = mappers.iter().cloned().collect();
    mappers.sort();
    println!("Mappers: {:?}", mappers);
    result
}

fn main() {
    scan_dirs().ok().unwrap();
}
