// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)

//! Unit tests for [super::CPU]
//!
//! These run instructions, and ensure their output is consistent with previous builds
//!
//! General test format:
//! 1. Prepare to do the thing
//! 2. Do the thing
//! 3. Compare the result to the expected result
//!
//! Some of these tests run >16M times, which is very silly

use super::*;
use crate::{
    bus,
    cpu::bus::{Bus, Region::*},
};
use rand::random;

mod decode;

fn setup_environment() -> (CPU, Bus) {
    let mut ch8 = (
        CPU {
            flags: Flags {
                debug: true,
                pause: false,
                monotonic: Some(8),
                ..Default::default()
            },
            ..CPU::default()
        },
        bus! {
            // Create a screen
            Screen  [0x0F00..0x1000] = include_bytes!("../../chip8Archive/roms/1dcell.ch8"),
        },
    );
    ch8.0
        .load_program_bytes(include_bytes!("tests/roms/jumptest.ch8"))
        .unwrap();
    ch8
}

fn print_screen(bytes: &[u8]) {
    bus! {Screen [0..0x100] = bytes}
        .print_screen()
        .expect("Printing screen should not fail if Screen exists.")
}

/// Unused instructions
mod unimplemented {
    use super::*;
    #[test]
    fn ins_5xyn() {
        let (mut cpu, mut bus) = setup_environment();
        cpu.screen.write(0x200u16, 0x500fu16);
        cpu.tick(&mut bus)
            .expect_err("0x500f is not an instruction");
    }
    #[test]
    fn ins_8xyn() {
        let (mut cpu, mut bus) = setup_environment();
        cpu.screen.write(0x200u16, 0x800fu16);
        cpu.tick(&mut bus)
            .expect_err("0x800f is not an instruction");
    }
    #[test]
    fn ins_9xyn() {
        let (mut cpu, mut bus) = setup_environment();
        cpu.screen.write(0x200u16, 0x900fu16);
        cpu.tick(&mut bus)
            .expect_err("0x900f is not an instruction");
    }
    #[test]
    fn ins_exbb() {
        let (mut cpu, mut bus) = setup_environment();
        cpu.screen.write(0x200u16, 0xe00fu16);
        cpu.tick(&mut bus)
            .expect_err("0xe00f is not an instruction");
    }
    // Fxbb
    #[test]
    fn ins_fxbb() {
        let (mut cpu, mut bus) = setup_environment();
        cpu.screen.write(0x200u16, 0xf00fu16);
        cpu.tick(&mut bus)
            .expect_err("0xf00f is not an instruction");
    }
}

mod sys {
    use super::*;
    /// 00e0: Clears the screen memory to 0
    #[test]
    fn clear_screen() {
        let (mut cpu, mut bus) = setup_environment();
        cpu.clear_screen(&mut bus);
        bus.get_region(Screen)
            .expect("Expected screen, got None")
            .iter()
            .for_each(|byte| assert_eq!(*byte, 0));
    }

    /// 00ee: Returns from subroutine
    #[test]
    fn ret() {
        let test_addr = random::<u16>() & 0x7ff;
        let (mut cpu, _) = setup_environment();
        // Place the address on the stack
        cpu.stack.push(test_addr);

        cpu.ret();

        // Verify the current address is the address from the stack
        assert_eq!(test_addr, cpu.pc);
    }
}

/// Tests control-flow instructions
///
/// Basically anything that touches the program counter
mod cf {

    use super::*;
    /// 1aaa: Sets the program counter to an absolute address
    #[test]
    fn jump() {
        let (mut cpu, _) = setup_environment();
        // Test all valid addresses
        for addr in 0x000..0xffe {
            // Jump to an address
            cpu.jump(addr);
            // Verify the current address is the jump target address
            assert_eq!(addr, cpu.pc);
        }
    }

    /// 2aaa: Pushes pc onto the stack, then jumps to a
    #[test]
    fn call() {
        let test_addr = random::<u16>();
        let (mut cpu, _) = setup_environment();
        // Save the current address
        let curr_addr = cpu.pc;
        // Call an address
        cpu.call(test_addr);
        // Verify the current address is the called address
        assert_eq!(test_addr, cpu.pc);
        // Verify the previous address was stored on the stack (sp+2)
        let stack_addr: u16 = cpu.stack.pop().expect("This should return test_addr");
        assert_eq!(stack_addr, curr_addr);
    }

    /// 3xbb: Skips the next instruction if register X == b
    #[test]
    fn skip_equals_immediate() {
        let (mut cpu, _) = setup_environment();
        for word in 0..=0xffff {
            let (a, b, addr) = (word as u8, (word >> 4) as u8, random::<u16>() & 0x7fe);
            for x in 0..=0xf {
                // set the PC to a random address
                cpu.pc = addr;

                cpu.v[x] = a;

                cpu.skip_equals_immediate(x, b);

                assert_eq!(cpu.pc, addr.wrapping_add(if a == b { 2 } else { 0 }));
            }
        }
    }

