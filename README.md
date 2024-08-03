# ayyboy
Yet another GameBoy and GameBoy Color emulator! A lot of the information had been figured out by reverse engineering software and hardware, complemented by the Pandocs and various online sources/blogs.
The project is not in a perfect state but I'd consider it ready for use. I would not have been able to kick things off this fast without the amazing help of the people over at the [Emulation Development](https://discord.com/invite/dkmJAes) Discord server.

![image](https://github.com/user-attachments/assets/a5213d9e-3bb1-40b6-a951-3e42c957c94a)
![image](https://github.com/user-attachments/assets/fce23396-8d24-448e-97f8-98adbce90413)
![image](https://github.com/user-attachments/assets/d569944f-17d1-462b-bcdd-39ac0ab6512e)
![image](https://github.com/user-attachments/assets/7b0cefa6-1f1d-47ba-b80b-0c47901948fd)
![image](https://github.com/user-attachments/assets/eded7e68-e5a8-4f30-ba2d-1e85f76fc211)

Pokemon Red Version (DMG)  |  Pokemon Gold Version (GBC)
:-------------------------:|:-------------------------:
[![](https://...Dark.png)](https://github.com/user-attachments/assets/a5213d9e-3bb1-40b6-a951-3e42c957c94a)  |  ![image](https://github.com/user-attachments/assets/fce23396-8d24-448e-97f8-98adbce90413)

## Features
* DMG and GBC support (incl. double speed mode)
* Support for ROM, MBC1, MBC3 and MBC5 (although none of the mappers I'd consder in a 100% functional state)
* Sound (mostly taken from [this blog](https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html) and [this emulator](https://github.com/NightShade256/Argentum))
* RAM-based save games (RAM is simply written to disk on emulator exit)
* Built-in open-source boot ROMs for [DMG](https://github.com/Hacktix/Bootix) and [GBC](https://github.com/LIJI32/SameBoy/tree/master/BootROMs)
* Scanline based renderer (no pixel FIFO)
* Various debug views
* ZIP file support

## Testing
* The CPU has been verified against the following tests and passes all of them:
  * [SM83 SingleStepTests](https://github.com/SingleStepTests/sm83)
  * [cpu_instrs.gb](https://github.com/retrio/gb-test-roms)
* The PPU has been tested with `dmg-acid` and `cgb-acid` and passes both

![image](https://github.com/user-attachments/assets/67b6e026-df09-4b0a-acf7-9111cfea16c1)
![image](https://github.com/user-attachments/assets/cc325d90-628a-4207-b8b4-72625d6ff195)
