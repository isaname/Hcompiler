// RISC-V 64-bit Register definitions and printing

// General-purpose Register Convention (RV64):
// Name         ABI Name    Meaning
// x0           zero        constant 0
// x1           ra          return address
// x2           sp          stack pointer
// x3           gp          global pointer
// x4           tp          thread pointer
// x5           t0          temporary
// x6-x7        t1-t2       temporary
// x8           s0/fp       saved/frame pointer
// x9           s1          saved
// x10-x11      a0-a1       arguments/return values
// x12-x17      a2-a7       arguments
// x18-x27      s2-s11      saved
// x28-x31      t3-t6       temporary

#[derive(Clone, Copy)]
pub struct Reg {
    pub id: u32,
}

impl Reg {
    pub fn new(id: u32) -> Self {
        assert!(id <= 31);
        Reg { id }
    }

    pub fn zero() -> Self { Reg::new(0) }
    pub fn ra() -> Self { Reg::new(1) }
    pub fn sp() -> Self { Reg::new(2) }
    pub fn gp() -> Self { Reg::new(3) }
    pub fn tp() -> Self { Reg::new(4) }
    pub fn fp() -> Self { Reg::new(8) }  // s0/fp

    pub fn a(i: u32) -> Self {
        assert!(i <= 7);
        if i < 2 {
            Reg::new(10 + i)  // a0-a1
        } else {
            Reg::new(10 + i)  // a2-a7
        }
    }

    pub fn t(i: u32) -> Self {
        assert!(i <= 6);
        match i {
            0 => Reg::new(5),      // t0
            1..=2 => Reg::new(5 + i),  // t1-t2
            3..=6 => Reg::new(25 + i), // t3-t6
            _ => panic!("Invalid temporary register index"),
        }
    }

    pub fn s(i: u32) -> Self {
        assert!(i <= 11);
        if i == 0 {
            Reg::new(8)  // s0/fp
        } else if i == 1 {
            Reg::new(9)  // s1
        } else {
            Reg::new(16 + i)  // s2-s11
        }
    }

    pub fn print(&self) -> String {
        match self.id {
            0 => "zero".to_string(),
            1 => "ra".to_string(),
            2 => "sp".to_string(),
            3 => "gp".to_string(),
            4 => "tp".to_string(),
            5 => "t0".to_string(),
            6 => "t1".to_string(),
            7 => "t2".to_string(),
            8 => "s0".to_string(),  // fp
            9 => "s1".to_string(),
            10..=17 => format!("a{}", self.id - 10),
            18..=27 => format!("s{}", self.id - 16),
            28..=31 => format!("t{}", self.id - 25),
            _ => panic!("Invalid register id"),
        }
    }
}

// Floating-point Register Convention (RV64F/D)
// Name         ABI Name    Meaning
// f0-f7        ft0-ft7     temporary
// f8-f9        fs0-fs1     saved
// f10-f11      fa0-fa1     arguments/return values
// f12-f17      fa2-fa7     arguments
// f18-f27      fs2-fs11    saved
// f28-f31      ft8-ft11    temporary

#[derive(Clone, Copy)]
pub struct FReg {
    pub id: u32,
}

impl FReg {
    pub fn new(id: u32) -> Self {
        assert!(id <= 31);
        FReg { id }
    }

    pub fn fa(i: u32) -> Self {
        assert!(i <= 7);
        if i < 2 {
            FReg::new(10 + i)  // fa0-fa1
        } else {
            FReg::new(10 + i)  // fa2-fa7
        }
    }

    pub fn ft(i: u32) -> Self {
        assert!(i <= 11);
        if i < 8 {
            FReg::new(i)  // ft0-ft7
        } else {
            FReg::new(20 + i)  // ft8-ft11
        }
    }

    pub fn fs(i: u32) -> Self {
        assert!(i <= 11);
        if i < 2 {
            FReg::new(8 + i)  // fs0-fs1
        } else {
            FReg::new(16 + i)  // fs2-fs11
        }
    }

    pub fn print(&self) -> String {
        match self.id {
            0..=7 => format!("ft{}", self.id),
            8..=9 => format!("fs{}", self.id - 8),
            10..=17 => format!("fa{}", self.id - 10),
            18..=27 => format!("fs{}", self.id - 16),
            28..=31 => format!("ft{}", self.id - 20),
            _ => panic!("Invalid FReg id"),
        }
    }
}

// There are no separate condition flag registers in RISC-V
// Comparisons directly produce results in general registers
