// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)

//! Contains implementations for each Chip-8 [Insn]

use super::{bus::Region, *};
use rand::random;

impl CPU {
    /// Executes a single [Insn]
    #[rustfmt::skip]
    #[inline(always)]
    pub(super) fn execute(&mut self, screen: &mut Bus, instruction: Insn) {
        match instruction {
            // Core Chip-8 instructions
            Insn::cls               => self.clear_screen(screen),
            Insn::ret               => self.ret(),
            Insn::jmp   {       A } => self.jump(A),
            Insn::call  {       A } => self.call(A),
            Insn::seb   {    x, B } => self.skip_equals_immediate(x, B),
            Insn::sneb  {    x, B } => self.skip_not_equals_immediate(x, B),
            Insn::se    { y, x    } => self.skip_equals(x, y),
            Insn::movb  {    x, B } => self.load_immediate(x, B),
            Insn::addb  {    x, B } => self.add_immediate(x, B),
            Insn::mov   { y, x    } => self.load(x, y),
            Insn::or    { y, x    } => self.or(x, y),
            Insn::and   { y, x    } => self.and(x, y),
            Insn::xor   { y, x    } => self.xor(x, y),
            Insn::add   { y, x    } => self.add(x, y),
            Insn::sub   { y, x    } => self.sub(x, y),
            Insn::shr   { y, x    } => self.shift_right(x, y),
            Insn::bsub  { y, x    } => self.backwards_sub(x, y),
            Insn::shl   { y, x    } => self.shift_left(x, y),
            Insn::sne   { y, x    } => self.skip_not_equals(x, y),
            Insn::movI  {       A } => self.load_i_immediate(A),
            Insn::jmpr  {       A } => self.jump_indexed(A),
            Insn::rand  {    x, B } => self.rand(x, B),
            Insn::draw  { y, x, n } => self.draw(x, y, n, screen),
            Insn::sek   {    x    } => self.skip_key_equals(x),
            Insn::snek  {    x    } => self.skip_key_not_equals(x),
            Insn::getdt {    x    } => self.load_delay_timer(x),
            Insn::waitk {    x    } => self.wait_for_key(x),
            Insn::setdt {    x    } => self.store_delay_timer(x),
            Insn::movst {    x    } => self.store_sound_timer(x),
            Insn::addI  {    x    } => self.add_i(x),
            Insn::font  {    x    } => self.load_sprite(x),
            Insn::bcd   {    x    } => self.bcd_convert(x),
            Insn::dmao  {    x    } => self.store_dma(x),
            Insn::dmai  {    x    } => self.load_dma(x),
            // Super-Chip extensions
            Insn::scd   {       n } => self.scroll_down(n, screen),
            Insn::scr               => self.scroll_right(screen),
            Insn::scl               => self.scroll_left(screen),
            Insn::halt              => self.flags.pause(),
            Insn::lores             => self.init_lores(screen),
            Insn::hires             => self.init_hires(screen),
            Insn::hfont {    x    } => self.load_big_sprite(x),
            Insn::flgo  {    x    } => self.store_flags(x),
            Insn::flgi  {    x    } => self.load_flags(x),
            // XO-Chip extensions
            _ => unimplemented!(),
        }
    }
}

/// |`0aaa`| Issues a "System call" (ML routine)
///
/// |opcode| effect                             |
/// |------|------------------------------------|
/// |`00e0`| Clear screen memory to all 0       |
/// |`00ee`| Return from subroutine             |
impl CPU {
    /// |`00e0`| Clears the screen memory to 0
    #[inline(always)]
    pub(super) fn clear_screen(&mut self, bus: &mut Bus) {
        bus.clear_region(Region::Screen);
    }
    /// |`00ee`| Returns from subroutine
    #[inline(always)]
    pub(super) fn ret(&mut self) {
        self.pc = self.stack.pop().unwrap_or(0x200);
    }
}

