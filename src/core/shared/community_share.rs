// What the filer reports of one income item on a community-property return:
// None when the return divides nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Share {
    // The owner's part as a fraction; the other spouse reports the rest.
    pub(crate) owner_num: i64,
    pub(crate) owner_den: i64,
    // Property gains follow ownership independently of separate-property income.
    pub(crate) gain_num: i64,
    pub(crate) gain_den: i64,
    // The filer owns the document.
    pub(crate) filer_owns: bool,
}

impl Share {
    // The whole item is the filer's: nothing is divided.
    pub(crate) fn whole(&self) -> bool {
        self.filer_owns && self.owner_num == self.owner_den && self.gain_num == self.gain_den
    }
    // Whether the active income fraction leaves the physical amount undivided.
    pub(crate) fn income_fraction_is_whole(&self) -> bool {
        self.owner_num == self.owner_den
    }
    // The filer owns the document (and so the operator's own deductions).
    pub(crate) fn filer_owns(&self) -> bool {
        self.filer_owns
    }
    // Nothing of the item is the filer's.
    pub(crate) fn none(&self) -> bool {
        if self.filer_owns {
            self.owner_num == 0 && self.gain_num == 0
        } else {
            self.owner_num == self.owner_den && self.gain_num == self.gain_den
        }
    }
    // The filer's part of `amount`; the owner keeps an odd cent so the two
    // columns of Form 8958 add to the total.
    pub(crate) fn filer(&self, amount: i64) -> i64 {
        let owners = crate::kernel::ratio::fraction(amount, self.owner_num, self.owner_den);
        if self.filer_owns { owners } else { amount - owners }
    }
    // Preserve the owner-first cent boundary and the other spouse's exact
    // residual without narrowing an intermediate source expense total.
    pub(crate) fn filer_exact(
        &self,
        amount: &crate::kernel::ratio::exact::Exact,
    ) -> crate::kernel::ratio::exact::Exact {
        use crate::kernel::ratio::exact;
        let owners = exact::rounded_exact(
            &(amount * exact::ratio(self.owner_num, self.owner_den).expect("owner denominator")),
        );
        if self.filer_owns { owners } else { amount - owners }
    }
    pub(crate) fn gain(&self, amount: i64) -> i64 {
        self.for_gains().filer(amount)
    }
    pub(crate) fn for_gains(&self) -> Self {
        Self { owner_num: self.gain_num, owner_den: self.gain_den, ..*self }
    }
    pub(crate) fn counterpart(&self) -> Self {
        Self { filer_owns: !self.filer_owns, ..*self }
    }
}

// The field lists beside each source type classify money, property gains and
// facts exhaustively. A newly added field must be classified before compilation
// can succeed. The only arithmetic is Share's shared-kernel projection.
pub(crate) trait Divide: Clone {
    fn divided(&self, share: Share) -> Self;
}
impl Divide for i64 {
    fn divided(&self, share: Share) -> Self {
        share.filer(*self)
    }
}
impl<T: Divide> Divide for alloc::boxed::Box<T> {
    fn divided(&self, share: Share) -> Self {
        alloc::boxed::Box::new((**self).divided(share))
    }
}
impl<T: Divide> Divide for Option<T> {
    fn divided(&self, share: Share) -> Self {
        self.as_ref().map(|value| value.divided(share))
    }
}
impl<T: Divide> Divide for alloc::vec::Vec<T> {
    fn divided(&self, share: Share) -> Self {
        self.iter().map(|value| value.divided(share)).collect()
    }
}
macro_rules! divide_record {
    ($ty:ty { amounts: [$($amount:ident),* $(,)?], gains: [$($gain:ident),* $(,)?], facts: [$($fact:ident),* $(,)?] $(,)? }) => {
        impl crate::community_share::Divide for $ty {
            fn divided(&self, share: crate::community_share::Share) -> Self {
                let _ = share;
                let Self { $($amount,)* $($gain,)* $($fact,)* } = self;
                Self {
                    $($amount: crate::community_share::Divide::divided($amount, share),)*
                    $($gain: crate::community_share::Divide::divided($gain, share.for_gains()),)*
                    $($fact: $fact.clone(),)*
                }
            }
        }
    };
}
pub(crate) use divide_record;
macro_rules! divide_enum {
    ($ty:ident {
        records: [$($variant:ident { amounts: [$($amount:ident),* $(,)?], facts: [$($fact:ident),* $(,)?] }),* $(,)?],
        units: [$($unit:ident),* $(,)?],
        tuples: [$($tuple:ident),* $(,)?] $(,)?
    }) => {
        impl crate::community_share::Divide for $ty {
            fn divided(&self, share: crate::community_share::Share) -> Self {
                let _ = share;
                match self {
                    $(Self::$variant { $($amount,)* $($fact,)* } => Self::$variant {
                        $($amount: crate::community_share::Divide::divided($amount, share),)*
                        $($fact: $fact.clone(),)*
                    },)*
                    $(Self::$unit => Self::$unit,)*
                    $(Self::$tuple(value) => Self::$tuple(crate::community_share::Divide::divided(value, share)),)*
                }
            }
        }
    };
}
pub(crate) use divide_enum;