    /// 4xbb: Skips the next instruction if register X != b
    #[test]
    fn skip_not_equals_immediate() {
        let (mut cpu, _) = setup_environment();
        for word in 0..=0xffff {
            let (a, b, addr) = (word as u8, (word >> 4) as u8, random::<u16>() & 0x7fe);
            for x in 0..=0xf {
                // set the PC to a random address
                cpu.pc = addr;

                cpu.v[x] = a;

                cpu.skip_not_equals_immediate(x, b);

                assert_eq!(cpu.pc, addr.wrapping_add(if a != b { 2 } else { 0 }));
            }
        }
    }

    /// 5xy0: Skips the next instruction if register X != register Y
    #[test]
    fn skip_equals() {
        let (mut cpu, _) = setup_environment();
        for word in 0..=0xffff {
            let (a, b, addr) = (word as u8, (word >> 4) as u8, random::<u16>() & 0x7fe);
            for reg in 0..=0xff {
                let (x, y) = (reg & 0xf, reg >> 4);
                if x == y {
                    continue;
                }
                // set the PC to a random address
                cpu.pc = addr;

                (cpu.v[x], cpu.v[y]) = (a, b);

                cpu.skip_equals(x, y);

                assert_eq!(cpu.pc, addr.wrapping_add(if a == b { 2 } else { 0 }));
            }
        }
    }

    /// 9xy0: Skip next instruction if X != y
    #[test]
    fn skip_not_equals() {
        let (mut cpu, _) = setup_environment();
        for word in 0..=0xffff {
            let (a, b, addr) = (word as u8, (word >> 4) as u8, random::<u16>() & 0x7fe);
            for reg in 0..=0xff {
                let (x, y) = (reg & 0xf, reg >> 4);
                if x == y {
                    continue;
                }
                // set the PC to a random address
                cpu.pc = addr;

                (cpu.v[x], cpu.v[y]) = (a, b);

                cpu.skip_not_equals(x, y);

                assert_eq!(cpu.pc, addr.wrapping_add(if a != b { 2 } else { 0 }));
            }
        }
    }

    /// Badr: Jump to &adr + v0
    #[test]
    fn jump_indexed() {
        let (mut cpu, _) = setup_environment();
        // For every valid address
        for addr in 0..0x1000 {
            // For every valid offset
            for v0 in 0..=0xff {
                // set v[0] = v0
                cpu.v[0] = v0;

                cpu.jump_indexed(addr);

                assert_eq!(cpu.pc, addr.wrapping_add(v0.into()));
            }
        }
    }
    /// Tests `stupid_jumps` Quirk behavior
    #[test]
    fn jump_stupid() {
        let (mut cpu, _) = setup_environment();
        cpu.flags.quirks.stupid_jumps = true;

        //set v[0..F] to 0123456789abcdef
        for i in 0..0x10 {
            cpu.v[i] = i as u8;
        }
        // just WHY
        for reg in 0..0x10 {
            // attempts to jump to 0x`reg`00 + 0
            cpu.jump_indexed(reg * 0x100);
            // jumps to 0x`reg`00 + v`reg` instead
            assert_eq!(cpu.pc, reg * 0x101);
        }
    }
}

mod math {
    use super::*;
    /// 6xbb: Loads immediate byte b into register vX
    #[test]
    fn load_immediate() {
        let (mut cpu, _) = setup_environment();
        for test_register in 0x0..=0xf {
            for test_byte in 0x0..=0xff {
                cpu.load_immediate(test_register, test_byte);
                assert_eq!(cpu.v[test_register], test_byte)
            }
        }
    }

    /// 7xbb: Adds immediate byte b to register vX
    #[test]
    fn add_immediate() {
        let (mut cpu, _) = setup_environment();
        for test_register in 0x0..=0xf {
            let mut sum = 0u8;
            for test_byte in 0x0..=0xff {
                // Note: Chip-8 allows unsigned overflow
                sum = sum.wrapping_add(test_byte);

                cpu.add_immediate(test_register, test_byte);

                assert_eq!(cpu.v[test_register], sum);
            }
        }
    }