/// Super Chip screen-control routines
///
/// |opcode| effect                             |
/// |------|------------------------------------|
/// |`00cN`| Scroll the screen down N lines     |
/// |`00fb`| Scroll the screen right            |
/// |`00fc`| Scroll the screen left             |
/// |`00fe`| Initialize lores mode              |
/// |`00ff`| Initialize hires mode              |
impl CPU {
    /// # |`00cN`|
    /// Scroll the screen down N lines
    #[inline(always)]
    pub(super) fn scroll_down(&mut self, n: Nib, screen: &mut Bus) {
        match self.flags.draw_mode {
            true => {
                // Get a line from the bus
                for i in (0..16 * (64 - n as usize)).step_by(16).rev() {
                    let line: u128 = screen.read(i);
                    screen.write(i - (n as usize * 16), 0u128);
                    screen.write(i, line);
                }
            }
            false => {
                // Get a line from the bus
                for i in (0..8 * (32 - n as usize)).step_by(8).rev() {
                    let line: u64 = screen.read(i);
                    screen.write(i, 0u64);
                    screen.write(i + (n as usize * 8), line);
                }
            }
        }
    }

    /// # |`00fb`|
    /// Scroll the screen right
    #[inline(always)]
    pub(super) fn scroll_right(&mut self, screen: &mut impl ReadWrite<u128>) {
        // Get a line from the bus
        for i in (0..16 * 64_usize).step_by(16) {
            //let line: u128 = bus.read(self.screen + i) >> 4;
            screen.write(i, screen.read(i) >> 4);
        }
    }
    /// # |`00fc`|
    ///  Scroll the screen left
    #[inline(always)]
    pub(super) fn scroll_left(&mut self, screen: &mut impl ReadWrite<u128>) {
        // Get a line from the bus
        for i in (0..16 * 64_usize).step_by(16) {
            let line: u128 = u128::wrapping_shl(screen.read(i), 4);
            screen.write(i, line);
        }
    }
    /// # |`00fe`|
    /// Initialize lores mode
    pub(super) fn init_lores(&mut self, screen: &mut Bus) {
        self.flags.draw_mode = false;
        screen.set_region(Region::Screen, 0..256);
        self.clear_screen(screen);
    }
    /// # |`00ff`|
    /// Initialize hires mode
    pub(super) fn init_hires(&mut self, screen: &mut Bus) {
        self.flags.draw_mode = true;
        screen.set_region(Region::Screen, 0..1024);
        self.clear_screen(screen);
    }
}

/// |`1aaa`| Sets pc to an absolute address
impl CPU {
    /// |`1aaa`| Sets the program counter to an absolute address
    #[inline(always)]
    pub(super) fn jump(&mut self, a: Adr) {
        // jump to self == halt
        if a.wrapping_add(2) == self.pc {
            self.flags.pause = true;
        }
        self.pc = a;
    }
}

/// |`2aaa`| Pushes pc onto the stack, then jumps to a
impl CPU {
    /// |`2aaa`| Pushes pc onto the stack, then jumps to a
    #[inline(always)]
    pub(super) fn call(&mut self, a: Adr) {
        self.stack.push(self.pc);
        self.pc = a;
    }
}

/// |`3xbb`| Skips next instruction if register X == b
impl CPU {
    /// |`3xbb`| Skips the next instruction if register X == b
    #[inline(always)]
    pub(super) fn skip_equals_immediate(&mut self, x: Reg, b: u8) {
        if self.v[x] == b {
            self.pc = self.pc.wrapping_add(2);
        }
    }
}

/// |`4xbb`| Skips next instruction if register X != b
impl CPU {
    /// |`4xbb`| Skips the next instruction if register X != b
    #[inline(always)]
    pub(super) fn skip_not_equals_immediate(&mut self, x: Reg, b: u8) {
        if self.v[x] != b {
            self.pc = self.pc.wrapping_add(2);
        }
    }
}

/// |`5xyn`| Performs a register-register comparison
///
/// |opcode| effect                             |
/// |------|------------------------------------|
/// |`5XY0`| Skip next instruction if vX == vY  |
impl CPU {
    /// |`5xy0`| Skips the next instruction if register X != register Y
    #[inline(always)]
    pub(super) fn skip_equals(&mut self, x: Reg, y: Reg) {
        if self.v[x] == self.v[y] {
            self.pc = self.pc.wrapping_add(2);
        }
    }
}

