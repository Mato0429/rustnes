pub mod cpu;
pub mod emufile;
pub mod nescart;
pub mod ppu;

use cpu::Cpu;
use nescart::{NesCart, PpuRead, PpuWrite};
use ppu::Ppu;

struct PpuBus<'a> {
    cart: &'a mut NesCart,
    vram: &'a mut [u8; 0x800],
}

impl<'a> ppu::Bus for PpuBus<'a> {
    fn read(&mut self, addr: u16) -> u8 {
        let addr = addr & 0x3FFF;
        let addr = if addr >= 0x3000 { addr - 0x1000 } else { addr };

        match self.cart.ppu_read(addr) {
            PpuRead::Internal(addr) => self.vram[(addr & 0x7FF) as usize],
            PpuRead::External(data) => data,
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x3FFF;
        let addr = if addr >= 0x3000 { addr - 0x1000 } else { addr };

        match self.cart.ppu_write(addr, data) {
            PpuWrite::Internal(addr, data) => self.vram[(addr & 0x7FF) as usize] = data,
            PpuWrite::External => (),
        }
    }
}

struct SyncBus<'a> {
    cart: &'a mut NesCart,
    wram: &'a mut [u8; 0x800],
    ppu: &'a mut Ppu,
    vram: &'a mut [u8; 0x800],
}

impl<'a> cpu::Bus for SyncBus<'a> {
    fn nmi_active(&self) -> bool {
        self.ppu.nmi_active()
    }

    fn irq_active(&self) -> bool {
        self.cart.irq_active()
    }

    fn read(&mut self, addr: u16) -> u8 {
        self.sync_with_cpu();

        let mut ppubus = PpuBus {
            cart: self.cart,
            vram: self.vram,
        };

        match addr {
            0x0000..=0x1FFF => self.wram[(addr & 0x7FF) as usize],
            0x2000..=0x3FFF if addr & 0x07 == 2 => self.ppu.read_ppustat(),
            0x2000..=0x3FFF if addr & 0x07 == 4 => self.ppu.read_oamdata(),
            0x2000..=0x3FFF if addr & 0x07 == 7 => self.ppu.read_ppudata(&mut ppubus),
            0x2000..=0x3FFF => 0x00,
            0x4000..=0x401F => 0x00,
            0x4020..=u16::MAX => self.cart.cpu_read(addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.sync_with_cpu();

        let mut ppubus = PpuBus {
            cart: self.cart,
            vram: self.vram,
        };

        match addr {
            0x0000..=0x1FFF => self.wram[(addr & 0x7FF) as usize] = data,
            0x2000..=0x3FFF if addr & 0x07 == 0 => self.ppu.write_ppuctrl(data),
            0x2000..=0x3FFF if addr & 0x07 == 1 => self.ppu.write_ppumask(data),
            0x2000..=0x3FFF if addr & 0x07 == 3 => self.ppu.write_oamaddr(data),
            0x2000..=0x3FFF if addr & 0x07 == 4 => self.ppu.write_oamdata(data),
            0x2000..=0x3FFF if addr & 0x07 == 5 => self.ppu.write_ppuscrl(data),
            0x2000..=0x3FFF if addr & 0x07 == 6 => self.ppu.write_ppuaddr(data),
            0x2000..=0x3FFF if addr & 0x07 == 7 => self.ppu.write_ppudata(&mut ppubus, data),
            0x2000..=0x3FFF => (),
            0x4000..=0x401F => (),
            0x4020..=u16::MAX => self.cart.cpu_write(addr, data),
        }
    }
}

impl<'a> SyncBus<'a> {
    fn sync_with_cpu(&mut self) {
        let mut ppubus = PpuBus {
            cart: self.cart,
            vram: self.vram,
        };

        self.ppu.cpu_step(&mut ppubus);
        self.cart.cpu_step();
    }
}

#[derive(Debug, Clone)]
pub struct Nes {
    pub cart: NesCart,
    pub cpu: Cpu,
    pub wram: [u8; 0x800],
    pub ppu: Ppu,
    pub vram: [u8; 0x800],
}

impl Nes {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            cart: NesCart::empty(),
            cpu: Cpu::new(),
            wram: [0u8; 0x800],
            ppu: Ppu::new(),
            vram: [0u8; 0x800],
        }
    }

    pub fn reset(&mut self) {
        self.ppu.reset();

        let mut cpubus = SyncBus {
            cart: &mut self.cart,
            wram: &mut self.wram,
            ppu: &mut self.ppu,
            vram: &mut self.vram,
        };
        self.cpu.reset(&mut cpubus);
    }

    pub fn step(&mut self) {
        let mut cpubus = SyncBus {
            cart: &mut self.cart,
            wram: &mut self.wram,
            ppu: &mut self.ppu,
            vram: &mut self.vram,
        };

        self.cpu.step(&mut cpubus);
    }

    pub fn load_cart(&mut self, cart: NesCart) {
        self.cart = cart;
    }

    pub fn display_buffer(&self) -> &[u8; 256 * 240 * 4] {
        self.ppu.display_buffer()
    }
}
