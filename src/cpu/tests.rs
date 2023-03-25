use super::*;
use crate::{
    bus,
    bus::{Bus, Region::*},
};

fn setup_environment() -> (CPU, Bus) {
    (
        CPU {
            flags: ControlFlags {
                debug: true,
                pause: false,
                ..Default::default()
            },
            ..CPU::default()
        },
        bus! {
            // Load the charset into ROM
            Charset [0x0050..0x00A0] = include_bytes!("../mem/charset.bin"),
            // Load the ROM file into RAM
            Program [0x0200..0x1000] = include_bytes!("../../chip-8/BC_test.ch8"),
            // Create a screen
            Screen  [0x0F00..0x1000] = include_bytes!("../../chip-8/IBM Logo.ch8"),
        },
    )
}

/// Unused instructions
#[test]
#[should_panic]
fn unimplemented() {
    let (mut cpu, mut bus) = setup_environment();
    bus.write(0x200u16, 0xffffu16); // 0xffff is not an instruction
    cpu.tick(&mut bus);
    cpu.unimplemented(0xffff);
}

/// 0aaa: Handles a "machine language function call" (lmao)
#[test]
#[should_panic]
fn sys() {
    let (mut cpu, mut bus) = setup_environment();
    bus.write(0x200u16, 0x0200u16); // 0x0200 is not one of the defined ML routines
    cpu.tick(&mut bus);
    cpu.sys(0x200);
}

/// 00e0: Clears the screen memory to 0
#[test]
fn clear_screen() {
    let (mut cpu, mut bus) = setup_environment();
    bus.write(0x200u16, 0x00e0u16);
    // Check if screen RAM is cleared
    cpu.tick(&mut bus);
    bus.get_region(Screen)
        .expect("Expected screen, got None")
        .iter()
        .for_each(|byte| assert_eq!(*byte, 0));
}

/// 00ee: Returns from subroutine
#[test]
fn ret() {
    let test_addr = random::<u16>() & 0x7ff;
    let (mut cpu, mut bus) = setup_environment();
    let sp_orig = cpu.sp;
    // Place the address on the stack
    bus.write(cpu.sp.wrapping_add(2), test_addr);
    // Call an address
    cpu.ret(&mut bus);
    // Verify the current address is the address from the stack
    assert_eq!(test_addr, cpu.pc);
    assert!(dbg!(cpu.sp.wrapping_sub(sp_orig)) == 0x2);
    // Verify the stack pointer has moved
}

/// 1aaa: Sets the program counter to an absolute address
#[test]
fn jump() {
    // Generate a random test address that's not 0x200
    let test_addr = random::<u16>() & !0x200;
    let (mut cpu, _) = setup_environment();
    // Call an address
    cpu.jump(test_addr);
    // Verify the current address is the called address
    assert_eq!(test_addr, cpu.pc);
}

/// 2aaa: Pushes pc onto the stack, then jumps to a
#[test]
fn call() {
    let test_addr = random::<u16>();
    let (mut cpu, mut bus) = setup_environment();
    // Save the current address
    let curr_addr = cpu.pc;
    // Call an address
    cpu.call(test_addr, &mut bus);
    // Verify the current address is the called address
    assert_eq!(test_addr, cpu.pc);
    // Verify the previous address was stored on the stack (sp+2)
    let stack_addr: u16 = bus.read(cpu.sp.wrapping_add(2));
    assert_eq!(stack_addr, curr_addr);
}

/// 3xbb: Skips the next instruction if register X == b
#[test]
fn skip_if_x_equal_byte() {
    let (mut cpu, _) = setup_environment();
    for word in 0..=0xffff {
        let (a, b, addr) = (word as u8, (word >> 4) as u8, random::<u16>() & 0x7fe);
        for x in 0..=0xf {
            // set the PC to a random address
            cpu.pc = addr;
            // set the register under test to a
            cpu.v[x] = a;
            // do the thing
            cpu.skip_if_x_equal_byte(x, b);
            // validate the result
            assert_eq!(cpu.pc, addr.wrapping_add(if dbg!(a == b) { 2 } else { 0 }));
        }
    }
}

