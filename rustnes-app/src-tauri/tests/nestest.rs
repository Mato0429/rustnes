use std::{fs::File, io::BufReader};

use rustnes_app_lib::nes::{
    cpu::{Bus, Cpu},
    loader::parse_emufile,
    mapper::{Mapper, MapperLogic},
};

struct MocBus {
    wram: [u8; 0x800],
    nesrom: Mapper,
}

impl Bus for MocBus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..0x2000 => self.wram[(addr & 0x7FF) as usize],
            0x2000..0x4020 => 0x00,
            0x4020..=u16::MAX => self.nesrom.cpu_read(addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..0x2000 => self.wram[(addr & 0x7FF) as usize] = data,
            0x2000..0x4020 => (),
            0x4020..=u16::MAX => self.nesrom.cpu_write(addr, data),
        }
    }
}

#[test]
fn nestest() {
    let file = File::open("/workspaces/rustnes/assets/roms/nestest.nes").unwrap();
    let emufile = parse_emufile(BufReader::new(file)).unwrap();

    let mut bus = MocBus {
        wram: [0u8; 0x800],
        nesrom: emufile.nesrom,
    };

    let mut cpu = Cpu::new();
    cpu.reset(&mut bus);
    cpu.reg.pc = 0xC000;

    for i in 0..8991 {
        println!(
            "{:04}: {:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:}",
            i + 1,
            cpu.reg.pc,
            cpu.reg.a,
            cpu.reg.x,
            cpu.reg.y,
            cpu.reg.p.as_byte(false),
            cpu.reg.sp,
            cpu.total_cycle
        );

        cpu.step(&mut bus)
    }

    println!("Result 0x02: {:03X}h", bus.wram[0x02]);
    println!("Result 0x03: {:03X}h", bus.wram[0x03]);
}
