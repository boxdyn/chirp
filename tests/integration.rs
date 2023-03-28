//! Testing methods on Chirp's public API
use chirp::prelude::*;
use std::{collections::hash_map::DefaultHasher, hash::Hash};

#[test]
fn chip8() {
    let ch8 = Chip8::default(); // Default
    let ch82 = ch8.clone(); // Clone
    assert_eq!(ch8, ch82); // PartialEq
    println!("{ch8:?}"); // Debug
}

mod bus {
    use super::*;
    mod region {
        use super::*;
        //  #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[test]
        fn copy() {
            let r1 = Screen;
            let r2 = r1;
            assert_eq!(r1, r2);
        }
        #[test]
        fn clone() {
            let r1 = Screen;
            let r2 = r1.clone();
            assert_eq!(r1, r2);
        }
        #[test]
        fn display() {
            println!("{Charset}{Program}{Screen}{Stack}{Count}");
        }
        #[test]
        fn debug() {
            println!("{Charset:?}{Program:?}{Screen:?}{Stack:?}{Count:?}");
        }
        // lmao the things you do for test coverage
        #[test]
        fn eq() {
            assert_eq!(Screen, Screen);
            assert_ne!(Charset, Program);
        }
        #[test]
        fn ord() {
            assert_eq!(Stack, Charset.max(Program).max(Screen).max(Stack));
            assert!(Charset < Program && Program < Screen && Screen < Stack);
        }
        #[test]
        fn hash() {
            let mut hasher = DefaultHasher::new();
            Stack.hash(&mut hasher);
            println!("{hasher:?}");
        }
    }
    #[test]
    #[should_panic]
    fn bus_missing_region() {
        // Print the screen of a bus with no screen
        bus! {}.print_screen().unwrap()
    }
}

mod cpu {
    use super::*;

    #[test]
    fn set_break() {
        let mut cpu = CPU::default();
        let point = 0x234;
        assert_eq!(cpu.breakpoints(), &[]);
        // Attempt to set the same breakpoint 100 times
        for _ in 0..100 {
            cpu.set_break(point);
        }
        assert_eq!(cpu.breakpoints(), &[point]);
    }
    #[test]
    fn unset_break() {
        let mut cpu = CPU::default();
        let point = 0x234;
        // set TWO breakpoints
        cpu.set_break(point + 1);
        cpu.set_break(point);
        assert_eq!(cpu.breakpoints(), &[point + 1, point]);
        // Attempt to unset the same breakpoint 100 times
        for _ in 0..100 {
            cpu.unset_break(point);
        }
        // Only unset the matching point
        assert_eq!(cpu.breakpoints(), &[point + 1]);
    }

    #[test]
    fn press_invalid_key() {
        let mut cpu = CPU::default();
        let cpu2 = cpu.clone();
        cpu.press(0x21345134);
        // no change has been made
        assert_eq!(cpu, cpu2);
    }
    #[test]
    fn release_invalid_key() {
        let mut cpu = CPU::default();
        let cpu2 = cpu.clone();
        cpu.release(0x21345134);
        // no change has been made
        assert_eq!(cpu, cpu2);
    }
    #[test]
    fn set_invalid_gpr() {
        let mut cpu = CPU::default();
        let cpu2 = cpu.clone();
        cpu.set_v(0x21345134, 0xff);
        // no change has been made
        assert_eq!(cpu, cpu2);
    }
    mod controlflags {
        use super::*;
        //#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[test]
        fn clone() {
            let cf1 = ControlFlags {
                debug: false,
                pause: false,
                keypause: false,
                vbi_wait: false,
                lastkey: None,
                quirks: Default::default(),
                monotonic: None,
            };
            let cf2 = cf1.clone();
            assert_eq!(cf1, cf2)
        }
        #[test]
        fn debug() {
            println!("{:?}", ControlFlags::default());
        }
        #[test]
        fn default() {
            assert_eq!(
                ControlFlags::default(),
                ControlFlags {
                    debug: false,
                    pause: false,
                    keypause: false,
                    vbi_wait: false,
                    lastkey: Default::default(),
                    quirks: Default::default(),
                    monotonic: Default::default()
                }
            )
        }
        #[test]
        fn eq() {
            let cf1 = ControlFlags::default();
            let cf2 = ControlFlags {
                debug: true,
                pause: true,
                keypause: true,
                vbi_wait: true,
                lastkey: Default::default(),
                quirks: Default::default(),
                monotonic: Default::default(),
            };
            assert_ne!(cf1, cf2);
        }
        #[test]
        fn ord() {
            let cf1 = ControlFlags::default();
            let cf2 = ControlFlags {
                debug: true,
                pause: true,
                keypause: true,
                vbi_wait: true,
                lastkey: Default::default(),
                quirks: Default::default(),
                monotonic: Default::default(),
            };
            assert!(cf1 < cf2);
            assert_eq!(ControlFlags::default(), cf1.min(cf2));
        }
        #[test]
        fn hash() {
            let mut hasher = DefaultHasher::new();
            ControlFlags::default().hash(&mut hasher);
            println!("{:?}", hasher);
        }
    }
}

