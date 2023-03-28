pub use chirp::prelude::*;

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
            Program [0x0200..0x1000] = include_bytes!("../chip-8/BC_test.ch8"),
            // Create a screen
            Screen  [0x0F00..0x1000] = include_bytes!("../chip-8/IBM Logo.ch8"),
        },
    )
}

/// These are a series of interpreter tests using Timendus's incredible test suite
/// TODO: These are technically integration tests, and should be moved to src/tests
mod chip8_test_suite {
    use super::*;

    struct SuiteTest {
        program: &'static [u8],
        screen: &'static [u8],
    }

    fn run_screentest(test: SuiteTest, mut cpu: CPU, mut bus: Bus) {
        // Load the test program
        bus = bus.load_region(Program, test.program);
        // The test suite always initiates a keypause on test completion
        while !cpu.flags.keypause {
            cpu.multistep(&mut bus, 8);
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
        let (mut c, b) = setup_environment();
        c.flags.quirks = true.into();
        run_screentest(
            SuiteTest {
                program: include_bytes!("chip8-test-suite/bin/chip8-test-suite.ch8"),
                screen: include_bytes!("screens/chip8-test-suite.ch8/splash.bin"),
            },
            c,
            b,
        )
    }

    #[test]
    fn flags_test() {
        let (mut c, mut b) = setup_environment();
        c.flags.quirks = true.into();
        b.write(0x1ffu16, 3u8);
        run_screentest(
            SuiteTest {
                program: include_bytes!("chip8-test-suite/bin/chip8-test-suite.ch8"),
                screen: include_bytes!("screens/chip8-test-suite.ch8/flags.bin"),
            },
            c,
            b,
        )
    }

    #[test]
    fn quirks_test() {
        let (mut c, mut b) = setup_environment();
        c.flags.quirks = true.into();
        b.write(0x1feu16, 0x0104u16);
        run_screentest(
            SuiteTest {
                program: include_bytes!("chip8-test-suite/bin/chip8-test-suite.ch8"),
                screen: include_bytes!("screens/chip8-test-suite.ch8/quirks.bin"),
            },
            c,
            b,
        )
    }
}
