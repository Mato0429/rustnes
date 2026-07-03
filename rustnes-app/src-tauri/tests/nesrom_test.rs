use rustnes_app_lib::nes::NesRom;
use std::fs::File;

#[test]
fn nesrom_test() {
    let target = "/workspaces/rustnes/resources/roms/nestest.nes";
    let file = File::open(target).unwrap();
    let rom = NesRom::create(file).unwrap();
    println!("{:?}", rom.header);
}
