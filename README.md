# Chip8 Emulator in Rust
This is a chip8 emulator written in Rust following the book https://github.com/aquova/chip8-book. The emulator consists
on a chip8 core where actual CPU is emulated and two front-ends for it. One is a rust binary in `desktop` and the other is
in WebAssembly in `wasm`.

While the WebAssembly one is incomplete, the Binary one is fully functional.

# How to run it

It requires a `roms` folder in the root where you can place your games. You can find those on the internet pretty easy.

Once you have the roms you can just run the following commands to make it work:
```
cd desktop
cargo run "../roms/{NAME_OF_THE_ROM}"
```

# Disclaimer
This emulator is by no means complete and exhaustive, I wrote it in order to learn Rust and also to learn more about Emulation
which has always been an interest of mine. If you find any issues with it or have some other ideas, please feel free to fork it 
or suggest things. I'm sharing this here in case it can be useful to somebody who might be interested in going down the same path.