/// |`6xbb`| Loads immediate byte b into register vX
impl CPU {
    /// |`6xbb`| Loads immediate byte b into register vX
    #[inline(always)]
    pub(super) fn load_immediate(&mut self, x: Reg, b: u8) {
        self.v[x] = b;
    }
}

/// |`7xbb`| Adds immediate byte b to register vX
impl CPU {
    /// |`7xbb`| Adds immediate byte b to register vX
    #[inline(always)]
    pub(super) fn add_immediate(&mut self, x: Reg, b: u8) {
        self.v[x] = self.v[x].wrapping_add(b);
    }
}

/// |`8xyn`| Performs ALU operation
///
/// |opcode| effect                             |
/// |------|------------------------------------|
/// |`8xy0`| Y = X                              |
/// |`8xy1`| X = X | Y                          |
/// |`8xy2`| X = X & Y                          |
/// |`8xy3`| X = X ^ Y                          |
/// |`8xy4`| X = X + Y; Set vF=carry            |
/// |`8xy5`| X = X - Y; Set vF=carry            |
/// |`8xy6`| X = X >> 1                         |
/// |`8xy7`| X = Y - X; Set vF=carry            |
/// |`8xyE`| X = X << 1                         |
impl CPU {
    /// |`8xy0`| Loads the value of y into x
    #[inline(always)]
    pub(super) fn load(&mut self, x: Reg, y: Reg) {
        self.v[x] = self.v[y];
    }
    /// |`8xy1`| Performs bitwise or of vX and vY, and stores the result in vX
    ///
    /// # Quirk
    /// The original chip-8 interpreter will clobber vF for any 8-series instruction
    #[inline(always)]
    pub(super) fn or(&mut self, x: Reg, y: Reg) {
        self.v[x] |= self.v[y];
        if !self.flags.quirks.bin_ops {
            self.v[0xf] = 0;
        }
    }
    /// |`8xy2`| Performs bitwise and of vX and vY, and stores the result in vX
    ///
    /// # Quirk
    /// The original chip-8 interpreter will clobber vF for any 8-series instruction
    #[inline(always)]
    pub(super) fn and(&mut self, x: Reg, y: Reg) {
        self.v[x] &= self.v[y];
        if !self.flags.quirks.bin_ops {
            self.v[0xf] = 0;
        }
    }
    /// |`8xy3`| Performs bitwise xor of vX and vY, and stores the result in vX
    ///
    /// # Quirk
    /// The original chip-8 interpreter will clobber vF for any 8-series instruction
    #[inline(always)]
    pub(super) fn xor(&mut self, x: Reg, y: Reg) {
        self.v[x] ^= self.v[y];
        if !self.flags.quirks.bin_ops {
            self.v[0xf] = 0;
        }
    }
    /// |`8xy4`| Performs addition of vX and vY, and stores the result in vX
    #[inline(always)]
    pub(super) fn add(&mut self, x: Reg, y: Reg) {
        let carry;
        (self.v[x], carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[0xf] = carry.into();
    }
    /// |`8xy5`| Performs subtraction of vX and vY, and stores the result in vX
    #[inline(always)]
    pub(super) fn sub(&mut self, x: Reg, y: Reg) {
        let carry;
        (self.v[x], carry) = self.v[x].overflowing_sub(self.v[y]);
        self.v[0xf] = (!carry).into();
    }
    /// |`8xy6`| Performs bitwise right shift of vX
    ///
    /// # Quirk
    /// On the original chip-8 interpreter, this shifts vY and stores the result in vX
    #[inline(always)]
    pub(super) fn shift_right(&mut self, x: Reg, y: Reg) {
        let src: Reg = if self.flags.quirks.shift { x } else { y };
        let shift_out = self.v[src] & 1;
        self.v[x] = self.v[src] >> 1;
        self.v[0xf] = shift_out;
    }
    /// |`8xy7`| Performs subtraction of vY and vX, and stores the result in vX
    #[inline(always)]
    pub(super) fn backwards_sub(&mut self, x: Reg, y: Reg) {
        let carry;
        (self.v[x], carry) = self.v[y].overflowing_sub(self.v[x]);
        self.v[0xf] = (!carry).into();
    }
    /// 8X_E: Performs bitwise left shift of vX
    ///
    /// # Quirk
    /// On the original chip-8 interpreter, this would perform the operation on vY
    /// and store the result in vX. This behavior was left out, for now.
    #[inline(always)]
    pub(super) fn shift_left(&mut self, x: Reg, y: Reg) {
        let src: Reg = if self.flags.quirks.shift { x } else { y };
        let shift_out: u8 = self.v[src] >> 7;
        self.v[x] = self.v[src] << 1;
        self.v[0xf] = shift_out;
    }
}

/// |`9xyn`| Performs a register-register comparison
///
/// |opcode| effect                             |
/// |------|------------------------------------|
/// |`9XY0`| Skip next instruction if vX != vY  |
impl CPU {
    /// |`9xy0`| Skip next instruction if X != y
    #[inline(always)]
    pub(super) fn skip_not_equals(&mut self, x: Reg, y: Reg) {
        if self.v[x] != self.v[y] {
            self.pc = self.pc.wrapping_add(2);
        }
    }
}

/// |`Aaaa`| Load address #a into register I
impl CPU {
    /// |`Aadr`| Load address #adr into register I
    #[inline(always)]
    pub(super) fn load_i_immediate(&mut self, a: Adr) {
        self.i = a;
    }
}

/// |`Baaa`| Jump to &adr + v0
impl CPU {
    /// |`Badr`| Jump to &adr + v0
    ///
    /// Quirk:
    /// On the Super-Chip, this does stupid shit
    #[inline(always)]
    pub(super) fn jump_indexed(&mut self, a: Adr) {
        let reg = if self.flags.quirks.stupid_jumps {
            a as usize >> 8
        } else {
            0
        };
        self.pc = a.wrapping_add(self.v[reg] as Adr);
    }
}

/// |`Cxbb`| Stores a random number & the provided byte into vX
impl CPU {
    /// |`Cxbb`| Stores a random number & the provided byte into vX
    #[inline(always)]
    pub(super) fn rand(&mut self, x: Reg, b: u8) {
        self.v[x] = random::<u8>() & b;
    }
}

/// |`Dxyn`| Draws n-byte sprite to the screen at coordinates (vX, vY)
impl CPU {
    /// |`Dxyn`| Draws n-byte sprite to the screen at coordinates (vX, vY)
    ///
    /// # Quirk
    /// On the original chip-8 interpreter, this will wait for a VBI
    #[inline(always)]
    pub(super) fn draw(&mut self, x: Reg, y: Reg, n: Nib, screen: &mut Bus) {
        if !self.flags.quirks.draw_wait {
            self.flags.draw_wait = true;
        }
        // self.draw_hires handles both hi-res mode and drawing 16x16 sprites
        if self.flags.draw_mode || n == 0 {
            self.draw_hires(x, y, n, screen);
        } else {
            self.draw_lores(x, y, n, screen);
        }
    }

