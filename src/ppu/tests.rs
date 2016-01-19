use super::*;
use memory::MemSegment;
use mappers::{Mapper, MapperParams};
use std::rc::Rc;
use std::cell::RefCell;
use cart::Cart;
use ppu::{AddrByte, OAMEntry, PPUCtrl};
use screen::DummyScreen;

fn create_test_ppu() -> PPU {
    create_test_ppu_with_rom(vec![0u8; 0x1000])
}

fn create_test_ppu_with_rom(chr_rom: Vec<u8>) -> PPU {
    let mapper = Mapper::new(0, MapperParams::simple(vec![0u8; 0x1000], chr_rom));
    let cart = Cart::new(mapper);
    PPU::new(Rc::new(RefCell::new(cart)), Box::new(DummyScreen::new()))
}

fn assert_register_single_writable(idx: u16, getter: &Fn(&PPU) -> u8) {
    let mut ppu = create_test_ppu();
    ppu.write(idx, 12);
    assert_eq!(getter(&ppu), 12);
    ppu.write(idx, 125);
    assert_eq!(getter(&ppu), 125);
}

fn assert_register_double_writable(idx: u16, getter: &Fn(&PPU) -> u16) {
    let mut ppu = create_test_ppu();
    ppu.write(idx, 0xDE);
    assert_eq!(getter(&ppu), 0xDE00);
    assert_eq!(AddrByte::Low, ppu.reg.address_latch);
    ppu.write(idx, 0xAD);
    assert_eq!(getter(&ppu), 0xDEAD);
    assert_eq!(AddrByte::High, ppu.reg.address_latch);
}

fn assert_register_ignores_writes(idx: u16, getter: &Fn(&PPU) -> u8) {
    let mut ppu = create_test_ppu();
    ppu.write(idx, 12);
    assert_eq!(getter(&ppu), 0);
    ppu.write(idx, 125);
    assert_eq!(getter(&ppu), 0);
}

fn assert_writing_register_fills_latch(idx: u16) {
    let mut ppu = create_test_ppu();
    ppu.write(idx, 12);
    assert_eq!(ppu.reg.dyn_latch, 12);
    ppu.write(idx, 125);
    assert_eq!(ppu.reg.dyn_latch, 125);
}

fn assert_register_is_readable(idx: u16, setter: &Fn(&mut PPU, u8) -> ()) {
    let mut ppu = create_test_ppu();
    setter(&mut ppu, 12);
    assert_eq!(ppu.read(idx), 12);
    setter(&mut ppu, 125);
    assert_eq!(ppu.read(idx), 125);
}

fn assert_register_not_readable(idx: u16) {
    let mut ppu = create_test_ppu();
    ppu.reg.dyn_latch = 12;
    assert_eq!(ppu.read(idx), 12);
    ppu.reg.dyn_latch = 125;
    assert_eq!(ppu.read(idx), 125);
}

#[test]
fn ppuctrl_is_write_only_register() {
    assert_register_single_writable(0x2000, &|ref ppu| ppu.reg.ppuctrl.bits);
    assert_writing_register_fills_latch(0x2000);
    assert_register_not_readable(0x2000);
}

#[test]
fn ppu_mirrors_address() {
    assert_register_single_writable(0x2008, &|ref ppu| ppu.reg.ppuctrl.bits);
    assert_register_single_writable(0x2010, &|ref ppu| ppu.reg.ppuctrl.bits);
}

#[test]
fn ppumask_is_write_only_register() {
    assert_register_single_writable(0x2001, &|ref ppu| ppu.reg.ppumask.bits());
    assert_writing_register_fills_latch(0x2001);
    assert_register_not_readable(0x2001);
}

#[test]
fn ppustat_is_read_only_register() {
    assert_register_ignores_writes(0x2002, &|ref ppu| ppu.reg.ppustat.bits);
    assert_writing_register_fills_latch(0x2002);
    assert_register_is_readable(0x2002,
                                &|ref mut ppu, val| {
                                    ppu.reg.ppustat = PPUStat::from_bits_truncate(val);
                                    ppu.reg.dyn_latch = val;
                                });
}

#[test]
fn reading_ppustat_returns_part_of_dynlatch() {
    let mut ppu = create_test_ppu();
    ppu.reg.dyn_latch = 0b0001_0101;
    ppu.reg.ppustat = PPUStat::from_bits_truncate(0b1010_0101);
    assert_eq!(ppu.read(0x2002), 0b1011_0101);
}

#[test]
fn reading_ppustat_clears_addr_latch() {
    let mut ppu = create_test_ppu();
    ppu.reg.address_latch = AddrByte::Low;
    ppu.read(0x2002);
    assert_eq!(ppu.reg.address_latch, AddrByte::High);
}

#[test]
fn oamaddr_is_write_only_register() {
    assert_register_single_writable(0x2003, &|ref ppu| ppu.reg.oamaddr);
    assert_writing_register_fills_latch(0x2003);
    assert_register_not_readable(0x2003);
}

#[test]
fn ppuscroll_is_2x_write_only_register() {
    assert_register_double_writable(0x2005, &|ref ppu| ppu.reg.ppuscroll);
    assert_writing_register_fills_latch(0x2005);
    assert_register_not_readable(0x2005);
}

