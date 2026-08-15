//! Byte quantities with human-readable binary-unit display.

use std::fmt;

/// One kibibyte.
pub const KIB: u64 = 1024;
/// One mebibyte.
pub const MIB: u64 = 1024 * KIB;
/// One gibibyte.
pub const GIB: u64 = 1024 * MIB;

/// A byte count that displays in binary units (KiB/MiB/GiB) with two decimals.
///
/// The planner and CLI always speak GiB with two decimals (e.g. `1.50 GiB`)
/// so residual-space output matches the design's worked examples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ByteSize(pub u64);

impl ByteSize {
    /// Construct from a raw byte count.
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Construct from whole gibibytes.
    pub const fn from_gib(gib: u64) -> Self {
        Self(gib * GIB)
    }

    /// Raw byte count.
    pub const fn bytes(self) -> u64 {
        self.0
    }

    /// Value in gibibytes as a float (for display and residual math).
    pub fn as_gib(self) -> f64 {
        self.0 as f64 / GIB as f64
    }

    /// Saturating subtraction.
    pub const fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

impl From<u64> for ByteSize {
    fn from(bytes: u64) -> Self {
        Self(bytes)
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = self.0;
        if b >= GIB {
            write!(f, "{:.2} GiB", b as f64 / GIB as f64)
        } else if b >= MIB {
            write!(f, "{:.2} MiB", b as f64 / MIB as f64)
        } else if b >= KIB {
            write!(f, "{:.2} KiB", b as f64 / KIB as f64)
        } else {
            write!(f, "{b} B")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_bytes() {
        assert_eq!(ByteSize::new(0).to_string(), "0 B");
        assert_eq!(ByteSize::new(1023).to_string(), "1023 B");
    }

    #[test]
    fn displays_binary_units_with_two_decimals() {
        assert_eq!(ByteSize::new(KIB).to_string(), "1.00 KiB");
        assert_eq!(ByteSize::new(MIB + MIB / 2).to_string(), "1.50 MiB");
        assert_eq!(ByteSize::new(GIB + GIB / 2).to_string(), "1.50 GiB");
        // The design's worked example: 1.50 GiB minimum residual.
        assert_eq!(ByteSize::new(1_610_612_736).to_string(), "1.50 GiB");
    }

    #[test]
    fn gib_round_trip() {
        assert_eq!(ByteSize::from_gib(8).bytes(), 8 * GIB);
        assert!((ByteSize::from_gib(4).as_gib() - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn saturating_sub_floors_at_zero() {
        let a = ByteSize::new(10);
        let b = ByteSize::new(25);
        assert_eq!(a.saturating_sub(b), ByteSize::new(0));
        assert_eq!(b.saturating_sub(a), ByteSize::new(15));
    }
}
