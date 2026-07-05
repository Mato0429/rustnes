use rustnes_app_lib::nes::parse_emufile;
use std::fs::File;

#[test]
fn nesrom_test() {
    let target = "/workspaces/rustnes/resources/roms/nestest.nes";
    let file = File::open(target).unwrap();
    let rom = parse_emufile(file).unwrap();
    println!("{:?}", rom);
}
