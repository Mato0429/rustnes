use std::{
    fmt,
    fs::File,
    io::{BufReader, Read},
};

use rustnes_core::nes::{
    cpu::{Bus, Cpu},
    emufile::parse_emufile,
    nescart::{
        mapper::{Mapper, MapperArgs},
        NesCart,
    },
};

struct MockBus {
    wram: [u8; 0x800],
    nescart: NesCart,
}

impl Bus for MockBus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..0x2000 => self.wram[(addr & 0x7FF) as usize],
            0x2000..0x4020 => 0x00,
            0x4020..=u16::MAX => self.nescart.cpu_read(addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..0x2000 => self.wram[(addr & 0x7FF) as usize] = data,
            0x2000..0x4020 => (),
            0x4020..=u16::MAX => self.nescart.cpu_write(addr, data),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Step {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u8,
    cyc: u128,
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:}",
            self.pc, self.a, self.x, self.y, self.p, self.sp, self.cyc
        )
    }
}

#[test]
fn nestest() {
    let mut nestest_log = File::open("/workspaces/rustnes/assets/testroms/nestest.log").unwrap();
    let mut log_string = String::new();
    nestest_log.read_to_string(&mut log_string).unwrap();

    let nestest = File::open("/workspaces/rustnes/assets/testroms/nestest.nes").unwrap();
    let emufile = parse_emufile(BufReader::new(nestest)).unwrap();

    let mapper_args = MapperArgs {
        horizontal_nt: emufile.horizontal_nt,
        alternative_nt: emufile.alternative_nt,
        prgrom_size: emufile.prgrom.len() as u32,
        prgram_size: emufile.prgram_size,
        chrrom_size: emufile.chrrom.len() as u32,
        chrram_size: emufile.chrram_size,
    };

    let nescart = NesCart {
        mapper: Mapper::new(emufile.mapper_id, emufile.submapper, mapper_args).unwrap(),
        prgrom: emufile.prgrom,
        prgram: vec![0u8; emufile.prgram_size as usize],
        chrrom: emufile.chrrom,
        chrram: vec![0u8; emufile.chrram_size as usize],
    };

    let mut bus = MockBus {
        wram: [0u8; 0x800],
        nescart,
    };

    let mut cpu = Cpu::new();
    cpu.reset(&mut bus);
    cpu.reg.pc = 0xC000;

    for (step, line) in log_string.lines().enumerate() {
        let log_step = Step {
            pc: u16::from_str_radix(&line[0..4], 16).unwrap(),
            a: u8::from_str_radix(&line[50..52], 16).unwrap(),
            x: u8::from_str_radix(&line[55..57], 16).unwrap(),
            y: u8::from_str_radix(&line[60..62], 16).unwrap(),
            p: u8::from_str_radix(&line[65..67], 16).unwrap(),
            sp: u8::from_str_radix(&line[71..73], 16).unwrap(),
            cyc: line[90..].trim().parse::<u128>().unwrap(),
        };

        let my_step = Step {
            pc: cpu.reg.pc,
            a: cpu.reg.a,
            x: cpu.reg.x,
            y: cpu.reg.y,
            p: cpu.reg.p.as_byte(false),
            sp: cpu.reg.sp,
            cyc: cpu.total_cycle,
        };

        println!("{:04}: {:}", step + 1, my_step);
        if my_step != log_step {
            println!("\x1b[31m{:04}: {:}\x1b[0m", step + 1, log_step);
            panic!("nestest failed at step: {:}", step + 1);
        }

        cpu.step(&mut bus);
    }

    println!("Result 0x02: {:03X}h", bus.wram[0x02]);
    println!("Result 0x03: {:03X}h", bus.wram[0x03]);
}