/// 4xbb: Skips the next instruction if register X != b
#[test]
fn skip_if_x_not_equal_byte() {
    let (mut cpu, _) = setup_environment();
    for word in 0..=0xffff {
        let (a, b, addr) = (word as u8, (word >> 4) as u8, random::<u16>() & 0x7fe);
        for x in 0..=0xf {
            // set the PC to a random address
            cpu.pc = addr;
            // set the register under test to a
            cpu.v[x] = a;
            // do the thing
            cpu.skip_if_x_not_equal_byte(x, b);
            // validate the result
            assert_eq!(cpu.pc, addr.wrapping_add(if a != b { 2 } else { 0 }));
        }
    }
}

/// 5xy0: Skips the next instruction if register X != register Y
#[test]
fn skip_if_x_equal_y() {
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
            // set the registers under test to a, b
            (cpu.v[x], cpu.v[y]) = (a, b);
            // do the thing
            cpu.skip_if_x_equal_y(x, y);
            // validate the result
            assert_eq!(cpu.pc, addr.wrapping_add(if a == b { 2 } else { 0 }));
        }
    }
}
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
            // Wrapping-add to the running total (Chip-8 allows unsigned overflow)
            sum = sum.wrapping_add(test_byte);
            // Perform add #byte, vReg
            cpu.add_immediate(test_register, test_byte);
            //Verify the running total in the register matches
            assert_eq!(cpu.v[test_register], sum);
        }
    }
}

/// 8xy0: Loads the value of y into x
#[test]
fn load_y_into_x() {
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
            cpu.load_y_into_x(x, y);
            // verify results
            assert_eq!(cpu.v[x], test_value);
            assert_eq!(cpu.v[y], test_value);
        }
    }
}

/// 8xy1: Performs bitwise or of vX and vY, and stores the result in vX
#[test]
fn x_orequals_y() {
    let (mut cpu, _) = setup_environment();
    for word in 0..=0xffff {
        let (a, b) = (word as u8, (word >> 4) as u8);
        let expected_result = a | b;
        for reg in 0..=0xff {
            let (x, y) = (reg & 0xf, reg >> 4);
            // set the registers under test to a, b
            (cpu.v[x], cpu.v[y]) = (a, b);

            // do the thing
            cpu.x_orequals_y(x, y);

            // validate the result
            assert_eq!(cpu.v[x], if x == y { b } else { expected_result });
        }
    }
}

/// 8xy2: Performs bitwise and of vX and vY, and stores the result in vX
#[test]
fn x_andequals_y() {
    let (mut cpu, _) = setup_environment();
    for word in 0..=0xffff {
        let (a, b) = (word as u8, (word >> 4) as u8);
        let expected_result = a & b;
        for reg in 0..=0xff {
            let (x, y) = (reg & 0xf, reg >> 4);
            // set the registers under test to a, b
            (cpu.v[x], cpu.v[y]) = (a, b);

            // do the thing
            cpu.x_andequals_y(x, y);

            // validate the result
            assert_eq!(cpu.v[x], if x == y { b } else { expected_result });
        }
    }
}

/// 8xy3: Performs bitwise xor of vX and vY, and stores the result in vX
#[test]
fn x_xorequals_y() {
    let (mut cpu, _) = setup_environment();
    for word in 0..=0xffff {
        let (a, b) = (word as u8, (word >> 4) as u8);
        let expected_result = a ^ b;
        for reg in 0..=0xff {
            let (x, y) = (reg & 0xf, reg >> 4);
            // set the registers under test to a, b
            (cpu.v[x], cpu.v[y]) = (a, b);

            // do the thing
            cpu.x_xorequals_y(x, y);

            // validate the result
            assert_eq!(cpu.v[x], if x == y { 0 } else { expected_result });
        }
    }
}

