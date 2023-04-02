//! These are a series of interpreter tests using Timendus's incredible test suite

pub use chirp::*;

fn setup_environment() -> (CPU, Bus) {
    let mut cpu = CPU::default();
    cpu.flags = ControlFlags {
        debug: true,
        pause: false,
        monotonic: Some(8),
        ..Default::default()
    };
    (
        cpu,
        bus! {
            // Load the charset into ROM
            Charset [0x0050..0x00A0] = include_bytes!("../src/mem/charset.bin"),
            // Load the ROM file into RAM
            Program [0x0200..0x1000] = include_bytes!("../chip8-test-suite/bin/chip8-test-suite.ch8"),
            // Create a screen, and fill it with garbage data
            Screen  [0x0F00..0x1000] = include_bytes!("chip8_test_suite.rs"),
        },
    )
}

struct SuiteTest {
    test: u16,
    screen: &'static [u8],
}

fn run_screentest(test: SuiteTest, mut cpu: CPU, mut bus: Bus) {
    // Set the test to run
    bus.write(0x1feu16, test.test);
    // The test suite always initiates a keypause on test completion
    while !cpu.flags.keypause {
        cpu.multistep(&mut bus, 8).unwrap();
    }
    // Compare the screen to the reference screen buffer
    bus.print_screen().unwrap();
    bus! {crate::bus::Region::Screen [0..256] = test.screen}
        .print_screen()
        .unwrap();
    assert_eq!(bus.get_region(Screen).unwrap(), test.screen);
}

#[test]
fn splash_screen() {
    let (cpu, bus) = setup_environment();
    run_screentest(
        SuiteTest {
            test: 0,
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
            test: 0x01,
            screen: include_bytes!("screens/chip8-test-suite/IBM.bin"),
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
            test: 0x03,
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
            test: 0x0104,
            screen: include_bytes!("screens/chip8-test-suite/quirks.bin"),
        },
        cpu,
        bus,
    )
}