    /// 8xy0: Loads the value of y into x
    #[test]
    fn load() {
        let (mut cpu, _) = setup_environment();
        // We use zero as a sentinel value for this test, so loop from 1 to 255
        for test_value in 1..=0xff {
            for reg in 0..=0xff {
                let (x, y) = (reg & 0xf, reg >> 4);
                if x == y {
                    continue;
                }
                // Set vY to the test value
                cpu.v[y] = test_value;
                // zero X
                cpu.v[x] = 0;

                cpu.load(x, y);

                // verify results
                assert_eq!(cpu.v[x], test_value);
                assert_eq!(cpu.v[y], test_value);
            }
        }
    }
    mod or {
        use super::*;

        /// 8xy1: Performs bitwise or of vX and vY, and stores the result in vX
        #[test]
        fn or() {
            let (mut cpu, _) = setup_environment();
            for word in 0..=0xffff {
                let (a, b) = (word as u8, (word >> 4) as u8);
                let expected_result = a | b;
                for reg in 0..=0xff {
                    let (x, y) = (reg & 0xf, reg >> 4);

                    cpu.v[0xf] = 0xc5; // sentinel
                    (cpu.v[x], cpu.v[y]) = (a, b);

                    cpu.or(x, y);

                    if x != 0xf {
                        assert_eq!(cpu.v[x], if x == y { b } else { expected_result });
                    }
                    assert_eq!(cpu.v[0xf], 0);
                }
            }
        }
        /// Same test, with [Quirks::bin_ops] flag set
        #[test]
        fn or_quirk() {
            let (mut cpu, _) = setup_environment();
            cpu.flags.quirks.bin_ops = true;
            for word in 0..=0xffff {
                let (a, b) = (word as u8, (word >> 4) as u8);
                let expected_result = a | b;
                for reg in 0..=0xff {
                    let (x, y) = (reg & 0xf, reg >> 4);

                    (cpu.v[x], cpu.v[y]) = (a, b);

                    cpu.or(x, y);

                    assert_eq!(cpu.v[x], if x == y { b } else { expected_result });
                }
            }
        }
    }

    mod and {
        use super::*;
        /// 8xy2: Performs bitwise and of vX and vY, and stores the result in vX
        #[test]
        fn and() {
            let (mut cpu, _) = setup_environment();
            for word in 0..=0xffff {
                let (a, b) = (word as u8, (word >> 4) as u8);
                let expected_result = a & b;
                for reg in 0..=0xff {
                    let (x, y) = (reg & 0xf, reg >> 4);

                    cpu.v[0xf] = 0xc5; // Sentinel
                    (cpu.v[x], cpu.v[y]) = (a, b);

                    cpu.and(x, y);
                    if x != 0xf {
                        assert_eq!(cpu.v[x], if x == y { b } else { expected_result });
                    }
                    assert_eq!(cpu.v[0xf], 0)
                }
            }
        }
        // The same test with [Quirks::bin_ops] flag set
        #[test]
        fn and_quirk() {
            let (mut cpu, _) = setup_environment();
            cpu.flags.quirks.bin_ops = true;
            for word in 0..=0xffff {
                let (a, b) = (word as u8, (word >> 4) as u8);
                let expected_result = a & b;
                for reg in 0..=0xff {
                    let (x, y) = (reg & 0xf, reg >> 4);

                    (cpu.v[x], cpu.v[y]) = (a, b);

                    cpu.and(x, y);

                    assert_eq!(cpu.v[x], if x == y { b } else { expected_result });
                }
            }
        }
    }
    mod xor {
        use super::*;

        /// 8xy3: Performs bitwise xor of vX and vY, and stores the result in vX
        #[test]
        fn xor() {
            let (mut cpu, _) = setup_environment();
            for word in 0..=0xffff {
                let (a, b) = (word as u8, (word >> 4) as u8);
                let expected_result = a ^ b;
                for reg in 0..=0xff {
                    let (x, y) = (reg & 0xf, reg >> 4);

                    cpu.v[0xf] = 0xc5; // Sentinel
                    (cpu.v[x], cpu.v[y]) = (a, b);

                    cpu.xor(x, y);
                    if x != 0xf {
                        assert_eq!(cpu.v[x], if x == y { 0 } else { expected_result });
                    }
                    assert_eq!(cpu.v[0xf], 0);
                }
            }
        }
        // The same test with [Quirks::bin_ops] flag set
        #[test]
        fn xor_quirk() {
            let (mut cpu, _) = setup_environment();
            cpu.flags.quirks.bin_ops = true;
            for word in 0..=0xffff {
                let (a, b) = (word as u8, (word >> 4) as u8);
                let expected_result = a ^ b;
                for reg in 0..=0xff {
                    let (x, y) = (reg & 0xf, reg >> 4);

                    (cpu.v[x], cpu.v[y]) = (a, b);

                    cpu.xor(x, y);

                    assert_eq!(cpu.v[x], if x == y { 0 } else { expected_result });
                }
            }
        }
    }

