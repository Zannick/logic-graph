//! Observation utilities for use with the Matcher Trie.

use std::cmp::{max, min, Ord};
use std::fmt::Debug;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum IntegerObservation<T> {
    #[default]
    Unknown,
    Eq(T),
    Ge(T),
    Le(T),
    Range(T, T),
}

impl<T> IntegerObservation<T>
where
    T: Copy + Ord + Debug + Eq + std::ops::Add<Output = T>,
{
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unknown, obs) | (obs, Self::Unknown) => obs,
            (Self::Eq(i), Self::Eq(j)) if i == j => self,
            (Self::Eq(i), Self::Ge(v)) | (Self::Ge(v), Self::Eq(i)) if i >= v => Self::Eq(i),
            (Self::Eq(i), Self::Le(v)) | (Self::Le(v), Self::Eq(i)) if i <= v => Self::Eq(i),
            (Self::Eq(i), Self::Range(lo, hi)) | (Self::Range(lo, hi), Self::Eq(i))
                if i >= lo && i <= hi =>
            {
                Self::Eq(i)
            }
            (Self::Ge(g1), Self::Ge(g2)) => Self::Ge(max(g1, g2)),
            (Self::Ge(lo), Self::Le(hi)) | (Self::Le(hi), Self::Ge(lo)) if lo <= hi => {
                Self::Range(lo, hi)
            }
            (Self::Ge(v), Self::Range(lo, hi)) | (Self::Range(lo, hi), Self::Ge(v)) if v <= hi => {
                let lo = max(v, lo);
                if lo == hi {
                    Self::Eq(lo)
                } else {
                    Self::Range(lo, hi)
                }
            }
            (Self::Le(v1), Self::Le(v2)) => Self::Le(min(v1, v2)),
            (Self::Le(v), Self::Range(lo, hi)) | (Self::Range(lo, hi), Self::Le(v)) if v >= lo => {
                let hi = min(v, hi);
                if lo == hi {
                    Self::Eq(lo)
                } else {
                    Self::Range(lo, min(v, hi))
                }
            }
            // two ranges get clamped
            (Self::Range(lo1, hi1), Self::Range(lo2, hi2)) if lo1 <= hi2 && lo2 <= hi1 => {
                let lo = max(lo1, lo2);
                let hi = min(hi1, hi2);
                if lo == hi {
                    Self::Eq(lo)
                } else {
                    Self::Range(lo, hi)
                }
            }
            _ => panic!("Contradictory observations: {:?} vs {:?}", self, other),
        }
    }

    pub fn shift(self, i: T) -> Self {
        match self {
            IntegerObservation::Unknown => self,
            IntegerObservation::Eq(v) => IntegerObservation::Eq(v + i),
            IntegerObservation::Ge(v) => IntegerObservation::Ge(v + i),
            IntegerObservation::Le(v) => IntegerObservation::Le(v + i),
            IntegerObservation::Range(lo, hi) => IntegerObservation::Range(lo + i, hi + i),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn eq() {
        let e = IntegerObservation::Eq(4);
        let ge = IntegerObservation::Ge(2);
        let le = IntegerObservation::Le(5);
        let range = IntegerObservation::Range(0, 6);
        assert_eq!(e, e.combine(e));
        assert_eq!(e, e.combine(ge));
        assert_eq!(e, e.combine(le));
        assert_eq!(e, e.combine(range));
    }

    #[test]
    fn ge() {
        let e = IntegerObservation::Eq(4);
        let ge = IntegerObservation::Ge(2);
        let ge2 = IntegerObservation::Ge(1);
        let ge3 = IntegerObservation::Ge(7);
        let le = IntegerObservation::Le(5);
        let range = IntegerObservation::Range(0, 6);
        let range2 = IntegerObservation::Range(0, 2);
        assert_eq!(e, ge.combine(e));
        assert_eq!(IntegerObservation::Range(2, 5), ge.combine(le));
        assert_eq!(ge, ge.combine(ge2));
        assert_eq!(ge3, ge.combine(ge3));
        assert_eq!(IntegerObservation::Range(2, 6), ge.combine(range));
        assert_eq!(IntegerObservation::Eq(2), ge.combine(range2));
    }

    #[test]
    fn le() {
        let e = IntegerObservation::Eq(4);
        let ge = IntegerObservation::Ge(2);
        let le = IntegerObservation::Le(5);
        let le2 = IntegerObservation::Le(4);
        let le3 = IntegerObservation::Le(8);
        let range = IntegerObservation::Range(0, 6);
        let range2 = IntegerObservation::Range(5, 8);
        assert_eq!(e, le.combine(e));
        assert_eq!(IntegerObservation::Range(2, 5), le.combine(ge));
        assert_eq!(le2, le.combine(le2));
        assert_eq!(le, le.combine(le3));
        assert_eq!(IntegerObservation::Range(0, 5), le.combine(range));
        assert_eq!(IntegerObservation::Eq(5), le.combine(range2));
    }

    #[test]
    fn range() {
        let e = IntegerObservation::Eq(4);
        let ge = IntegerObservation::Ge(2);
        let ge2 = IntegerObservation::Ge(6);
        let le = IntegerObservation::Le(5);
        let range = IntegerObservation::Range(0, 6);
        let range2 = IntegerObservation::Range(4, 6);
        let range3 = IntegerObservation::Range(6, 12);
        assert_eq!(e, range.combine(e));
        assert_eq!(IntegerObservation::Range(2, 6), range.combine(ge));
        assert_eq!(IntegerObservation::Range(0, 5), range.combine(le));
        assert_eq!(range2, range.combine(range2));
        assert_eq!(IntegerObservation::Eq(6), range.combine(ge2));
        assert_eq!(IntegerObservation::Eq(6), range.combine(range3));
    }
}