    /// |`Dxyn`| Chip-8: Draws n-byte sprite to the screen at coordinates (vX, vY)
    #[inline(always)]
    pub(super) fn draw_lores(&mut self, x: Reg, y: Reg, n: Nib, scr: &mut Bus) {
        self.draw_sprite(self.v[x] as u16 % 64, self.v[y] as u16 % 32, n, 64, 32, scr);
    }

    #[inline(always)]
    pub(super) fn draw_sprite(&mut self, x: u16, y: u16, n: Nib, w: u16, h: u16, screen: &mut Bus) {
        let w_bytes = w / 8;
        self.v[0xf] = 0;
        if let Some(sprite) = self
            .screen
            .get(self.i as usize..(self.i + n as u16) as usize)
        {
            for (line, &sprite) in sprite.iter().enumerate() {
                let line = line as u16;
                let sprite = ((sprite as u16) << (8 - (x % 8))).to_be_bytes();
                for (addr, &byte) in sprite.iter().enumerate().filter_map(|(idx, byte)| {
                    let x = (x / 8) + idx as u16;
                    Some((
                        if self.flags.quirks.screen_wrap {
                            ((y + line) % h * w_bytes + (x % w_bytes)) % (w_bytes * h)
                        } else if x < w_bytes {
                            (y + line) * w_bytes + x
                        } else {
                            return None;
                        },
                        byte,
                    ))
                }) {
                    let display: u8 = screen.read(addr);
                    screen.write(addr, byte ^ display);
                    if byte & display != 0 {
                        self.v[0xf] = 1;
                    }
                }
            }
        }
    }
}
/// |`Dxyn`| Super-Chip extension high-resolution graphics mode
impl CPU {
    /// |`Dxyn`| Super-Chip extension high-resolution graphics mode
    #[inline(always)]
    pub(super) fn draw_hires(&mut self, x: Reg, y: Reg, n: Nib, screen: &mut Bus) {
        if !self.flags.quirks.draw_wait {
            self.flags.draw_wait = true;
        }
        let (w, h) = match self.flags.draw_mode {
            true => (128, 64),
            false => (64, 32),
        };
        let (x, y) = (self.v[x] as u16 % w, self.v[y] as u16 % h);
        match n {
            0 => self.draw_schip_sprite(x, y, w, screen),
            _ => self.draw_sprite(x, y, n, w, h, screen),
        }
    }
    /// Draws a 16x16 Super Chip sprite
    #[inline(always)]
    pub(super) fn draw_schip_sprite(&mut self, x: u16, y: u16, w: u16, screen: &mut Bus) {
        self.v[0xf] = 0;
        let w_bytes = w / 8;
        if let Some(sprite) = self.screen.get(self.i as usize..(self.i + 32) as usize) {
            let sprite = sprite.to_owned();
            for (line, sprite) in sprite.chunks_exact(2).enumerate() {
                let sprite = u16::from_be_bytes(
                    sprite
                        .try_into()
                        .expect("Chunks should only return 2 bytes"),
                );
                let addr = (y + line as u16) * w_bytes + x / 8;
                let sprite = (sprite as u32) << (16 - (x % 8));
                let display: u32 = screen.read(addr);
                screen.write(addr, display ^ sprite);
                if display & sprite != 0 {
                    self.v[0xf] += 1;
                }
            }
        }
    }
}

/// |`Exbb`| Skips instruction on value of keypress
///
/// |opcode| effect                             |
/// |------|------------------------------------|
/// |`eX9e`| Skip next instruction if key == vX |
/// |`eXa1`| Skip next instruction if key != vX |
impl CPU {
    /// |`Ex9E`| Skip next instruction if key == vX
    #[inline(always)]
    pub(super) fn skip_key_equals(&mut self, x: Reg) {
        if self.keys[self.v[x] as usize & 0xf] {
            self.pc += 2;
        }
    }
    /// |`ExaE`| Skip next instruction if key != vX
    #[inline(always)]
    pub(super) fn skip_key_not_equals(&mut self, x: Reg) {
        if !self.keys[self.v[x] as usize & 0xf] {
            self.pc += 2;
        }
    }
}

/// |`Fxbb`| Performs IO
///
/// |opcode| effect                             |
/// |------|------------------------------------|
/// |`fX07`| Set vX to value in delay timer     |
/// |`fX0a`| Wait for input, store key in vX    |
/// |`fX15`| Set sound timer to the value in vX |
/// |`fX18`| set delay timer to the value in vX |
/// |`fX1e`| Add vX to I                        |
/// |`fX29`| Load sprite for character x into I |
/// |`fX33`| BCD convert X into I[0..3]         |
/// |`fX55`| DMA Stor from I to registers 0..=X |
/// |`fX65`| DMA Load from I to registers 0..=X |
impl CPU {
    /// |`Fx07`| Get the current DT, and put it in vX
    /// ```py
    /// vX = DT
    /// ```
    #[inline(always)]
    pub(super) fn load_delay_timer(&mut self, x: Reg) {
        self.v[x] = self.delay as u8;
    }
    /// |`Fx0A`| Wait for key, then vX = K
    #[inline(always)]
    pub(super) fn wait_for_key(&mut self, x: Reg) {
        if let Some(key) = self.lastkey {
            self.v[x] = key as u8;
            self.lastkey = None;
        } else {
            self.pc = self.pc.wrapping_sub(2);
            self.flags.keypause = true;
        }
    }
    /// |`Fx15`| Load vX into DT
    /// ```py
    /// DT = vX
    /// ```
    #[inline(always)]
    pub(super) fn store_delay_timer(&mut self, x: Reg) {
        self.delay = self.v[x] as f64;
    }
    /// |`Fx18`| Load vX into ST
    /// ```py
    /// ST = vX;
    /// ```
    #[inline(always)]
    pub(super) fn store_sound_timer(&mut self, x: Reg) {
        self.sound = self.v[x] as f64;
    }
    /// |`Fx1e`| Add vX to I,
    /// ```py
    /// I += vX;
    /// ```
    #[inline(always)]
    pub(super) fn add_i(&mut self, x: Reg) {
        self.i += self.v[x] as u16;
    }
    /// |`Fx29`| Load sprite for character x into I
    /// ```py
    /// I = sprite(X);
    /// ```
    #[inline(always)]
    pub(super) fn load_sprite(&mut self, x: Reg) {
        self.i = self.font + (5 * (self.v[x] as Adr % 0x10));
    }
    /// |`Fx33`| BCD convert X into I`[0..3]`
    #[inline(always)]
    pub(super) fn bcd_convert(&mut self, x: Reg) {
        let x = self.v[x];
        self.screen.write(self.i.wrapping_add(2), x % 10);
        self.screen.write(self.i.wrapping_add(1), x / 10 % 10);
        self.screen.write(self.i, x / 100 % 10);
    }
    /// |`Fx55`| DMA Stor from I to registers 0..=X
    ///
    /// # Quirk
    /// The original chip-8 interpreter uses I to directly index memory,
    /// with the side effect of leaving I as I+X+1 after the transfer is done.
    #[inline(always)]
    pub(super) fn store_dma(&mut self, x: Reg) {
        let i = self.i as usize;
        for (reg, value) in self
            .screen
            .get_mut(i..=i + x)
            .unwrap_or_default()
            .iter_mut()
            .enumerate()
        {
            *value = self.v[reg]
        }
        if !self.flags.quirks.dma_inc {
            self.i += x as Adr + 1;
        }
    }
    /// |`Fx65`| DMA Load from I to registers 0..=X
    ///
    /// # Quirk
    /// The original chip-8 interpreter uses I to directly index memory,
    /// with the side effect of leaving I as I+X+1 after the transfer is done.
    #[inline(always)]
    pub(super) fn load_dma(&mut self, x: Reg) {
        let i = self.i as usize;
        for (reg, value) in self
            .screen
            .get(i..=i + x)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            self.v[reg] = *value;
        }
        if !self.flags.quirks.dma_inc {
            self.i += x as Adr + 1;
        }
    }
}

