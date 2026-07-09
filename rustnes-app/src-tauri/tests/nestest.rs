use rustnes_app_lib::nes::{loader::parse_emufile, Nes};
use std::fs::File;
use std::io::BufReader;

#[test]
fn nestest() {
    let file = File::open("/workspaces/rustnes/resources/roms/nestest.nes").expect("");
    let emufile = parse_emufile(BufReader::new(file)).expect("");
    let mut nes = Nes::new(emufile.env_info);

    nes.load(emufile.nesrom);
    nes.cpu.reg.pc.hi = 0xC0;
    nes.cpu.reg.pc.lo = 0x00;
    println!("{:?}", nes);

    // 8991
    for i in 0..7000 {
        let reg = nes.cpu.reg;
        nes.step();

        println!(
            "{:04}: {:#06X} A:{:02X} X:{:02X}, Y:{:02X}, P:{:02X}, SP:{:02X}",
            i + 1,
            u16::from(reg.pc),
            reg.a,
            reg.x,
            reg.y,
            reg.p.as_byte(false),
            u8::from(reg.sp)
        );
    }
}
