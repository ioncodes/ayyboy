# ayyboy
Yet another GameBoy and GameBoy Color emulator! A lot of the information had been figured out by reverse engineering software and hardware, complemented by the Pandocs and various online sources/blogs.
The project is not in a perfect state but it works well with my childhood games. I would not have been able to kick things off this fast without the amazing help of the people over at the [Emulation Development](https://discord.com/invite/dkmJAes) Discord server.

| Pokemon Red Version (DMG)                                                                 | Pokemon Gold Version (GBC)                                                                | Legend of Zelda - Link's Awakening DX (GBC)                                               | Déjà Vu I & II: The Casebooks of Ace Harding (GBC)                                        | Debug View (GBC)                                                                          |
| ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| ![image](https://github.com/user-attachments/assets/a5213d9e-3bb1-40b6-a951-3e42c957c94a) | ![image](https://github.com/user-attachments/assets/fce23396-8d24-448e-97f8-98adbce90413) | ![image](https://github.com/user-attachments/assets/d569944f-17d1-462b-bcdd-39ac0ab6512e) | ![image](https://github.com/user-attachments/assets/7b0cefa6-1f1d-47ba-b80b-0c47901948fd) | ![image](https://github.com/user-attachments/assets/eded7e68-e5a8-4f30-ba2d-1e85f76fc211) |

## Features
* DMG and GBC support (incl. double speed mode)
* Support for ROM, MBC1, MBC3 and MBC5 (although none of the mappers I'd consder in a 100% functional state)
* MBC5 rumble pak support through Lovense sex toys
* Sound (mostly taken from [this blog](https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html) and [this emulator](https://github.com/NightShade256/Argentum))
* RAM-based save games (RAM is simply written to disk on emulator exit and loaded on startup if a `.sav` file exists)
* Built-in open-source boot ROMs for [DMG](https://github.com/Hacktix/Bootix) and [GBC](https://github.com/LIJI32/SameBoy/tree/master/BootROMs)
* Scanline based renderer (no pixel FIFO)
* Various debug views
* ZIP file support

You might be wondering why I bothered implementing sex toy support. I do not have a clear answer to this question! I wondered how I could implement rumble support and since a PC cannot (usually?) vibrate, my brain came up with a vibrator feature.  

Lovense support is not compiled-in by default, however, it is available through the `nsfw` feature flag during compilation. Enabling this flag will force the emulator to start searching for nearby Lovense BLE products and connect to the first that matches a specific regex if rumble support is detected for a game.  

<details>
<summary>Open me to see a demo of a game controlling the Lush 2</summary>  
 
https://github.com/user-attachments/assets/eb051257-8fdc-421d-9159-86bf55ab8cbe

</details>

## Usage
Compile the emulator yourself or download a [release](). Note that providing a bootrom is completely optional and that `--log-to-file` will enable instruction tracing be default.

```
Usage: ayyboy.exe [OPTIONS] <ROM>

Arguments:
  <ROM>

Options:
      --bios <BIOS>
      --log-to-file
  -h, --help         Print help
```

## Testing
* The CPU has been verified against the following tests and passes all of them:
  * [SM83 SingleStepTests](https://github.com/SingleStepTests/sm83)
  * [cpu_instrs.gb](https://github.com/retrio/gb-test-roms)
* The PPU has been tested with `dmg-acid2` and `cgb-acid2` and passes both

| dmg-acid2                                                                                 | cgb-acid2                                                                                 |
| ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| ![image](https://github.com/user-attachments/assets/67b6e026-df09-4b0a-acf7-9111cfea16c1) | ![image](https://github.com/user-attachments/assets/cc325d90-628a-4207-b8b4-72625d6ff195) |