    /// 8xy4: Performs addition of vX and vY, and stores the result in vX, carry in vF
    ///       If X is F, *only* stores borrow
    #[test]
    fn add() {
        let (mut cpu, _) = setup_environment();
        for word in 0..=0xffff {
            let (a, b) = (word as u8, (word >> 4) as u8);
            for reg in 0..=0xff {
                let (x, y) = (reg & 0xf, reg >> 4);
                // calculate the expected result
                // If x == y, a is discarded
                let (expected, carry) = if x == y { b } else { a }.overflowing_add(b);

                (cpu.v[x], cpu.v[y]) = (a, b);

                cpu.add(x, y);

                // if the destination is vF, the result was discarded, and only the carry was kept
                if x != 0xf {
                    assert_eq!(cpu.v[x], expected);
                }
                assert_eq!(cpu.v[0xf], carry.into());
            }
        }
    }

    /// 8xy5: Performs subtraction of vX and vY, and stores the result in vX, borrow in vF
    #[test]
    fn sub() {
        let (mut cpu, _) = setup_environment();
        for word in 0..=0xffff {
            let (a, b) = (word as u8, (word >> 4) as u8);
            for reg in 0..=0xff {
                let (x, y) = (reg & 0xf, reg >> 4);
                // calculate the expected result
                let (expected, carry) = if x == y { b } else { a }.overflowing_sub(b);

                (cpu.v[x], cpu.v[y]) = (a, b);

                cpu.sub(x, y);

                // if the destination is vF, the result was discarded, and only the carry was kept
                if x != 0xf {
                    assert_eq!(cpu.v[x], expected);
                }
                // The borrow flag for subtraction is inverted
                assert_eq!(cpu.v[0xf], (!carry).into());
            }
        }
    }
    mod shift_right {
        use super::*;
        /// 8xy6: Performs bitwise right shift of vX, stores carry-out in vF
        #[test]
        fn shift_right() {
            let (mut cpu, _) = setup_environment();
            for word in 0..=0xff {
                for reg in 0..=0xff {
                    let (x, y) = (reg & 0xf, reg >> 4);
                    // set the register under test to `word`
                    (cpu.v[x], cpu.v[y]) = (0, word);

                    cpu.shift_right(x, y);

                    // if the destination is vF, the result was discarded, and only the carry was kept
                    if x != 0xf {
                        assert_eq!(cpu.v[x], word >> 1);
                    }
                    // The borrow flag for subtraction is inverted
                    assert_eq!(cpu.v[0xf], word & 1);
                }
            }
        }
        /// The same test, with [Quirks::shift] quirk flag set
        #[test]
        fn shift_right_quirk() {
            let (mut cpu, _) = setup_environment();
            cpu.flags.quirks.shift = true;
            for word in 0..=0xff {
                for reg in 0..=0xff {
                    let (x, y) = (reg & 0xf, reg >> 4);
                    // set the register under test to `word`
                    cpu.v[x] = word;

                    cpu.shift_right(x, y);

                    // if the destination is vF, the result was discarded, and only the carry was kept
                    if x != 0xf {
                        assert_eq!(cpu.v[x], word >> 1);
                    }
                    // The borrow flag for subtraction is inverted
                    assert_eq!(cpu.v[0xf], word & 1);
                }
            }
        }
    }

    /// 8xy7: Performs subtraction of vY and vX, and stores the result in vX and ~carry in vF
    #[test]
    fn backwards_sub() {
        let (mut cpu, _) = setup_environment();
        for word in 0..=0xffff {
            let (a, b) = (word as u8, (word >> 4) as u8);
            for reg in 0..=0xff {
                let (x, y) = (reg & 0xf, reg >> 4);
                // calculate the expected result
                let (expected, carry) = if x == y { a } else { b }.overflowing_sub(a);
                (cpu.v[x], cpu.v[y]) = (a, b);

                cpu.backwards_sub(x, y);

                // if the destination is vF, the result was discarded, and only the carry was kept
                if x != 0xf {
                    assert_eq!(cpu.v[x], expected);
                }
                // The borrow flag for subtraction is inverted
                assert_eq!(cpu.v[0xf], (!carry).into());
            }
        }
    }

    mod shift_left {
        use super::*;
        #[test]
        fn shift_left() {
            let (mut cpu, _) = setup_environment();
            for word in 1..=0xff {
                for reg in 0..=0xff {
                    let (x, y) = (reg & 0xf, reg >> 4);
                    // set the register under test to `word`
                    (cpu.v[x], cpu.v[y]) = (0, word);

                    cpu.shift_left(x, y);

                    // if the destination is vF, the result was discarded, and only the carry was kept
                    if x != 0xf {
                        assert_eq!(cpu.v[x], word << 1);
                    }
                    // The borrow flag for subtraction is inverted
                    assert_eq!(cpu.v[0xf], word >> 7);
                }
            }
        }

