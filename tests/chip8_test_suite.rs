//! These are a series of interpreter tests using Timendus's incredible test suite

pub use chirp::*;

fn setup_environment() -> (CPU, Bus) {
    let mut cpu = CPU::default();
    cpu.flags = Flags {
        debug: true,
        pause: false,
        monotonic: Some(10),
        ..Default::default()
    };
    (
        cpu,
        bus! {
            // Load the charset into ROM
            Charset [0x0050..0x00A0] = include_bytes!("../src/mem/charset.bin"),
            // Load the ROM file into RAM
            Program [0x0200..0x1000],
            // Create a screen, and fill it with garbage
            Screen  [0x0F00..0x1000] = include_bytes!("chip8_test_suite.rs"),
        },
    )
}

struct SuiteTest {
    test: u8,
    data: &'static [u8],
    screen: &'static [u8],
}

fn run_screentest(test: SuiteTest, mut cpu: CPU, mut bus: Bus) {
    // Set the test to run
    bus.write(0x1ffu16, test.test);
    bus.load_region(Program, test.data).unwrap();
    // The test suite always initiates a keypause on test completion
    while !(cpu.flags.keypause || cpu.flags.pause) {
        cpu.multistep(&mut bus, 100).unwrap();
    }
    // Compare the screen to the reference screen buffer
    bus.print_screen().unwrap();
    bus! {crate::cpu::bus::Region::Screen [0..256] = test.screen}
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