#[test]
fn error() {
    let error = chirp::error::Error::FunkyMath {
        word: 0x1234,
        explanation: "This shit was funky".to_owned(),
    };
    println!("{error} {error:?}");
}

mod ui_builder {
    use super::*;
    #[test]
    fn ui_builder() -> Result<()> {
        let builder = UIBuilder::new(32, 64).rom("dummy.ch8").build()?;
        println!("{builder:?}");
        Ok(())
    }
    #[test]
    fn default() {
        let ui_builder = UIBuilder::default();
        println!("{ui_builder:?}");
    }
    #[test]
    fn clone_debug() {
        let ui_builder_clone = UIBuilder::default().clone();
        println!("{ui_builder_clone:?}");
    }
}
mod ui {
    use super::*;
    fn new_chip8() -> Chip8 {
        Chip8 {
            cpu: CPU::default(),
            bus: bus! {},
        }
    }
    #[test]
    fn frame() -> Result<()> {
        let mut ui = UIBuilder::new(32, 64).build()?;
        let mut ch8 = new_chip8();
        ui.frame(&mut ch8).unwrap();
        Ok(())
    }
    #[test]
    fn keys() -> Result<()> {
        let mut ui = UIBuilder::new(32, 64).build()?;
        let mut ch8 = new_chip8();
        let ch8 = &mut ch8;
        ui.frame(ch8).unwrap();
        ui.keys(ch8);
        Ok(())
    }
    #[test]
    fn debug() -> Result<()> {
        println!("{:?}", UIBuilder::new(32, 64).build()?);
        Ok(())
    }
}

mod framebuffer_format {
    use super::*;
    #[test]
    fn default() {
        let _fbf = FrameBufferFormat::default();
    }
    #[test]
    fn clone() {
        let fbf = FrameBufferFormat {
            fg: 0x12345678,
            bg: 0x90abcdef,
        };
        let fbf2 = fbf.clone();
        assert_eq!(fbf, fbf2);
    }
    #[test]
    fn debug() {
        println!("{:?}", FrameBufferFormat::default());
    }
    #[test]
    fn eq() {
        assert_eq!(FrameBufferFormat::default(), FrameBufferFormat::default());
        assert_ne!(
            FrameBufferFormat {
                fg: 0xff00ff,
                bg: 0x00ff00
            },
            FrameBufferFormat {
                fg: 0x00ff00,
                bg: 0xff00ff
            },
        );
    }
    #[test]
    fn ord() {
        assert!(
            FrameBufferFormat::default()
                == FrameBufferFormat {
                    fg: 0xffffff,
                    bg: 0xffffff,
                }
                .min(FrameBufferFormat::default())
        );
    }
    #[test]
    fn hash() {
        let mut hasher = DefaultHasher::new();
        FrameBufferFormat::default().hash(&mut hasher);
        println!("{hasher:?}");
    }
}

mod framebuffer {
    use super::*;
    // [derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[test]
    fn new() {
        assert_eq!(FrameBuffer::new(64, 32), FrameBuffer::default());
    }
    #[test]
    fn clone() {
        let fb1 = FrameBuffer::default();
        let fb2 = fb1.clone();
        assert_eq!(fb1, fb2);
    }

    #[test]
    fn debug() {
        println!("{:?}", FrameBuffer::default());
    }

    #[test]
    fn eq() {
        assert_eq!(FrameBuffer::new(64, 32), FrameBuffer::default());
    }

    #[test]
    fn ord() {
        assert!(FrameBuffer::new(21, 12) == FrameBuffer::new(21, 12).min(FrameBuffer::new(34, 46)));
    }

    #[test]
    fn hash() {
        let mut hasher = DefaultHasher::new();
        FrameBuffer::default().hash(&mut hasher);
        println!("{hasher:?}");
    }
}

mod quirks {
    use super::*;
    use chirp::cpu::Quirks;

    #[test]
    fn from_true() {
        let quirks_true = Quirks::from(true);
        assert_eq!(
            quirks_true,
            Quirks {
                bin_ops: true,
                shift: true,
                draw_wait: true,
                dma_inc: true,
                stupid_jumps: false,
            }
        )
    }

    #[test]
    fn from_false() {
        let quirks_true = Quirks::from(false);
        assert_eq!(
            quirks_true,
            Quirks {
                bin_ops: false,
                shift: false,
                draw_wait: false,
                dma_inc: false,
                stupid_jumps: false,
            }
        )
    }

    #[test]
    fn clone() {
        let q1 = Quirks {
            bin_ops: false,
            shift: true,
            draw_wait: false,
            dma_inc: true,
            stupid_jumps: false,
        };
        let q2 = q1.clone();
        assert_eq!(q1, q2);
    }

    #[test]
    fn debug() {
        println!("{:?}", Quirks::from(true));
    }

    #[test]
    fn eq() {
        assert_ne!(Quirks::from(false), Quirks::from(true));
    }

    #[test]
    fn ord() {
        assert!(Quirks::from(false) < Quirks::from(true));
        assert!(Quirks::from(true) == Quirks::from(false).max(Quirks::from(true)));
    }

    #[test]
    fn hash() {
        let mut hasher = DefaultHasher::new();
        Quirks::from(true).hash(&mut hasher);
        println!("{hasher:?}");
    }
}
