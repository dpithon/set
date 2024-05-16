mod bounds;

pub use bounds::Bound;
use bounds::IBound::{self, Closed, LeftOpen, NegInfy, PosInfy, RightOpen};

use std::cmp::PartialEq;
use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum Interval {
    Empty,
    Infinity,
    Ival(IBound, IBound),
}

use Interval::*;

pub enum Union {
    Single(Interval),
    Couple(Interval, Interval),
}

impl Display for Union {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Union::Single(i) => write!(f, "{i}"),
            Union::Couple(a, b) => write!(f, "{a} U {b}"),
        }
    }
}

pub const INFINITY: Interval = Interval::Infinity;
pub const EMPTY: Interval = Interval::Empty;

impl Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Empty => write!(f, "∅"),
            Infinity => write!(f, "]-∞,+∞["),
            Ival(Closed(k1), Closed(k2)) if k1 == k2 => write!(f, "{{{k1:5.2}}}"),
            Ival(Closed(k1), Closed(k2)) => write!(f, "[{k1:5.2},{k2:5.2}]"),
            Ival(Closed(k1), RightOpen(k2)) => write!(f, "[{k1:5.2},{k2:5.2}["),
            Ival(Closed(k1), PosInfy) => write!(f, "[{k1:5.2},+∞["),
            Ival(LeftOpen(k1), Closed(k2)) => write!(f, "]{k1:5.2},{k2:5.2}]"),
            Ival(LeftOpen(k1), RightOpen(k2)) => write!(f, "]{k1:5.2},{k2:5.2}["),
            Ival(LeftOpen(k1), PosInfy) => write!(f, "]{k1:5.2},+∞["),
            Ival(NegInfy, Closed(k2)) => write!(f, "]-∞,{k2:5.2}]"),
            Ival(NegInfy, RightOpen(k2)) => write!(f, "]-∞,{k2:5.2}["),
            _ => panic!("Malformed interval {:?}", self),
        }
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Empty, Empty) => true,
            (Infinity, Infinity) => true,
            (Ival(a1, a2), Ival(b1, b2)) => a1 == b1 && a2 == b2,
            _ => false,
        }
    }
}

impl Interval {
    /// Build interval from given bounds
    ///
    /// # Returns
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use interval::{Interval, Open, Closed, Unbound};
    ///
    /// let a = Interval::new(Open(42.), Closed(43.));
    /// let b = Interval::new(Unbound, Unbound);
    /// let c = Interval::singleton(42.);
    ///
    /// assert_eq!(format!("{a}"), "]42.00,43.00]");
    /// assert_eq!(format!("{b}"), "]-∞,+∞[");
    /// assert_eq!(format!("{c}"), "{42.00}");
    /// ```
    ///
    pub fn new(b1: Bound, b2: Bound) -> Self {
        let b1 = match b1 {
            Bound::Open(k) => LeftOpen(k),
            Bound::Closed(k) => Closed(k),
            Bound::Unbound => NegInfy,
        };
        let b2 = match b2 {
            Bound::Open(k) => RightOpen(k),
            Bound::Closed(k) => Closed(k),
            Bound::Unbound => PosInfy,
        };

        if b2 < b1 {
            Empty
        } else if (b1, b2) == (NegInfy, PosInfy) {
            Infinity
        } else {
            Ival(b1, b2)
        }
    }

    pub fn singleton(k: f64) -> Self {
        Ival(Closed(k), Closed(k))
    }

    pub fn is_singleton(&self) -> bool {
        match self {
            Ival(Closed(k1), Closed(k2)) => k1 == k2,
            _ => false,
        }
    }

    pub fn is_empty(self) -> bool {
        self == Empty
    }

    pub fn union(self, other: Interval) -> Union {
        match (self, other) {
            (a, Empty) | (Empty, a) => Union::Single(a),
            (Infinity, _) | (_, Infinity) => Union::Single(Infinity),

            (Ival(a1, a2), Ival(b1, b2)) => {
                if self.overlap(other) || self.adhere_to(other) {
                    Union::Single(Ival(a1.min(b1), a2.max(b2)))
                } else if b1 > a2 {
                    Union::Couple(self, other)
                } else {
                    Union::Couple(other, self)
                }
            }
        }
    }

