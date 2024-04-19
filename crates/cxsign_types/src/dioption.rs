pub enum Dioption<T1, T2> {
    None,
    First(T1),
    Second(T2),
    Both(T1, T2),
}
impl<T1, T2> Dioption<T1, T2> {
    pub fn into_first(self) -> Option<T1> {
        match self {
            Dioption::First(f) => Some(f),
            Dioption::Both(f, _) => Some(f),
            _ => None,
        }
    }
    pub fn into_second(self) -> Option<T2> {
        match self {
            Dioption::Second(s) => Some(s),
            Dioption::Both(_, s) => Some(s),
            _ => None,
        }
    }
    pub fn into_both(self) -> Option<(T1, T2)> {
        match self {
            Dioption::Both(f, s) => Some((f, s)),
            _ => None,
        }
    }
    pub fn first(&self) -> Option<&T1> {
        match self {
            Dioption::First(f) => Some(f),
            Dioption::Both(f, _) => Some(f),
            _ => None,
        }
    }
    pub fn second(&self) -> Option<&T2> {
        match self {
            Dioption::Second(s) => Some(s),
            Dioption::Both(_, s) => Some(s),
            _ => None,
        }
    }
    pub fn both(&self) -> Option<(&T1, &T2)> {
        match self {
            Dioption::Both(f, s) => Some((f, s)),
            _ => None,
        }
    }
    pub fn is_both(&self) -> bool {
        match self {
            Dioption::Both(_, _) => true,
            _ => false,
        }
    }
    pub fn map<B1, B2, FF, SF>(self, mut ff: FF, mut sf: SF) -> Dioption<B1, B2>
    where
        Self: Sized,
        FF: FnMut(T1) -> B1,
        SF: FnMut(T2) -> B2,
    {
        match self {
            Dioption::First(f) => Dioption::First(ff(f)),
            Dioption::Second(s) => Dioption::Second(sf(s)),
            Dioption::Both(f, s) => Dioption::Both(ff(f), sf(s)),
            Dioption::None => Dioption::None,
        }
    }
    pub fn map_first<B, FF>(self, ff: FF) -> Dioption<B, T2>
    where
        Self: Sized,
        FF: FnMut(T1) -> B,
    {
        self.map(ff, |s| s)
    }
    pub fn map_second<B, F>(self, f: F) -> Dioption<T1, B>
    where
        Self: Sized,
        F: FnMut(T2) -> B,
    {
        self.map(|f| f, f)
    }
    pub fn push_first(&mut self, value: T1) -> bool {
        match self {
            Dioption::None => {
                *self = Dioption::First(value);
                true
            }
            Dioption::Second(s) => {
                unsafe {
                    *self = Dioption::Both(value, std::mem::replace(s, std::mem::zeroed()));
                }
                true
            }
            _ => false,
        }
    }
    pub fn push_second(&mut self, value: T2) -> bool {
        match self {
            Dioption::None => {
                *self = Dioption::Second(value);
                true
            }
            Dioption::First(f) => {
                unsafe {
                    *self = Dioption::Both(std::mem::replace(f, std::mem::zeroed()), value);
                }
                true
            }
            _ => false,
        }
    }
    pub fn set_first(&mut self, value: T1) {
        match self {
            Dioption::None | Dioption::First(_) => {
                *self = Dioption::First(value);
            }
            Dioption::Second(s) | Dioption::Both(_, s) => unsafe {
                *self = Dioption::Both(value, std::mem::replace(s, std::mem::zeroed()));
            },
        }
    }
    pub fn set_second(&mut self, value: T2) {
        match self {
            Dioption::None | Dioption::Second(_) => {
                *self = Dioption::Second(value);
            }
            Dioption::First(f) | Dioption::Both(f, _) => unsafe {
                *self = Dioption::Both(std::mem::replace(f, std::mem::zeroed()), value);
            },
        }
    }
}