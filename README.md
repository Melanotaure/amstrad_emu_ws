# Workspace for my Amstrad CPC emulator

| <img width="320" height="242" alt="image" src="https://github.com/user-attachments/assets/a64cf44b-fef3-410c-96f9-e7f607bb8d6e" /> | <img width="384" height="320" alt="image" src="https://github.com/user-attachments/assets/5ce4ad5d-3217-4742-94fa-145b9c501e30" /> |
| :---: | :---: |
| *Original computer image* | *My emulator window* |



In this Workspace you'll find (in crates):
- cpc-core
- cpc-desktop
- cpc-web
- z80-core

## cpc-core
This is the core emulator that is declined in a desktop version (emulator running natively on a PC) and cpc-web which is a WebApp.

The core doesn't run by itself, it depends on the z80-core and is a part of either cpc-desktop or cpc-web.
(See each top version to execute the emulator).

## cpc-desktop
This is the PC version of the emulator.

### How to execute it:
- from the workspace directory: ```cargo run --release --package cpc-desktop```
- from cpc-desktop directory: ```cargo run --release```

## cpc-web
**/!\ Work in progress...**

## z80-core
This is the Z80 CPU emulator in itself. It can be used independently as a crate.

### How to execute it as tests:
See its README file in its directory.
