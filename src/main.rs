use crate::gameboy::GameBoy;

mod gameboy;
mod lr35902;
mod memory;

fn main() {
    let rom = include_bytes!("../external/dmg_boot.bin").to_vec();

    let mut gb = GameBoy::new(rom);
    loop {
        gb.tick();
    }
}