        /// 8X_E: Performs bitwise left shift of vX
        // TODO: Test with authentic flag set
        #[test]
        fn shift_left_quirk() {
            let (mut cpu, _) = setup_environment();
            cpu.flags.quirks.shift = true;
            for word in 0..=0xff {
                for x in 0..=0xf {
                    // set the register under test to `word`
                    cpu.v[x] = word;

                    cpu.shift_left(x, x);

                    // if the destination is vF, the result was discarded, and only the carry was kept
                    if x != 0xf {
                        assert_eq!(cpu.v[x], word << 1);
                    }
                    // The borrow flag for subtraction is inverted
                    assert_eq!(cpu.v[0xf], word >> 7);
                }
            }
        }
    }
}

/// Test operations on the index/indirect register, I
mod i {
    use super::*;
    /// Aadr: Load address #adr into register I
    #[test]
    fn load_i_immediate() {
        let (mut cpu, _) = setup_environment();
        for addr in 0..0x1000 {
            // Load indirect register
            cpu.load_i_immediate(addr);
            // Validate register set to addr
            assert_eq!(cpu.i, addr);
        }
    }

    /// Fx1e: Add vX to I
    #[test]
    fn add_i() {
        let (mut cpu, _) = setup_environment();
        // For every valid address
        for addr in 0..0x1000 {
            // For every valid offset
            for x in 0..=0xfff {
                let (x, byte) = (x >> 8, x as u8);
                // set v[x] = byte
                (cpu.i, cpu.v[x]) = (addr as u16, byte);
                // add vX to indirect register
                cpu.add_i(x);
                // Validate register set
                assert_eq!(cpu.i, (addr + byte as usize) as u16)
            }
        }
    }
}

/// Screen, buttons, other things that would be peripherals on a real architecture
/// # Includes:
/// - Random number generation
/// - Drawing to the display
mod io {
    use super::*;
    use std::io::Write;

    /// Cxbb: Stores a random number & the provided byte into vX
    #[test]
    fn rand() {
        let (mut cpu, _) = setup_environment();
        for xb in 0..0x100fff {
            let (x, b) = ((xb >> 8) % 16, xb as u8);
            cpu.v[x] = 0;
            cpu.rand(x, b);
            // We don't know what the number will be,
            // but we do know it'll be <= b
            assert!(cpu.v[x] <= b);
        }
    }

    mod display {
        use super::*;
        struct ScreenTest {
            program: &'static [u8],
            screen: &'static [u8],
            steps: usize,
            quirks: Quirks,
        }

        const SCREEN_TESTS: [ScreenTest; 3] = [
            // The IBM Logo
            // # Quirks
            // Originally timed without quirks
            ScreenTest {
                program: include_bytes!("../../chip-8/IBM Logo.ch8"),
                screen: include_bytes!("tests/screens/IBM Logo.ch8/20.bin"),
                steps: 56,
                quirks: Quirks {
                    bin_ops: false,
                    shift: false,
                    draw_wait: false,
                    screen_wrap: true,
                    dma_inc: false,
                    stupid_jumps: false,
                },
            },
            // Rule 22 cellular automata
            // # Quirks
            // - Requires draw_wait false, or it just takes AGES.
            ScreenTest {
                program: include_bytes!("../../chip-8/1dcell.ch8"),
                screen: include_bytes!("tests/screens/1dcell.ch8/123342.bin"),
                steps: 123342,
                quirks: Quirks {
                    bin_ops: false,
                    shift: false,
                    draw_wait: true,
                    screen_wrap: true,
                    dma_inc: false,
                    stupid_jumps: false,
                },
            },
            // Rule 60 cellular automata
            ScreenTest {
                program: include_bytes!("../../chip-8/1dcell.ch8"),
                screen: include_bytes!("tests/screens/1dcell.ch8/2391162.bin"),
                steps: 2391162,
                quirks: Quirks {
                    bin_ops: false,
                    shift: false,
                    draw_wait: true,
                    screen_wrap: true,
                    dma_inc: false,
                    stupid_jumps: false,
                },
            },
        ];

        /// Dxyn: Draws n-byte sprite to the screen at coordinates (vX, vY)
        #[test]
        fn draw() {
            for test in SCREEN_TESTS {
                let (mut cpu, mut bus) = setup_environment();
                cpu.flags.quirks = test.quirks;
                // Debug mode is 5x slower
                cpu.flags.debug = false;
                // Load the test program
                cpu.screen.load_region(Program, test.program).unwrap();
                // Run the test program for the specified number of steps
                while cpu.cycle() < test.steps {
                    cpu.multistep(&mut bus, 10.min(test.steps - cpu.cycle()))
                        .expect("Draw tests should not contain undefined instructions");
                }
                // Compare the screen to the reference screen buffer
                bus.print_screen()
                    .expect("Printing screen should not fail if screen exists");
                print_screen(test.screen);
                assert_eq!(
                    bus.get_region(Screen)
                        .expect("Getting screen should not fail if screen exists"),
                    test.screen
                );
            }
        }
    }