    /// Check if intervals overlap
    ///
    /// Note that `Empty` overlap nothing.
    ///
    fn overlap(self, other: Interval) -> bool {
        match (self, other) {
            (_, Empty) | (Empty, _) => false,
            (Infinity, _) | (_, Infinity) => true,
            (Ival(a1, a2), Ival(b1, b2)) => b2 >= a1 && b1 <= a2,
        }
    }

    /// Check if interval endpoints could rejoin (ie ]2 and (2, (2 and 2] ...)
    ///
    fn adhere_to(self, other: Interval) -> bool {
        match (self, other) {
            (_, Empty) | (Empty, _) => false,
            (Infinity, _) | (_, Infinity) => false,
            (Ival(a1, a2), Ival(b1, b2)) => {
                a1.closure() == b2.closure() || b1.closure() == a2.closure()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_overlap_1() {
        let a = Interval::new(Bound::Unbound, Bound::Unbound);
        let b = Interval::new(Bound::Unbound, Bound::Unbound);

        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_2() {
        let a = Interval::new(Bound::Unbound, Bound::Unbound);
        let b = EMPTY;

        assert!(!a.overlap(b));
    }

    #[test]
    fn test_overlap_3() {
        let a = EMPTY;
        let b = Interval::new(Bound::Unbound, Bound::Unbound);

        assert!(!a.overlap(b));
    }

    #[test]
    fn test_overlap_4() {
        let a = Interval::new(Bound::Closed(42.), Bound::Closed(43.));
        let b = Interval::new(Bound::Unbound, Bound::Unbound);
        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_5() {
        let a = Interval::new(Bound::Unbound, Bound::Unbound);
        let b = Interval::new(Bound::Closed(42.), Bound::Closed(43.));
        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_6() {
        let a = Interval::new(Bound::Closed(42.), Bound::Open(43.));
        let b = Interval::new(Bound::Unbound, Bound::Unbound);
        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_7() {
        let a = Interval::new(Bound::Unbound, Bound::Unbound);
        let b = Interval::new(Bound::Closed(42.), Bound::Open(43.));
        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_8() {
        let a = Interval::new(Bound::Open(42.), Bound::Open(43.));
        let b = Interval::new(Bound::Unbound, Bound::Unbound);
        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_9() {
        let a = Interval::new(Bound::Unbound, Bound::Unbound);
        let b = Interval::new(Bound::Open(42.), Bound::Open(43.));
        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_10() {
        let a = Interval::new(Bound::Unbound, Bound::Open(43.));
        let b = Interval::new(Bound::Unbound, Bound::Unbound);
        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_11() {
        let a = Interval::new(Bound::Unbound, Bound::Unbound);
        let b = Interval::new(Bound::Open(42.), Bound::Unbound);
        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_12() {
        let a = EMPTY;
        let b = Interval::new(Bound::Unbound, Bound::Unbound);

        assert!(!a.overlap(b));
    }

    #[test]
    fn test_overlap_13() {
        let a = EMPTY;
        let b = EMPTY;

        assert!(!a.overlap(b));
    }

    #[test]
    fn test_overlap_14() {
        let a = Interval::new(Bound::Closed(42.), Bound::Closed(52.));
        let b = Interval::new(Bound::Closed(42.), Bound::Closed(52.));
        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_15() {
        let a = Interval::new(Bound::Closed(42.), Bound::Closed(52.));
        let b = Interval::new(Bound::Open(42.), Bound::Open(52.));

        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_16() {
        let a = Interval::new(Bound::Closed(42.), Bound::Closed(52.));
        let b = Interval::new(Bound::Open(42.), Bound::Open(52.));

        assert!(b.overlap(a));
    }

    #[test]
    fn test_overlap_17() {
        let a = Interval::new(Bound::Open(42.), Bound::Closed(52.));
        let b = Interval::new(Bound::Open(42.), Bound::Open(52.));

        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_18() {
        let a = Interval::new(Bound::Open(42.), Bound::Closed(52.));
        let b = Interval::new(Bound::Open(42.), Bound::Open(52.));

        assert!(b.overlap(a));
    }

    #[test]
    fn test_overlap_19() {
        let a = Interval::new(Bound::Closed(42.), Bound::Open(52.));
        let b = Interval::new(Bound::Open(42.), Bound::Open(52.));

        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_20() {
        let a = Interval::new(Bound::Closed(42.), Bound::Open(52.));
        let b = Interval::new(Bound::Open(42.), Bound::Open(52.));

        assert!(b.overlap(a));
    }

    #[test]
    fn test_overlap_21() {
        let a = Interval::new(Bound::Open(42.), Bound::Open(52.));
        let b = Interval::new(Bound::Open(42.), Bound::Open(52.));
        assert!(a.overlap(b));
    }

    #[test]
    fn test_overlap_22() {
        let a = Interval::new(Bound::Unbound, Bound::Closed(42.));
        let b = Interval::new(Bound::Open(42.), Bound::Open(52.));
        assert!(!a.overlap(b));
    }

    #[test]
    fn test_overlap_23() {
        let a = Interval::new(Bound::Unbound, Bound::Open(42.));
        let b = Interval::new(Bound::Open(42.), Bound::Open(52.));
        assert!(!a.overlap(b));
    }

    #[test]
    fn test_overlap_24() {
        let a = Interval::new(Bound::Closed(52.), Bound::Unbound);
        let b = Interval::new(Bound::Open(42.), Bound::Open(52.));
        assert!(!a.overlap(b));
    }

    #[test]
    fn test_overlap_25() {
        let a = Interval::new(Bound::Open(52.), Bound::Unbound);
        let b = Interval::new(Bound::Open(42.), Bound::Open(52.));
        assert!(!a.overlap(b));
    }

    #[test]
    fn test_union_1() {
        assert!(matches!(EMPTY.union(EMPTY), Union::Single(Empty)));
    }

    #[test]
    fn test_union_2() {
        let i = Interval::new(Bound::Open(42.), Bound::Closed(43.));
        assert!(match i.union(EMPTY) {
            Union::Single(Ival(LeftOpen(k1), Closed(k2))) => k1 == 42. && k2 == 43.,
            _ => false,
        });
    }

    #[test]
    fn test_union_3() {
        let i = Interval::new(Bound::Open(42.), Bound::Closed(43.));
        assert!(match EMPTY.union(i) {
            Union::Single(Ival(LeftOpen(k1), Closed(k2))) => k1 == 42. && k2 == 43.,
            _ => false,
        });
    }

    #[test]
    fn test_union_4() {
        assert!(matches!(EMPTY.union(EMPTY), Union::Single(Empty)));
    }

    #[test]
    fn test_union_5() {
        assert!(matches!(INFINITY.union(INFINITY), Union::Single(Infinity)));
    }

    #[test]
    fn test_union_6() {
        let a = Interval::new(Bound::Closed(42.), Bound::Closed(52.));
        let b = Interval::new(Bound::Open(42.), Bound::Open(52.));
        assert!(matches!(
            a.union(b),
            Union::Single(Ival(Closed(b1), Closed(b2))) if b1 == 42. && b2 == 52.
        ));
    }

    #[test]
    fn test_union_7() {
        let a = Interval::new(Bound::Closed(42.), Bound::Closed(52.));
        let b = Interval::new(Bound::Open(42.), Bound::Open(52.));
        assert!(matches!(
            b.union(a),
            Union::Single(Ival(Closed(b1), Closed(b2))) if b1 == 42. && b2 == 52.
        ));
    }

    #[test]
    fn test_union_8() {
        let a = Interval::new(Bound::Closed(42.), Bound::Closed(52.));
        let b = Interval::new(Bound::Open(22.), Bound::Open(45.));
        assert!(matches!(
            b.union(a),
            Union::Single(Ival(LeftOpen(b1), Closed(b2))) if b1 == 22. && b2 == 52.
        ));
    }

    #[test]
    fn test_build_1() {
        assert!(matches!(
            Interval::new(Bound::Unbound, Bound::Unbound),
            Infinity
        ));
    }

    #[test]
    fn test_build_2() {
        assert!(match Interval::new(Bound::Unbound, Bound::Closed(42.)) {
            Ival(NegInfy, Closed(k)) => k == 42.,
            _ => false,
        });
    }

    #[test]
    fn test_build_3() {
        assert!(match Interval::new(Bound::Unbound, Bound::Open(42.)) {
            Ival(NegInfy, RightOpen(k)) => k == 42.,
            _ => false,
        });
    }

    #[test]
    fn test_build_4() {
        assert!(
            match Interval::new(Bound::Closed(42.), Bound::Closed(43.)) {
                Ival(Closed(k1), Closed(k2)) => k1 == 42. && k2 == 43.,
                _ => false,
            }
        );
    }

    #[test]
    fn test_build_5() {
        assert_eq!(Interval::new(Bound::Closed(43.), Bound::Closed(42.)), EMPTY);
    }

    #[test]
    fn test_build_6() {
        assert_eq!(Interval::new(Bound::Closed(42.), Bound::Open(42.)), EMPTY);
    }

    #[test]
    fn test_build_7() {
        assert!(match Interval::new(Bound::Closed(42.), Bound::Open(43.)) {
            Ival(Closed(k1), RightOpen(k2)) => k1 == 42. && k2 == 43.,
            _ => false,
        });
    }

    #[test]
    fn test_build_8() {
        assert_eq!(Interval::new(Bound::Closed(43.), Bound::Open(42.)), EMPTY);
    }

    #[test]
    fn test_build_9() {
        assert!(match Interval::new(Bound::Closed(42.), Bound::Unbound) {
            Ival(Closed(k), PosInfy) => k == 42.,
            _ => false,
        });
    }

    #[test]
    fn test_build_10() {
        assert!(match Interval::new(Bound::Open(42.), Bound::Closed(43.)) {
            Ival(LeftOpen(k1), Closed(k2)) => k1 == 42. && k2 == 43.,
            _ => false,
        });
    }

    #[test]
    fn test_build_11() {
        assert_eq!(Interval::new(Bound::Open(43.), Bound::Closed(42.)), EMPTY);
    }

    #[test]
    fn test_build_12() {
        assert_eq!(Interval::new(Bound::Open(42.), Bound::Closed(42.)), EMPTY);
    }

    #[test]
    fn test_build_13() {
        assert_eq!(Interval::new(Bound::Open(42.), Bound::Open(42.)), EMPTY);
    }

    #[test]
    fn test_build_14() {
        assert!(match Interval::new(Bound::Open(42.), Bound::Unbound) {
            Ival(LeftOpen(k), PosInfy) => k == 42.,
            _ => false,
        });
    }

    #[test]
    fn test_build_15() {
        assert!(match Interval::singleton(42.) {
            Ival(Closed(k1), Closed(k2)) => k1 == k2,
            _ => false,
        });
    }

    #[test]
    fn test_build_16() {
        assert!(Interval::singleton(42.).is_singleton());
    }

    #[test]
    fn test_empty_1() {
        assert!(Interval::new(Bound::Open(42.), Bound::Open(42.)).is_empty());
    }

    #[test]
    fn test_empty_2() {
        assert!(EMPTY.is_empty());
    }

    #[test]
    fn test_display_1() {
        assert_eq!(format!("{EMPTY}"), "∅");
    }

    #[test]
    fn test_display_2() {
        let inf = Interval::new(Bound::Unbound, Bound::Unbound);
        assert_eq!(format!("{inf}"), "]-∞,+∞[");
    }

    #[test]
    fn test_display_3() {
        let sing = Interval::new(Bound::Closed(42.), Bound::Closed(42.));
        assert_eq!(format!("{sing}"), "{42.00}");
    }

    #[test]
    fn test_display_4() {
        let i = Interval::new(Bound::Closed(42.), Bound::Closed(43.));
        assert_eq!(format!("{i}"), "[42.00,43.00]");
    }

    #[test]
    fn test_display_5() {
        let i = Interval::new(Bound::Closed(42.), Bound::Open(43.));
        assert_eq!(format!("{i}"), "[42.00,43.00[");
    }

    #[test]
    fn test_display_6() {
        let i = Interval::new(Bound::Closed(42.), Bound::Unbound);
        assert_eq!(format!("{i}"), "[42.00,+∞[");
    }

    #[test]
    fn test_display_7() {
        let i = Interval::new(Bound::Open(42.), Bound::Closed(43.00));
        assert_eq!(format!("{i}"), "]42.00,43.00]");
    }

    #[test]
    fn test_display_8() {
        let i = Interval::new(Bound::Open(42.), Bound::Open(43.00));
        assert_eq!(format!("{i}"), "]42.00,43.00[");
    }

    #[test]
    fn test_display_9() {
        let i = Interval::new(Bound::Open(42.), Bound::Unbound);
        assert_eq!(format!("{i}"), "]42.00,+∞[");
    }

    #[test]
    fn test_display_10() {
        let i = Interval::new(Bound::Unbound, Bound::Closed(42.));
        assert_eq!(format!("{i}"), "]-∞,42.00]");
    }

    #[test]
    fn test_display_11() {
        let i = Interval::new(Bound::Unbound, Bound::Open(42.));
        assert_eq!(format!("{i}"), "]-∞,42.00[");
    }
}