/// 8xy4: Performs addition of vX and vY, and stores the result in vX, carry in vF
///       If X is F, *only* stores borrow
#[test]
fn x_addequals_y() {
    let (mut cpu, _) = setup_environment();
    for word in 0..=0xffff {
        let (a, b) = (word as u8, (word >> 4) as u8);
        for reg in 0..=0xff {
            let (x, y) = (reg & 0xf, reg >> 4);
            // calculate the expected result
            // If x == y, a is discarded
            let (expected, carry) = if x == y { b } else { a }.overflowing_add(b);
            // set the registers under test to a, b
            (cpu.v[x], cpu.v[y]) = (a, b);

            // do the thing
            cpu.x_addequals_y(x, y);

            // validate the result
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
fn x_subequals_y() {
    let (mut cpu, _) = setup_environment();
    for word in 0..=0xffff {
        let (a, b) = (word as u8, (word >> 4) as u8);
        for reg in 0..=0xff {
            let (x, y) = (reg & 0xf, reg >> 4);
            // calculate the expected result
            let (expected, carry) = if x == y { b } else { a }.overflowing_sub(b);
            // set the registers under test to a, b
            (cpu.v[x], cpu.v[y]) = (a, b);

            // do the thing
            cpu.x_subequals_y(x, y);

            // validate the result
            // if the destination is vF, the result was discarded, and only the carry was kept
            if x != 0xf {
                assert_eq!(cpu.v[x], expected);
            }
            // The borrow flag for subtraction is inverted
            assert_eq!(cpu.v[0xf], (!carry).into());
        }
    }
}

/// 8xy6: Performs bitwise right shift of vX, stores carry-out in vF
#[test]
fn shift_right_x() {
    let (mut cpu, _) = setup_environment();
    for word in 0..=0xff {
        for x in 0..=0xf {
            // set the register under test to `word`
            cpu.v[x] = word;
            // calculate the expected result
            let expected = word >> 1;
            // do the thing
            cpu.shift_right_x(x);

            // validate the result
            // if the destination is vF, the result was discarded, and only the carry was kept
            if x != 0xf {
                assert_eq!(cpu.v[x], expected);
            }
            // The borrow flag for subtraction is inverted
            assert_eq!(cpu.v[0xf], word & 1);
        }
    }
}

/// 8xy7: Performs subtraction of vY and vX, and stores the result in vX and ~carry in vF
#[test]
fn backwards_subtract() {
    let (mut cpu, _) = setup_environment();
    for word in 0..=0xffff {
        let (a, b) = (word as u8, (word >> 4) as u8);
        for reg in 0..=0xff {
            let (x, y) = (reg & 0xf, reg >> 4);
            // calculate the expected result
            let (expected, carry) = if x == y { a } else { b }.overflowing_sub(a);
            // set the registers under test to a, b
            (cpu.v[x], cpu.v[y]) = (a, b);

            // do the thing
            cpu.backwards_subtract(x, y);

            // validate the result
            // if the destination is vF, the result was discarded, and only the carry was kept
            if x != 0xf {
                assert_eq!(cpu.v[x], expected);
            }
            // The borrow flag for subtraction is inverted
            assert_eq!(cpu.v[0xf], (!carry).into());
        }
    }
}

/// 8X_E: Performs bitwise left shift of vX
#[test]
fn shift_left_x() {
    let (mut cpu, _) = setup_environment();
    for word in 0..=0xff {
        for x in 0..=0xf {
            // set the register under test to `word`
            cpu.v[x] = word;
            // calculate the expected result
            let expected = word << 1;
            // do the thing
            cpu.shift_left_x(x);

            // validate the result
            // if the destination is vF, the result was discarded, and only the carry was kept
            if x != 0xf {
                assert_eq!(cpu.v[x], expected);
            }
            // The borrow flag for subtraction is inverted
            assert_eq!(cpu.v[0xf], word >> 7);
        }
    }
}

/// 9xy0: Skip next instruction if X != y
#[test]
fn skip_if_x_not_equal_y() {
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
            // set the registers under test to a, b
            (cpu.v[x], cpu.v[y]) = (a, b);
            // do the thing
            cpu.skip_if_x_not_equal_y(x, y);
            // validate the result
            assert_eq!(cpu.pc, addr.wrapping_add(if a != b { 2 } else { 0 }));
        }
    }
}

/// Aadr: Load address #adr into register I
#[test]
fn load_indirect_register() {
    let (mut cpu, _) = setup_environment();
    // For every valid address
    for addr in 0..0x1000 {
        // Load indirect register
        cpu.load_indirect_register(addr);
        // Validate register set
        assert_eq!(cpu.i, addr);
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
            // jump indexed
            cpu.jump_indexed(addr);
            // Validate register set
            assert_eq!(cpu.pc, addr.wrapping_add(v0.into()));
        }
    }
}

/// Cxbb: Stores a random number & the provided byte into vX
//#[test]
#[allow(dead_code)]
fn rand() {
    todo!()
}

struct ScreenTest {
    program: &'static [u8],
    screen: &'static [u8],
    steps: usize,
    rate: usize,
}