    mod cf {
        use super::*;
        /// Ex9E: Skip next instruction if key == #X
        #[test]
        fn skip_key_equals() {
            let (mut cpu, _) = setup_environment();
            for ka in 0..=0x7fef {
                let (key, addr) = ((ka & 0xf) as u8, ka >> 8);
                // positive test (no keys except key pressed)
                cpu.keys = [false; 16]; // unset all keys
                cpu.keys[key as usize] = true; // except the one we care about
                for x in 0..=0xf {
                    cpu.pc = addr;
                    cpu.v[x] = key;
                    cpu.skip_key_equals(x);
                    assert_eq!(cpu.pc, addr.wrapping_add(2));
                    cpu.v[x] = 0xff;
                }
                // negative test (all keys except key pressed)
                cpu.keys = [true; 16]; // set all keys
                cpu.keys[key as usize] = false; // except the one we care about
                for x in 0..=0xf {
                    cpu.pc = addr;
                    cpu.v[x] = key;
                    cpu.skip_key_equals(x);
                    assert_eq!(cpu.pc, addr);
                    cpu.v[x] = 0xff;
                }
            }
        }

        /// ExaE: Skip next instruction if key != #X
        #[test]
        fn skip_key_not_equals() {
            let (mut cpu, _) = setup_environment();
            for ka in 0..=0x7fcf {
                let (key, addr) = ((ka & 0xf) as u8, ka >> 8);
                // positive test (no keys except key pressed)
                cpu.keys = [false; 16]; // unset all keys
                cpu.keys[key as usize] = true; // except the one we care about
                for x in 0..=0xf {
                    cpu.pc = addr;
                    cpu.v[x] = key;
                    cpu.skip_key_not_equals(x);
                    assert_eq!(cpu.pc, addr);
                    cpu.v[x] = 0xff;
                }
                // negative test (all keys except key pressed)
                cpu.keys = [true; 16]; // set all keys
                cpu.keys[key as usize] = false; // except the one we care about
                for x in 0..=0xf {
                    cpu.pc = addr;
                    cpu.v[x] = key;
                    cpu.skip_key_not_equals(x);
                    assert_eq!(cpu.pc, addr.wrapping_add(2));
                    cpu.v[x] = 0xff;
                }
            }
        }

        /// Fx0A: Wait for key, then vX = K
        ///
        /// The write happens on key *release*
        #[test]
        fn wait_for_key() {
            let (mut cpu, _) = setup_environment();
            for key in 0..0xf {
                for x in 0..0xf {
                    cpu.v[x] = 0xff;
                    cpu.wait_for_key(x);
                    assert!(cpu.flags.keypause);
                    assert_eq!(0xff, cpu.v[x]);
                    // There are three parts to a button press
                    // When the button is pressed
                    assert!(cpu.press(key).expect("Key should be pressed"));
                    assert!(!cpu.press(key).expect("Key shouldn't be pressed again"));
                    assert!(cpu.flags.keypause);
                    assert_eq!(0xff, cpu.v[x]);
                    // When the button is held
                    cpu.wait_for_key(x);
                    assert!(cpu.flags.keypause);
                    assert_eq!(0xff, cpu.v[x]);
                    // And when the button is released!
                    assert!(cpu.release(key).expect("Key should be released"));
                    assert!(!cpu.release(key).expect("Key shouldn't be released again"));
                    assert!(!cpu.flags.keypause);
                    assert_eq!(Some(key), cpu.flags.lastkey);
                    cpu.wait_for_key(x);
                    assert_eq!(key as u8, cpu.v[x]);
                }
            }
        }
    }

    /// Fx07: Get the current DT, and put it in vX
    #[test]
    fn get_delay_timer() {
        let (mut cpu, _) = setup_environment();
        for word in 0..=0xff {
            for x in 0..=0xf {
                // set the register under test to `word`
                cpu.delay = word as f64;

                cpu.load_delay_timer(x);

                assert_eq!(cpu.v[x], word);
            }
        }
    }

    /// Fx15: Load vX into DT
    #[test]
    fn load_delay_timer() {
        let (mut cpu, _) = setup_environment();
        for word in 0..=0xff {
            for x in 0..=0xf {
                // set the register under test to `word`
                cpu.v[x] = word;

                cpu.store_delay_timer(x);

                assert_eq!(cpu.delay, word as f64);
            }
        }
    }

