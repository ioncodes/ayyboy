use crate::error::AyyError;
use crate::gameboy::Mode;
use crate::joypad::Joypad;
use crate::memory::mapper::Mapper;
use crate::memory::{
    BOOTROM_MAPPER_REGISTER, EXTERNAL_RAM_END, EXTERNAL_RAM_START, JOYPAD_REGISTER,
    OAM_DMA_REGISTER, ROM_END, ROM_START,
};
use crate::sound::apu::Apu;
use crate::sound::{
    NR10, NR11, NR12, NR13, NR14, NR21, NR22, NR23, NR24, NR30, NR31, NR32, NR33, NR34, NR41, NR42,
    NR43, NR44, NR50, NR51, NR52, WAVE_PATTERN_RAM_END, WAVE_PATTERN_RAM_START,
};
use log::{debug, error};

use super::addressable::Addressable;
use super::{
    VRAM_BANK_SELECT_REGISTER, VRAM_END, VRAM_START, WRAM_BANK1_END, WRAM_BANK1_START,
    WRAM_BANK_SELECT_REGISTER,
};

// The last instruction unmaps the boot ROM. Execution continues normally,
// thus entering cartridge entrypoint at $100
const DMG_BOOTROM_SIZE: u16 = 0xff;
const CGB_BOOTROM_SIZE: u16 = 0x8ff;

pub struct Mmu {
    pub cartridge: Box<dyn Mapper>,
    pub joypad: Joypad,
    pub apu: Apu,
    memory: Vec<u8>,
    cgb_vram_bank1: Vec<u8>, // 0x2000 bank 1
    cgb_wram_bank1: Vec<u8>, // 0x1000 bank 1-7
    bootrom: Vec<u8>,
    mode: Mode,
}

impl Mmu {
    pub fn new(bootrom: Vec<u8>, cartridge: Box<dyn Mapper>, mode: Mode) -> Mmu {
        Mmu {
            cartridge,
            memory: vec![0; 0x10000],
            cgb_vram_bank1: vec![0; 0x2000],
            cgb_wram_bank1: vec![0; 0x1000 * 7],
            bootrom,
            joypad: Joypad::new(),
            apu: Apu::new(),
            mode,
        }
    }

    #[inline]
    pub fn read(&self, addr: u16) -> Result<u8, AyyError> {
        if cfg!(test) {
            return Ok(self.memory[addr as usize]);
        }

        let bootrom_size = match self.mode {
            Mode::Dmg => DMG_BOOTROM_SIZE,
            Mode::Cgb => CGB_BOOTROM_SIZE,
        };

        match addr {
            ROM_START..=ROM_END if self.is_bootrom_mapped() && addr <= bootrom_size => {
                Ok(self.bootrom[addr as usize])
            }
            ROM_START..=ROM_END => self.cartridge.read(addr),
            VRAM_START..=VRAM_END if self.current_vram_bank() == 0 => {
                Ok(self.memory[addr as usize])
            }
            VRAM_START..=VRAM_END if self.current_vram_bank() == 1 => {
                Ok(self.cgb_vram_bank1[(addr - VRAM_START) as usize]) // CGB
            }
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => self.cartridge.read(addr),
            WRAM_BANK1_START..=WRAM_BANK1_END => {
                let bank = self.current_wram_bank();
                if bank > 0 {
                    Ok(self.cgb_wram_bank1
                        [((bank as u16 - 1) * 0x1000 + (addr - WRAM_BANK1_START)) as usize])
                } else {
                    Ok(self.memory[addr as usize])
                }
            }
            JOYPAD_REGISTER => Ok(self.joypad.as_u8(self.memory[addr as usize])),
            NR10
            | NR11
            | NR12
            | NR13
            | NR14
            | NR21
            | NR22
            | NR23
            | NR24
            | NR30
            | NR31
            | NR32
            | NR33
            | NR34
            | NR41
            | NR42
            | NR43
            | NR44
            | NR50
            | NR51
            | NR52
            | WAVE_PATTERN_RAM_START..=WAVE_PATTERN_RAM_END => Ok(self.apu.read(addr)),
            _ => Ok(self.memory[addr as usize]),
        }
    }

    #[inline]
    pub fn read_as<T>(&self, addr: u16) -> Result<T, AyyError>
    where
        T: From<u8>,
    {
        Ok(T::from(self.read(addr)?))
    }

    #[inline]
    pub fn read16(&self, addr: u16) -> Result<u16, AyyError> {
        let lo = self.read(addr)? as u16;
        let hi = self.read(addr.wrapping_add(1))? as u16;
        Ok((hi << 8) | lo)
    }

