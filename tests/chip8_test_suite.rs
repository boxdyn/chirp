// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)
// When compiled, the resulting binary is licensed under version 3 of the GNU General Public License (see chip8-test-suite/LICENSE for details)

//! These are a series of interpreter tests using Timendus's incredible test suite

pub use chirp::*;

fn setup_environment() -> (CPU, Screen) {
    let mut cpu = CPU::default();
    cpu.flags = Flags {
        debug: true,
        pause: false,
        ..Default::default()
    };
    (cpu, Screen::default())
}

struct SuiteTest {
    test: u8,
    data: &'static [u8],
    screen: &'static [u8],
}

fn run_screentest(test: SuiteTest, mut cpu: CPU, mut screen: Screen) {
    // Set the test to run
    cpu.poke(0x1ffu16, test.test);
    cpu.load_program_bytes(test.data).unwrap();
    // The test suite always initiates a keypause on test completion
    while !(cpu.flags.is_paused()) {
        cpu.multistep(&mut screen, 10).unwrap();
        if cpu.cycle() > 1000000 {
            panic!("test {} took too long", test.test)
        }
    }
    // Compare the screen to the reference screen buffer
    screen.print_screen();
    Screen::from(test.screen).print_screen();
    assert_eq!(screen.grab(..).unwrap(), test.screen);
}

#[test]
fn splash_screen() {
    let (cpu, bus) = setup_environment();
    run_screentest(
        SuiteTest {
            test: 0,
            data: include_bytes!("../chip8-test-suite/bin/1-chip8-logo.ch8"),
            screen: include_bytes!("screens/chip8-test-suite/splash.bin"),
        },
        cpu,
        bus,
    )
}

#[test]
fn ibm_logo() {
    let (cpu, bus) = setup_environment();
    run_screentest(
        SuiteTest {
            test: 0x00,
            data: include_bytes!("../chip8-test-suite/bin/2-ibm-logo.ch8"),
            screen: include_bytes!("screens/chip8-test-suite/IBM.bin"),
        },
        cpu,
        bus,
    )
}

#[test]
fn corax_test() {
    let (cpu, bus) = setup_environment();
    run_screentest(
        SuiteTest {
            test: 0x00,
            data: include_bytes!("../chip8-test-suite/bin/3-corax+.ch8"),
            screen: include_bytes!("screens/chip8-test-suite/corax+.bin"),
        },
        cpu,
        bus,
    )
}

#[test]
fn flags_test() {
    let (cpu, bus) = setup_environment();
    run_screentest(
        SuiteTest {
            test: 0x00,
            data: include_bytes!("../chip8-test-suite/bin/4-flags.ch8"),
            screen: include_bytes!("screens/chip8-test-suite/flags.bin"),
        },
        cpu,
        bus,
    )
}

#[test]
fn quirks_test() {
    let (cpu, bus) = setup_environment();
    run_screentest(
        SuiteTest {
            test: 0x01,
            data: include_bytes!("../chip8-test-suite/bin/5-quirks.ch8"),
            screen: include_bytes!("screens/chip8-test-suite/quirks.bin"),
        },
        cpu,
        bus,
    )
}