#[test]
fn ppuaddr_is_2x_write_only_register() {
    assert_register_double_writable(0x2006, &|ref ppu| ppu.reg.ppuaddr);
    assert_writing_register_fills_latch(0x2006);
    assert_register_not_readable(0x2006);
}

#[test]
fn reading_oamdata_uses_oamaddr_as_index_into_oam() {
    let mut ppu = create_test_ppu();
    for x in 0u8..63u8 {
        ppu.oam[x as usize] = OAMEntry::new(x, x, x, x);
    }
    ppu.reg.oamaddr = 0;
    assert_eq!(ppu.read(0x2004), 0);
    ppu.reg.oamaddr = 10;
    assert_eq!(ppu.read(0x2004), 2);
}

#[test]
fn reading_oamdata_increments_oamaddr() {
    let mut ppu = create_test_ppu();
    ppu.reg.oamaddr = 0;
    ppu.read(0x2004);
    assert_eq!(ppu.reg.oamaddr, 1);
    ppu.reg.oamaddr = 255;
    ppu.read(0x2004);
    assert_eq!(ppu.reg.oamaddr, 0);
}

#[test]
fn writing_oamdata_uses_oamaddr_as_index_into_oam() {
    let mut ppu = create_test_ppu();
    ppu.reg.oamaddr = 0;
    ppu.write(0x2004, 12);
    assert_eq!(ppu.oam[0].y, 12);
    ppu.reg.oamaddr = 10;
    ppu.write(0x2004, 3);
    assert_eq!(ppu.oam[2].attr.bits(), 3);
}

#[test]
fn writing_oamdata_increments_oamaddr() {
    let mut ppu = create_test_ppu();
    ppu.reg.oamaddr = 0;
    ppu.write(0x2004, 12);
    assert_eq!(ppu.reg.oamaddr, 1);
    ppu.reg.oamaddr = 255;
    ppu.write(0x2004, 12);
    assert_eq!(ppu.reg.oamaddr, 0);
}

#[test]
fn ppu_can_read_chr_rom() {
    let mut chr_rom = vec![0u8; 0x2000];
    chr_rom[0x0ABC] = 12;
    chr_rom[0x0DBA] = 212;
    let mut ppu = create_test_ppu_with_rom(chr_rom);

    ppu.reg.ppuaddr = 0x0ABC;
    assert_eq!(ppu.read(0x2007), 12);

    ppu.reg.ppuaddr = 0x0DBA;
    assert_eq!(ppu.read(0x2007), 212);
}

#[test]
fn ppu_can_read_write_vram() {
    let mut ppu = create_test_ppu();

    ppu.reg.ppuaddr = 0x2ABC;
    ppu.write(0x2007, 12);
    ppu.reg.ppuaddr = 0x2ABC;
    assert_eq!(ppu.read(0x2007), 12);

    ppu.reg.ppuaddr = 0x2DBA;
    ppu.write(0x2007, 212);
    ppu.reg.ppuaddr = 0x2DBA;
    assert_eq!(ppu.read(0x2007), 212);

    // Mirroring
    ppu.reg.ppuaddr = 0x2EFC;
    ppu.write(0x2007, 128);
    ppu.reg.ppuaddr = 0x3EFC;
    assert_eq!(ppu.read(0x2007), 128);
}

#[test]
fn accessing_ppudata_increments_ppuaddr() {
    let mut ppu = create_test_ppu();
    ppu.reg.ppuaddr = 0x2000;
    ppu.read(0x2007);
    assert_eq!(ppu.reg.ppuaddr, 0x2001);
    ppu.write(0x2007, 0);
    assert_eq!(ppu.reg.ppuaddr, 0x2002);
}

#[test]
fn accessing_ppudata_increments_ppuaddr_by_32_when_ctrl_flag_is_set() {
    let mut ppu = create_test_ppu();
    ppu.reg.ppuctrl = PPUCtrl::new(0b0000_0100);
    ppu.reg.ppuaddr = 0x2000;
    ppu.read(0x2007);
    assert_eq!(ppu.reg.ppuaddr, 0x2020);
    ppu.write(0x2007, 0);
    assert_eq!(ppu.reg.ppuaddr, 0x2040);
}

#[test]
fn ppu_can_read_write_palette() {
    let mut ppu = create_test_ppu();

    ppu.reg.ppuaddr = 0x3F00;
    ppu.write(0x2007, 12);
    ppu.reg.ppuaddr = 0x3F00;
    assert_eq!(ppu.ppu_mem.palette[0], Color::from_bits_truncate(12));

    ppu.reg.ppuaddr = 0x3F01;
    ppu.write(0x2007, 212);
    ppu.reg.ppuaddr = 0x3F01;
    assert_eq!(ppu.read(0x2007), 212 & 0x3F);
}

#[test]
fn test_palette_mirroring() {
    let mut ppu = create_test_ppu();

    let mirrors = [0x3F10, 0x3F14, 0x3F18, 0x3F1C];
    let targets = [0x3F00, 0x3F04, 0x3F08, 0x3F0C];
    for x in 0..4 {

        ppu.reg.ppuaddr = targets[x];
        ppu.write(0x2007, 12);
        ppu.reg.ppuaddr = mirrors[x];
        assert_eq!(ppu.read(0x2007), 12);

        ppu.reg.ppuaddr = mirrors[x];
        ppu.write(0x2007, 12);
        ppu.reg.ppuaddr = targets[x];
        assert_eq!(ppu.read(0x2007), 12);
    }
}