const SCREEN_TESTS: [ScreenTest; 4] = [
    // Passing BC_test
    ScreenTest {
        program: include_bytes!("../../chip-8/BC_test.ch8"),
        screen: include_bytes!("tests/BC_test.ch8_197.bin"),
        steps: 197,
        rate: 8,
    },
    // The IBM Logo
    ScreenTest {
        program: include_bytes!("../../chip-8/IBM Logo.ch8"),
        screen: include_bytes!("tests/IBM Logo.ch8_20.bin"),
        steps: 20,
        rate: 8,
    },
    // Rule 22 cellular automata
    ScreenTest {
        program: include_bytes!("../../chip-8/1dcell.ch8"),
        screen: include_bytes!("tests/1dcell.ch8_123342.bin"),
        steps: 123342,
        rate: 8,
    },
    // Rule 60 cellular automata
    ScreenTest {
        program: include_bytes!("../../chip-8/1dcell.ch8"),
        screen: include_bytes!("tests/1dcell.ch8_2391162.bin"),
        steps: 2391162,
        rate: 8,
    },
];

/// Dxyn: Draws n-byte sprite to the screen at coordinates (vX, vY)
#[test]
fn draw() {
    for test in SCREEN_TESTS {
        let (mut cpu, mut bus) = setup_environment();
        // Load the test program
        bus = bus.load_region(Program, test.program);
        // Run the test program for the specified number of steps
        cpu.multistep(&mut bus, test.steps, test.rate);
        // Compare the screen to the reference screen buffer
        assert_eq!(bus.get_region(Screen).unwrap(), test.screen);
    }
}

/// Ex9E: Skip next instruction if key == #X
//#[test]
#[allow(dead_code)]
fn skip_if_key_equals_x() {
    todo!()
}

/// ExaE: Skip next instruction if key != #X
//#[test]
#[allow(dead_code)]
fn skip_if_key_not_x() {
    todo!()
}

/// Fx07: Get the current DT, and put it in vX
/// ```py
/// vX = DT
/// ```
#[test]
fn get_delay_timer() {
    let (mut cpu, _) = setup_environment();
    for word in 0..=0xff {
        for x in 0..=0xf {
            // set the register under test to `word`
            cpu.delay = word;
            // do the thing
            cpu.get_delay_timer(x);
            // validate the result
            assert_eq!(cpu.v[x], word);
        }
    }
}

/// Fx0A: Wait for key, then vX = K
//#[test]
#[allow(dead_code)]
fn wait_for_key() {
    todo!()
}

/// Fx15: Load vX into DT
/// ```py
/// DT = vX
/// ```
#[test]
fn load_delay_timer() {
    let (mut cpu, _) = setup_environment();
    for word in 0..=0xff {
        for x in 0..=0xf {
            // set the register under test to `word`
            cpu.v[x] = word;
            // do the thing
            cpu.load_delay_timer(x);
            // validate the result
            assert_eq!(cpu.delay, word);
        }
    }
}

/// Fx18: Load vX into ST
/// ```py
/// ST = vX;
/// ```
#[test]
fn load_sound_timer() {
    let (mut cpu, _) = setup_environment();
    for word in 0..=0xff {
        for x in 0..=0xf {
            // set the register under test to `word`
            cpu.v[x] = word;
            // do the thing
            cpu.load_sound_timer(x);
            // validate the result
            assert_eq!(cpu.sound, word);
        }
    }
}

/// Fx1e: Add vX to I,
/// ```py
/// I += vX;
/// ```
#[test]
fn add_to_indirect() {
    let (mut cpu, _) = setup_environment();
    // For every valid address
    for addr in 0..0x1000 {
        // For every valid offset
        for x in 0..=0xfff {
            let (x, byte) = (x >> 8, x as u8);
            // set v[x] = byte
            (cpu.i, cpu.v[x]) = (addr as u16, byte);
            // add vX to indirect register
            cpu.add_to_indirect(x);
            // Validate register set
            assert_eq!(cpu.i, (addr + byte as usize) as u16)
        }
    }
}

/// Fx29: Load sprite for character vX into I
/// ```py
/// I = sprite(vX);
/// ```
//#[test]
#[allow(dead_code)]
fn load_sprite_x() {
    todo!()
}

/// Fx33: BCD convert X into I`[0..3]`
//#[test]
#[allow(dead_code)]
fn bcd_convert_i() {
    todo!()
}

/// Fx55: DMA Stor from I to registers 0..X
//#[test]
#[allow(dead_code)]
fn dma_store() {
    todo!()
    // Load values into registers
    // Perform DMA store
    // Check that
}

/// Fx65: DMA Load from I to registers 0..X
//#[test]
#[allow(dead_code)]
fn dma_load() {
    todo!()
    // Perform DMA load
    // Check that registers grabbed the correct data
}