    #[inline]
    pub fn read_unchecked(&self, addr: u16) -> u8 {
        self.read(addr).unwrap()
    }

    #[inline]
    pub fn read_as_unchecked<T>(&self, addr: u16) -> T
    where
        T: From<u8>,
    {
        self.read_as(addr).unwrap()
    }

    #[inline]
    pub fn _read16_unchecked(&self, addr: u16) -> u16 {
        self.read16(addr).unwrap()
    }

    #[inline]
    pub fn _write16_unchecked(&mut self, addr: u16, data: u16) {
        self.write16(addr, data).unwrap();
    }

    #[inline]
    pub fn write(&mut self, addr: u16, data: u8) -> Result<(), AyyError> {
        if cfg!(test) {
            self.memory[addr as usize] = data;
            return Ok(());
        }

        let bootrom_size = match self.mode {
            Mode::Dmg => DMG_BOOTROM_SIZE,
            Mode::Cgb => CGB_BOOTROM_SIZE,
        };

        match addr {
            ROM_START..=ROM_END if self.is_bootrom_mapped() && addr <= bootrom_size => {
                error!("Attempted to write to bootrom");
            }
            ROM_START..=ROM_END => self.cartridge.write(addr, data)?,
            VRAM_START..=VRAM_END if self.current_vram_bank() == 0 => {
                self.memory[addr as usize] = data
            }
            VRAM_START..=VRAM_END if self.current_vram_bank() == 1 => {
                self.cgb_vram_bank1[(addr - VRAM_START) as usize] = data
            }
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => self.cartridge.write(addr, data)?,
            WRAM_BANK1_START..=WRAM_BANK1_END => {
                let bank = self.current_wram_bank();
                if bank > 0 {
                    self.cgb_wram_bank1
                        [((bank as u16 - 1) * 0x1000 + (addr - WRAM_BANK1_START)) as usize] = data
                } else {
                    self.memory[addr as usize] = data
                }
            }
            OAM_DMA_REGISTER => self.start_dma_transfer(data)?,
            NR10
            | NR11
            | NR12
            | NR13
            | NR14
            | NR21
            | NR22
            | NR23
            | NR24
            | NR30
            | NR31
            | NR32
            | NR33
            | NR34
            | NR41
            | NR42
            | NR43
            | NR44
            | NR50
            | NR51
            | NR52
            | WAVE_PATTERN_RAM_START..=WAVE_PATTERN_RAM_END => self.apu.write(addr, data),
            _ => self.memory[addr as usize] = data,
        }

        Ok(())
    }

    #[inline]
    pub fn write16(&mut self, addr: u16, data: u16) -> Result<(), AyyError> {
        let lo = data as u8;
        let hi = (data >> 8) as u8;
        self.write(addr, lo)?;
        self.write(addr.wrapping_add(1), hi)?;
        Ok(())
    }

    #[inline]
    pub fn write_unchecked(&mut self, addr: u16, data: u8) {
        self.write(addr, data).unwrap();
    }

    #[inline]
    pub fn is_bootrom_mapped(&self) -> bool {
        self.read(BOOTROM_MAPPER_REGISTER).unwrap() == 0x00
    }

    #[inline]
    pub fn current_vram_bank(&self) -> u8 {
        if self.mode == Mode::Cgb {
            self.read_unchecked(VRAM_BANK_SELECT_REGISTER) & 0b0000_0001
        } else {
            0
        }
    }

    #[inline]
    pub fn current_wram_bank(&self) -> u8 {
        if self.mode == Mode::Cgb {
            let bank = self.read_unchecked(WRAM_BANK_SELECT_REGISTER) & 0b0000_0111;
            if bank == 0 {
                1
            } else {
                bank
            }
        } else {
            0
        }
    }

    fn start_dma_transfer(&mut self, data: u8) -> Result<(), AyyError> {
        let src_addr = (data as u16) << 8;
        debug!("OAM DMA transfer from ${:04x}", src_addr);

        // TODO: Is this range correct?
        // TODO: Add cycles
        for i in 0..0xa0 {
            let byte = self.read(src_addr + i)?;
            self.write(0xfe00 + i, byte)?;
        }

        Ok(())
    }

    #[cfg(test)]
    pub fn resize_memory(&mut self, size: usize) {
        self.memory.resize(size, 0);
    }

    #[cfg(test)]
    pub fn unmap_bootrom(&mut self) {
        let _ = self.write(BOOTROM_MAPPER_REGISTER, 0x69);
    }
}
