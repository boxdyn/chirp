// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)

//! Exercises the instruction decode logic.
use super::*;

const INDX: &[u8; 16] = b"\0\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f";

/// runs one arbitrary operation on a brand new CPU
/// returns the CPU for inspection
fn run_single_op(op: &[u8]) -> CPU {
    let (mut cpu, mut bus) = (
        CPU::default(),
        bus! {
            Screen[0x0..0x1000],
        },
    );
    cpu.mem
        .load_region(Program, op).unwrap();
    cpu.v = *INDX;
    cpu.flags.quirks = Quirks::from(false);
    cpu.tick(&mut bus).unwrap(); // will panic if unimplemented
    cpu
}

#[rustfmt::skip] 
mod sys {
    use super::*;
    #[test]                 fn cls()   { run_single_op(b"\x00\xe0"); } 
    #[test]                 fn ret()   { run_single_op(b"\x00\xee"); } 
    #[test] #[should_panic] fn u0420() { run_single_op(b"\x04\x20"); }
}
#[rustfmt::skip] 
mod jump {
    use super::*;
    #[test] fn aligned()   { assert_eq!(0x230, run_single_op(b"\x12\x30").pc); } 
    #[test] fn unaligned() { assert_eq!(0x231, run_single_op(b"\x12\x31").pc); }
}
#[rustfmt::skip] 
mod call {
    use super::*;
    #[test] fn aligned()   { assert_eq!(0x230, run_single_op(b"\x22\x30").pc); } 
    #[test] fn unaligned() { assert_eq!(0x231, run_single_op(b"\x22\x31").pc); }
}
#[rustfmt::skip] 
mod skeb {
    use super::*;
    #[test] fn skip()    { assert_eq!(0x204, run_single_op(b"\x30\x00").pc); } 
    #[test] fn no_skip() { assert_eq!(0x202, run_single_op(b"\x30\x01").pc); }
}
#[rustfmt::skip] 
mod sneb {
    use super::*;
    #[test] fn skip()   { assert_eq!(0x204, run_single_op(b"\x40\x01").pc); } 
    #[test] fn noskip() { assert_eq!(0x202, run_single_op(b"\x40\x00").pc); }
}
#[rustfmt::skip] 
mod se {
    use super::*;
    #[test] fn skip()   { assert_eq!(0x204, run_single_op(b"\x50\x00").pc); }
    #[test] fn noskip() { assert_eq!(0x202, run_single_op(b"\x50\x10").pc); }
    #[test] #[should_panic] fn u5ff1() { run_single_op(b"\x5f\xf1"); }
    #[test] #[should_panic] fn u5ff2() { run_single_op(b"\x5f\xf2"); }
    #[test] #[should_panic] fn u5ff3() { run_single_op(b"\x5f\xf3"); }
    #[test] #[should_panic] fn u5ff4() { run_single_op(b"\x5f\xf4"); }
    #[test] #[should_panic] fn u5ff5() { run_single_op(b"\x5f\xf5"); }
    #[test] #[should_panic] fn u5ff6() { run_single_op(b"\x5f\xf6"); }
    #[test] #[should_panic] fn u5ff7() { run_single_op(b"\x5f\xf7"); }
    #[test] #[should_panic] fn u5ff8() { run_single_op(b"\x5f\xf8"); }
    #[test] #[should_panic] fn u5ff9() { run_single_op(b"\x5f\xf9"); }
    #[test] #[should_panic] fn u5ffa() { run_single_op(b"\x5f\xfa"); }
    #[test] #[should_panic] fn u5ffb() { run_single_op(b"\x5f\xfb"); }
    #[test] #[should_panic] fn u5ffc() { run_single_op(b"\x5f\xfc"); }
    #[test] #[should_panic] fn u5ffd() { run_single_op(b"\x5f\xfd"); }
    #[test] #[should_panic] fn u5ffe() { run_single_op(b"\x5f\xfe"); }
    #[test] #[should_panic] fn u5fff() { run_single_op(b"\x5f\xff"); }
}
#[rustfmt::skip] 
mod mov {
    use super::*;
    #[test] fn w00() { assert_eq!(0x00, run_single_op(b"\x61\x00").v[1]); }
    #[test] fn wc5() { assert_eq!(0xc5, run_single_op(b"\x62\xc5").v[2]); }
    #[test] fn wff() { assert_eq!(0xff, run_single_op(b"\x63\xff").v[3]); }
}
#[rustfmt::skip] 
mod add {
    use super::*;
    #[test] fn p00() { assert_eq!(0x01, run_single_op(b"\x71\x00").v[1]); }
    #[test] fn pc5() { assert_eq!(0xc7, run_single_op(b"\x72\xc5").v[2]); }
    #[test] fn pff() { assert_eq!(0x02, run_single_op(b"\x73\xff").v[3]); }
}
#[rustfmt::skip] 
mod alu {
    use super::*; 
    #[test] fn mov()  { assert_eq!(0x02, run_single_op(b"\x81\x20").v[1]); } 
    #[test] fn or()   { assert_eq!(0x03, run_single_op(b"\x81\x21").v[1]); } 
    #[test] fn and()  { assert_eq!(0x00, run_single_op(b"\x81\x22").v[1]); } 
    #[test] fn xor()  { assert_eq!(0x03, run_single_op(b"\x81\x23").v[1]); } 
    #[test] fn add()  { assert_eq!(0x03, run_single_op(b"\x81\x24").v[1]); } 
    #[test] fn sub()  { assert_eq!(0xff, run_single_op(b"\x81\x25").v[1]); } 
    #[test] fn shr()  { assert_eq!(0x01, run_single_op(b"\x81\x26").v[1]); } 
    #[test] fn bsub() { assert_eq!(0x01, run_single_op(b"\x81\x27").v[1]); }
    #[test] #[should_panic] fn u8128() { run_single_op(b"\x81\x28");       }
    #[test] #[should_panic] fn u8129() { run_single_op(b"\x81\x29");       }
    #[test] #[should_panic] fn u812a() { run_single_op(b"\x81\x2a");       }
    #[test] #[should_panic] fn u812b() { run_single_op(b"\x81\x2b");       }
    #[test] #[should_panic] fn u812c() { run_single_op(b"\x81\x2c");       }
    #[test] #[should_panic] fn u812d() { run_single_op(b"\x81\x2d");       }
    #[test] fn shl()  { assert_eq!(0x04, run_single_op(b"\x81\x2e").v[1]); }
    #[test] #[should_panic] fn u812f() { run_single_op(b"\x81\x2f");       }
}
#[rustfmt::skip] 
mod sne {
    use super::*;
    #[test] fn skip()    { assert_eq!(0x204, run_single_op(b"\x90\x10").pc); }
    #[test] fn no_skip() { assert_eq!(0x202, run_single_op(b"\x90\x00").pc); }
    #[test] #[should_panic] fn u9ff1() { run_single_op(b"\x9f\xf1");         }
    #[test] #[should_panic] fn u9ff2() { run_single_op(b"\x9f\xf2");         }
    #[test] #[should_panic] fn u9ff3() { run_single_op(b"\x9f\xf3");         }
    #[test] #[should_panic] fn u9ff4() { run_single_op(b"\x9f\xf4");         }
    #[test] #[should_panic] fn u9ff5() { run_single_op(b"\x9f\xf5");         }
    #[test] #[should_panic] fn u9ff6() { run_single_op(b"\x9f\xf6");         }
    #[test] #[should_panic] fn u9ff7() { run_single_op(b"\x9f\xf7");         }
    #[test] #[should_panic] fn u9ff8() { run_single_op(b"\x9f\xf8");         }
    #[test] #[should_panic] fn u9ff9() { run_single_op(b"\x9f\xf9");         }
    #[test] #[should_panic] fn u9ffa() { run_single_op(b"\x9f\xfa");         }
    #[test] #[should_panic] fn u9ffb() { run_single_op(b"\x9f\xfb");         }
    #[test] #[should_panic] fn u9ffc() { run_single_op(b"\x9f\xfc");         }
    #[test] #[should_panic] fn u9ffd() { run_single_op(b"\x9f\xfd");         }
    #[test] #[should_panic] fn u9ffe() { run_single_op(b"\x9f\xfe");         }
    #[test] #[should_panic] fn u9fff() { run_single_op(b"\x9f\xff");         }
}
#[rustfmt::skip] 
mod movi {
    use super::*;
    #[test] fn aligned()   { assert_eq!(0x230, run_single_op(b"\xa2\x30").i()); } 
    #[test] fn unaligned() { assert_eq!(0x231, run_single_op(b"\xa2\x31").i()); }
}
#[rustfmt::skip] 
mod jmpr {
    use super::*;
    #[test] fn aligned()   { assert_eq!(0x230, run_single_op(b"\xb2\x30").pc); } 
    #[test] fn unaligned() { assert_eq!(0x231, run_single_op(b"\xb2\x31").pc); }
}
#[rustfmt::skip] 
mod rand {
    use super::*;
    // for exhaustive testing, see src/cpu/tests.rs
    #[test] fn rand() { assert!(run_single_op(b"\xc0\x01").v[0] <= 1); }
}
#[rustfmt::skip] 
mod draw {
    use super::*;
    #[test] fn draw() { run_single_op(b"\xd0\x0f"); }
}
#[rustfmt::skip] 
mod key {
    use super::*;
    #[test] fn skip_key_equals()     { assert_eq!(0x202, run_single_op(b"\xe0\x9e").pc); }
    #[test] fn skip_key_not_equals() { assert_eq!(0x204, run_single_op(b"\xe0\xa1").pc); }
    #[test] #[should_panic] fn uefff() { run_single_op(b"\xef\xff"); }

}
#[rustfmt::skip] 
mod io {
    use super::*;
    #[test] fn load_delay_timer()  { assert_eq!(0x0, run_single_op(b"\xf7\x07").v[7]);    }
    #[test] fn wait_for_key()      { assert!(run_single_op(b"\xf0\x0a").flags.keypause);  }
    #[test] fn store_delay_timer() { assert_eq!(0xf, run_single_op(b"\xff\x15").delay()); }
    #[test] fn store_sound_timer() { assert_eq!(0xf, run_single_op(b"\xff\x18").sound()); }
    #[test] fn add_i()             { assert_eq!(0x0, run_single_op(b"\xf0\x1e").i);       }
    #[test] fn load_sprite()       { assert_eq!(0x50, run_single_op(b"\xf0\x29").i);      }
    #[test] fn bcd_convert()       { run_single_op(b"\xf0\x33"); /* nothing to check */   }
    #[test] fn store_dma()         { assert_eq!(INDX, run_single_op(b"\xff\x55").v());    }
    #[test] fn load_dma()          { assert_eq!([0;16], run_single_op(b"\xff\x65").v());  }
    // unimplemented
    #[test] #[should_panic] fn uffff() { run_single_op(b"\xff\xff"); }
}
