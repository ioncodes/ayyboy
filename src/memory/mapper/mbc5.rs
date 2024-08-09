use btleplug::api::Characteristic;
use btleplug::platform::Peripheral;
use log::{error, info};

use super::Mapper;

#[derive(Clone)]
pub struct Mbc5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: u16,
    ram_bank: u8,
    ram_enabled: bool,
    allow_rumble: bool,
    #[allow(dead_code)]
    lovense_toy: Option<(Peripheral, Characteristic)>,
}

impl Mbc5 {
    pub fn new(memory: Vec<u8>) -> Mbc5 {
        Mbc5 {
            rom: memory,
            ram: vec![0; 0x8000],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            allow_rumble: false,
            lovense_toy: None,
        }
    }

    pub fn with_rumble(memory: Vec<u8>) -> Mbc5 {
        let lovense_toy = Mbc5::find_lovense_toy();

        Mbc5 {
            rom: memory,
            ram: vec![0; 0x8000],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            allow_rumble: true,
            lovense_toy,
        }
    }

    #[cfg(feature = "nsfw")]
    fn queue_vibration(&self) {
        use btleplug::api::{Peripheral as _, WriteType};
        use tokio::runtime::Runtime;

        if let Some((peripheral, tx)) = &self.lovense_toy {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                peripheral
                    .write(&tx, "Vibrate:10;".as_bytes(), WriteType::WithoutResponse)
                    .await
                    .unwrap();
            });
        }
    }

    #[cfg(not(feature = "nsfw"))]
    fn queue_vibration(&self) {}

    #[cfg(feature = "nsfw")]
    fn stop_vibration(&self) {
        use btleplug::api::{Peripheral as _, WriteType};
        use tokio::runtime::Runtime;

        if let Some((peripheral, tx)) = &self.lovense_toy {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                peripheral
                    .write(&tx, "Vibrate:0;".as_bytes(), WriteType::WithoutResponse)
                    .await
                    .unwrap();
            });
        }
    }

    #[cfg(not(feature = "nsfw"))]
    fn stop_vibration(&self) {}

    #[cfg(feature = "nsfw")]
    fn find_lovense_toy() -> Option<(Peripheral, Characteristic)> {
        use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
        use btleplug::platform::Manager;
        use regex::Regex;
        use tokio::runtime::Runtime;
        use tokio::time;

        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let manager = Manager::new().await.unwrap();
            let adapters = manager.adapters().await.unwrap();
            let central = adapters.into_iter().nth(0).expect("No adapters found");

            info!("Scanning for Lovense toy");
            central.start_scan(ScanFilter::default()).await.unwrap();

            // Wait for a peripheral to be discovered
            time::sleep(time::Duration::from_secs(5)).await;

            let peripherals = central.peripherals().await.unwrap();
            let service_regex = Regex::new(r"^..300001-002.-4bd4-bbd5-a6920e4c5653").unwrap(); // Regex from: @Acurisu
            let tx_regex = Regex::new(r"^..300002-002.-4bd4-bbd5-a6920e4c5653").unwrap();

            for peripheral in peripherals {
                // Connect to all peripherals to discover the Lovense service
                if let Ok(_) = peripheral.connect().await {
                    // Discover services
                    peripheral.discover_services().await.unwrap();

                    let services = peripheral.services();
                    let lovense_service = services
                        .iter()
                        .find(|&service| service_regex.is_match(&service.uuid.to_string()));

                    // If the service is found, return the peripheral and the TX characteristic
                    if let Some(service) = lovense_service {
                        info!("Found Lovense toy");

                        let tx_characteristic = service
                            .characteristics
                            .iter()
                            .find(|&characteristic| tx_regex.is_match(&characteristic.uuid.to_string()))
                            .unwrap();

                        info!("Queuing vibration command to signal connection");
                        peripheral
                            .write(&tx_characteristic, "Vibrate:1;".as_bytes(), WriteType::WithoutResponse)
                            .await
                            .unwrap();
                        peripheral
                            .write(&tx_characteristic, "Vibrate:0;".as_bytes(), WriteType::WithoutResponse)
                            .await
                            .unwrap();

                        central.stop_scan().await.unwrap();

                        return Some((peripheral, tx_characteristic.clone()));
                    }
                }
            }

            central.stop_scan().await.unwrap();

            None
        })
    }

    #[cfg(not(feature = "nsfw"))]
    fn find_lovense_toy() -> Option<(Peripheral, Characteristic)> {
        None
    }
}

impl Mapper for Mbc5 {
    #[inline]
    fn read(&self, addr: u16) -> Result<u8, crate::error::AyyError> {
        match addr {
            0x0000..=0x3fff => Ok(self.rom[addr as usize]),
            0x4000..=0x7fff => {
                let addr = (addr as usize % 0x4000) + (self.rom_bank as usize * 0x4000);
                Ok(self.rom[addr])
            }
            0xa000..=0xbfff if self.ram_enabled => {
                let base_addr = (addr - 0xa000) as usize;
                let addr = base_addr + (self.ram_bank as usize * 0x2000);
                Ok(self.ram[addr])
            }
            0xa000..=0xbfff if !self.ram_enabled => {
                error!(
                    "MBC5: Attempted read from RAM bank {} while RAM is disabled",
                    self.ram_bank
                );
                Ok(0)
            }
            _ => {
                error!("MBC5: Unmapped read from address {:04x}", addr);
                Ok(0)
            }
        }
    }

    #[inline]
    fn write(&mut self, addr: u16, data: u8) -> Result<(), crate::error::AyyError> {
        match addr {
            0x0000..=0x1fff => {
                self.ram_enabled = data & 0x0f == 0x0a;
                Ok(())
            }
            0x2000..=0x2fff => {
                self.rom_bank = (self.rom_bank & 0x100) | data as u16;
                Ok(())
            }
            0x3000..=0x3fff => {
                self.rom_bank = (self.rom_bank & 0xff) | ((data as u16 & 0x1) << 8);
                Ok(())
            }
            0x4000..=0x5fff => {
                self.ram_bank = data & 0x0f;

                if self.ram_bank & 0b1000 != 0 && self.allow_rumble {
                    info!("Triggering vibration");
                    self.queue_vibration();
                } else if self.allow_rumble {
                    info!("Stopping vibration");
                    self.stop_vibration();
                }
                Ok(())
            }
            0xa000..=0xbfff if self.ram_enabled => {
                let base_addr = (addr - 0xa000) as usize;
                let addr = base_addr + (self.ram_bank as usize * 0x2000);
                self.ram[addr] = data;
                Ok(())
            }
            0xa000..=0xbfff if !self.ram_enabled => {
                error!(
                    "MBC5: Attempted write to RAM bank {} while RAM is disabled",
                    self.ram_bank
                );
                Ok(())
            }
            _ => {
                error!("MBC5: Unmapped write to address {:04x}", addr);
                Ok(())
            }
        }
    }

    fn dump_ram(&self) -> Vec<u8> {
        self.ram.clone()
    }

    fn load_ram(&mut self, ram: Vec<u8>) {
        self.ram = ram;
    }

    #[inline]
    fn current_rom_bank(&self) -> u16 {
        self.rom_bank
    }

    #[inline]
    fn current_ram_bank(&self) -> u8 {
        self.ram_bank
    }

    #[inline]
    fn name(&self) -> String {
        if !self.allow_rumble {
            String::from("MBC5")
        } else {
            String::from("MBC5+RUMBLE")
        }
    }
}
