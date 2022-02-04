
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FaceMask3(u32);

impl FaceMask3 {
    pub const NONE: FaceMask3 = FaceMask3(0);
    pub const FULL: FaceMask3 = FaceMask3::new(true, true, true, true, true, true);
    pub const fn from_raw(u: u32) -> Self {
        FaceMask3(u)
    }
    pub const fn to_raw(self) -> u32 {
        self.0
    }
    pub const fn new(xn: bool, xp: bool, yn: bool, yp: bool, zn: bool, zp: bool) -> Self {
        FaceMask3(
            (if xn { 1 } else { 0 })
                | (if xp { 2 } else { 0 })
                | (if yn { 4 } else { 0 })
                | (if yp { 8 } else { 0 })
                | (if zn { 16 } else { 0 })
                | (if zp { 32 } else { 0 }),
        )
    }

    pub fn xn(self) -> bool {
        self.0 & 1 != 0
    }
    pub fn xp(self) -> bool {
        self.0 & 2 != 0
    }
    pub fn x(self) -> bool {
        self.0 & (1 | 2) != 0
    }
    pub fn yn(self) -> bool {
        self.0 & 4 != 0
    }
    pub fn yp(self) -> bool {
        self.0 & 8 != 0
    }
    pub fn y(self) -> bool {
        self.0 & (4 | 8) != 0
    }
    pub fn zn(self) -> bool {
        self.0 & 16 != 0
    }
    pub fn zp(self) -> bool {
        self.0 & 32 != 0
    }
    pub fn z(self) -> bool {
        self.0 & (32 | 16) != 0
    }
    pub fn set_xn(&mut self, xn: bool) {
        self.0 = (self.0 & !1) | (if xn { 1 } else { 0 });
    }

    pub fn set_xp(&mut self, xp: bool) {
        self.0 = (self.0 & !2) | (if xp { 2 } else { 0 });
    }

    pub fn set_yn(&mut self, yn: bool) {
        self.0 = (self.0 & !4) | (if yn { 4 } else { 0 });
    }

    pub fn set_yp(&mut self, yp: bool) {
        self.0 = (self.0 & !8) | (if yp { 8 } else { 0 });
    }

    pub fn set_zn(&mut self, zn: bool) {
        self.0 = (self.0 & !16) | (if zn { 16 } else { 0 });
    }
    pub fn set_zp(&mut self, zp: bool) {
        self.0 = (self.0 & !32) | (if zp { 32 } else { 0 });
    }

    pub fn and(self, o: FaceMask3) -> FaceMask3 {
        FaceMask3::from_raw(self.0 & o.0)
    }

    pub fn count(self) -> u32 {
        let mut c = 0;
        let mut n = self.0;
        while n != 0 {
            if n & 1 != 0 {
                c += 1;
            }
            n = n >> 1;
        }
        c
    }
}