    /// Fx18: Load vX into ST
    #[test]
    fn store_sound_timer() {
        let (mut cpu, _) = setup_environment();
        for word in 0..=0xff {
            for x in 0..=0xf {
                // set the register under test to `word`
                cpu.v[x] = word;

                cpu.store_sound_timer(x);

                assert_eq!(cpu.sound, word as f64);
            }
        }
    }

    mod sprite {
        use super::*;

        struct SpriteTest {
            input: u8,
            output: &'static [u8],
        }

        /// Verify the character sprite addresses with the data they should return
        #[rustfmt::skip]
        const TESTS: [SpriteTest; 16] = [
            SpriteTest { input: 0x0, output: &[0xf0, 0x90, 0x90, 0x90, 0xf0] },
            SpriteTest { input: 0x1, output: &[0x20, 0x60, 0x20, 0x20, 0x70] },
            SpriteTest { input: 0x2, output: &[0xf0, 0x10, 0xf0, 0x80, 0xf0] },
            SpriteTest { input: 0x3, output: &[0xf0, 0x10, 0xf0, 0x10, 0xf0] },
            SpriteTest { input: 0x4, output: &[0x90, 0x90, 0xf0, 0x10, 0x10] },
            SpriteTest { input: 0x5, output: &[0xf0, 0x80, 0xf0, 0x10, 0xf0] },
            SpriteTest { input: 0x6, output: &[0xf0, 0x80, 0xf0, 0x90, 0xf0] },
            SpriteTest { input: 0x7, output: &[0xf0, 0x10, 0x20, 0x40, 0x40] },
            SpriteTest { input: 0x8, output: &[0xf0, 0x90, 0xf0, 0x90, 0xf0] },
            SpriteTest { input: 0x9, output: &[0xf0, 0x90, 0xf0, 0x10, 0xf0] },
            SpriteTest { input: 0xa, output: &[0xf0, 0x90, 0xf0, 0x90, 0x90] },
            SpriteTest { input: 0xb, output: &[0xe0, 0x90, 0xe0, 0x90, 0xe0] },
            SpriteTest { input: 0xc, output: &[0xf0, 0x80, 0x80, 0x80, 0xf0] },
            SpriteTest { input: 0xd, output: &[0xe0, 0x90, 0x90, 0x90, 0xe0] },
            SpriteTest { input: 0xe, output: &[0xf0, 0x80, 0xf0, 0x80, 0xf0] },
            SpriteTest { input: 0xf, output: &[0xf0, 0x80, 0xf0, 0x80, 0x80] },
        ];

        /// Fx29: Load sprite for character vX into I
        #[test]
        fn load_sprite() {
            let (mut cpu, _) = setup_environment();
            for test in TESTS {
                let reg = 0xf & random::<usize>();
                // load number into CPU register
                cpu.v[reg] = test.input;

                cpu.load_sprite(reg);

                let addr = cpu.i as usize;
                assert_eq!(
                    cpu.screen
                        .get(addr..addr.wrapping_add(5))
                        .expect("Region at addr should exist!"),
                    test.output,
                );
            }
        }
    }

    mod bcdtest {
        pub(self) use super::*;

        struct BCDTest {
            // value to test
            input: u8,
            // result
            output: &'static [u8],
        }

        const BCD_TESTS: [BCDTest; 3] = [
            BCDTest {
                input: 000,
                output: &[0, 0, 0],
            },
            BCDTest {
                input: 255,
                output: &[2, 5, 5],
            },
            BCDTest {
                input: 127,
                output: &[1, 2, 7],
            },
        ];

        /// Fx33: BCD convert X into I`[0..3]`
        #[test]
        fn bcd_convert() {
            for test in BCD_TESTS {
                let (mut cpu, _) = setup_environment();
                let addr = 0xff0 & random::<u16>() as usize;
                // load CPU registers
                cpu.i = addr as u16;
                cpu.v[5] = test.input;

                cpu.bcd_convert(5);

                assert_eq!(
                    cpu.screen.get(addr..addr.saturating_add(3)),
                    Some(test.output)
                )
            }
        }
    }

