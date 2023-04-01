//! Tests for chirp-minifb

use super::ui::*;
use chirp::*;
use std::{collections::hash_map::DefaultHasher, hash::Hash};

mod ui_builder {
    use super::*;
    #[test]
    fn ui_builder() -> Result<()> {
        let builder = UIBuilder::new(32, 64, "dummy.ch8").build()?;
        println!("{builder:?}");
        Ok(())
    }
    #[test]
    fn default() {
        let ui_builder = UIBuilder::default();
        println!("{ui_builder:?}");
    }
    #[test]
    #[allow(clippy::redundant_clone)]
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
        let mut ui = UIBuilder::new(32, 64, "dummy.ch8").build()?;
        let mut ch8 = new_chip8();
        ui.frame(&mut ch8).unwrap();
        Ok(())
    }
    #[test]
    fn keys() -> Result<()> {
        let mut ui = UIBuilder::new(32, 64, "dummy.ch8").build()?;
        let mut ch8 = new_chip8();
        let ch8 = &mut ch8;
        ui.frame(ch8).unwrap();
        ui.keys(ch8).unwrap();
        Ok(())
    }
    #[test]
    fn debug() -> Result<()> {
        println!("{:?}", UIBuilder::new(32, 64, "dummy.ch8").build()?);
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