/// |`Fxbb`| Super Chip: Performs IO
///
/// |opcode| effect                             |
/// |------|------------------------------------|
/// |`Fx30`| 16x16 equivalent of load_sprite    |
/// |`Fx75`| Save to "flag registers"           |
/// |`Fx85`| Load from "flag registers"         |
impl CPU {
    /// |`Fx30`| (Super-Chip) 16x16 equivalent of [CPU::load_sprite]
    ///
    /// TODO: Actually make and import the 16x font
    #[inline(always)]
    pub(super) fn load_big_sprite(&mut self, x: Reg) {
        self.i = self.font + (5 * 8) + (16 * (self.v[x] as Adr % 0x10));
    }

    /// |`Fx75`| (Super-Chip) Save to "flag registers"
    /// I just chuck it in 0x0..0xf. Screw it.
    #[inline(always)]
    pub(super) fn store_flags(&mut self, x: Reg) {
        // TODO: Save these, maybe
        for (reg, value) in self
            .screen
            .get_mut(0..=x)
            .unwrap_or_default()
            .iter_mut()
            .enumerate()
        {
            *value = self.v[reg]
        }
    }

    /// |`Fx85`| (Super-Chip) Load from "flag registers"
    /// I just chuck it in 0x0..0xf. Screw it.
    #[inline(always)]
    pub(super) fn load_flags(&mut self, x: Reg) {
        for (reg, value) in self
            .screen
            .get(0..=x)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            self.v[reg] = *value;
        }
    }
}