    /// Fx55: DMA Stor from I to registers 0..X
    // TODO: Test with dma_inc quirk set
    #[test]
    fn dma_store() {
        let (mut cpu, _) = setup_environment();
        const DATA: &[u8] = b"ABCDEFGHIJKLMNOP";
        // Load some test data into memory
        let addr = 0x456;
        cpu.v
            .as_mut_slice()
            .write_all(DATA)
            .expect("Loading test data should succeed");
        for len in 0..16 {
            // Perform DMA store
            cpu.i = addr as u16;
            cpu.store_dma(len);
            // Check that bus grabbed the correct data
            let bus = cpu
                .screen
                .get_mut(addr..addr + DATA.len())
                .expect("Getting a mutable slice at addr 0x0456 should not fail");
            assert_eq!(bus[0..=len], DATA[0..=len]);
            assert_eq!(bus[len + 1..], [0; 16][len + 1..]);
            // clear
            bus.fill(0);
        }
    }

    /// Fx65: DMA Load from I to registers 0..X
    // TODO: Test with dma_inc quirk set
    #[test]
    fn dma_load() {
        let (mut cpu, _) = setup_environment();
        const DATA: &[u8] = b"ABCDEFGHIJKLMNOP";
        // Load some test data into memory
        let addr = 0x456;
        cpu.screen
            .get_mut(addr..addr + DATA.len())
            .expect("Getting a mutable slice at addr 0x0456..0x0466 should not fail")
            .write_all(DATA)
            .unwrap();
        for len in 0..16 {
            // Perform DMA load
            cpu.i = addr as u16;
            cpu.load_dma(len);
            // Check that registers grabbed the correct data
            assert_eq!(cpu.v[0..=len], DATA[0..=len]);
            assert_eq!(cpu.v[len + 1..], [0; 16][len + 1..]);
            // clear
            cpu.v.fill(0);
        }
    }
}

mod behavior {
    use super::*;

    mod realtime {
        use super::*;
        use std::time::Duration;
        #[test]
        fn delay() {
            let (mut cpu, mut bus) = setup_environment();
            cpu.flags.monotonic = None;
            cpu.delay = 10.0;
            for _ in 0..2 {
                cpu.multistep(&mut bus, 8)
                    .expect("Running valid instructions should always succeed");
                std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
            }
            // time is within 1 frame deviance over a theoretical 2 frame pause
            assert!(7.0 <= cpu.delay && cpu.delay <= 9.0);
        }
        #[test]
        fn sound() {
            let (mut cpu, mut bus) = setup_environment();
            cpu.flags.monotonic = None; // disable monotonic timing
            cpu.sound = 10.0;
            for _ in 0..2 {
                cpu.multistep(&mut bus, 8)
                    .expect("Running valid instructions should always succeed");
                std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
            }
            // time is within 1 frame deviance over a theoretical 2 frame pause
            assert!(7.0 <= cpu.sound && cpu.sound <= 9.0);
        }
        #[test]
        fn vbi_wait() {
            let (mut cpu, mut bus) = setup_environment();
            cpu.flags.monotonic = None; // disable monotonic timing
            cpu.flags.draw_wait = true;
            for _ in 0..2 {
                cpu.multistep(&mut bus, 8)
                    .expect("Running valid instructions should always succeed");
                std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
            }
            // Display wait is disabled after a 1 frame pause
            assert!(!cpu.flags.draw_wait);
        }
    }
    mod breakpoint {

        use super::*;
        #[test]
        #[cfg_attr(feature = "unstable", no_coverage)]
        fn hit_break() {
            let (mut cpu, mut bus) = setup_environment();
            cpu.set_break(0x202);
            match cpu.multistep(&mut bus, 10) {
                Err(crate::error::Error::BreakpointHit { addr, next }) => {
                    assert_eq!(0x202, addr); // current address is 202
                    assert_eq!(0x1204, next); // next insn is `jmp 204`
                }
                other => unreachable!("{:?}", other),
            }
            assert!(cpu.flags.pause);
            assert_eq!(0x202, cpu.pc);
        }
        #[test]
        #[cfg_attr(feature = "unstable", no_coverage)]
        fn hit_break_singlestep() {
            let (mut cpu, mut bus) = setup_environment();
            cpu.set_break(0x202);
            match cpu.singlestep(&mut bus) {
                Err(crate::error::Error::BreakpointHit { addr, next }) => {
                    assert_eq!(0x202, addr); // current address is 202
                    assert_eq!(0x1204, next); // next insn is `jmp 204`
                }
                other => unreachable!("{:?}", other),
            }
            assert!(cpu.flags.pause);
            assert_eq!(0x202, cpu.pc);
        }
    }

    #[test]
    #[cfg_attr(feature = "unstable", no_coverage)]
    fn invalid_pc() {
        let (mut cpu, mut bus) = setup_environment();
        // The bus extends from 0x0..0x1000
        cpu.pc = 0xfff;
        match cpu.tick(&mut bus) {
            Err(Error::InvalidAddressRange { range }) => {
                eprintln!("InvalidBusRange {{ {range:04x?} }}")
            }
            other => unreachable!("{other:04x?}"),
        }
    }
}
