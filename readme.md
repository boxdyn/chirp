# Chirp

How does an emulator work?
I don't know!

So I wrote this, to see if i can find out.

## Features:
- 32 * 64 1bpp pixel display, scaled 16x
- Full coverage of the original Chip-8 insn set
- 64-bit floating point internal sound/delay timers
- Pause/Resume
- Set and unset breakpoints
- A fairly nice command-line interface

## Keybinds:
- F1: Dump CPU registers
- F2: Dump screen to terminal
- F3: Dump screen to file
- F4: Enable/Disable live disassembly
- F5: Pause/Resume
- F6: Single-step instruction
- F7: Set breakpoint at current instruction
- F8: Unset breakpoint at current instruction
- F9: Soft-reset the CPU

## Keypad mapping:
### QWERTY: 
|   |   |   |   |
|---|---|---|---|
| 1 | 2 | 3 | 4 |
| Q | W | E | R |
| A | S | D | F |
| Z | X | C | V |
### Chip-8:
|   |   |   |   |
|---|---|---|---|
| 1 | 2 | 3 | C |
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
| A | 0 | B | F |

## Command Line Interface:
```
Usage: chirp [OPTIONS]

Positional arguments:
  file                 Load a ROM to run on Chirp.

Optional arguments:
  -h, --help           Print this help message.
  -d, --debug          Enable debug mode at startup.
  -p, --pause          Enable pause mode at startup.
  -s, --speed SPEED    Set the instructions-per-frame rate.
  -S, --step STEP      Run the emulator as fast as possible for `step` instructions.
  -z, --vfreset        Disable setting vF to 0 after a bitwise operation.
  -x, --drawsync       Disable waiting for vblank after issuing a draw call.
  -c, --memory         Use CHIP-48 style DMA instructions, which don't touch I.
  -v, --shift          Use CHIP-48 style bit-shifts, which don't touch vY.
  -b, --jumping        Use SUPER-CHIP style indexed jump, which is indexed relative to v[adr].
  -B, --break BP       Set breakpoints for the emulator to stop at.
  -D, --data WORD      Load additional word at address 0x1fe
  -f, --frame-rate FR  Set the target framerate. (default: 60)
  ```

## TODO:

- [ ] Implement sound
- [ ] Finish unit tests for "quirks"
- [ ] Make pausing/unpausing the emulator less messy
- [ ] Make resetting the emulator possible
- [ ] Allow code to be passed in hex on the command line? Hmm
- [ ] Assembler for my assembly syntax
- [ ] Make a UI for realtime configuration
- [ ] Cycle accuracy with original Chip-8 interpreter