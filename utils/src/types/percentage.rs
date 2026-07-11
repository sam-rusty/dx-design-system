use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::iter::Sum;
use std::ops::*;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use validator::ValidateRange;

use super::Numeric;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Percentage(f32);

impl Percentage {
    #[inline]
    pub fn new(value: f32) -> Self {
        Self(value)
    }

    pub fn new_max_100(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    #[inline]
    pub fn value(&self) -> f32 {
        self.0
    }

    #[inline]
    pub fn abs(&self) -> f32 {
        self.0.abs()
    }

    #[inline]
    pub fn min_zero(self) -> Self {
        if self.0.is_sign_negative() {
            Self(0.0)
        } else {
            self
        }
    }
}

impl Display for Percentage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0 * 100.0)
    }
}

// Conversions and comparisons with f32
impl From<f32> for Percentage {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl PartialEq<f32> for Percentage {
    #[inline]
    fn eq(&self, other: &f32) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<f32> for Percentage {
    #[inline]
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

// Arithmetic operations
impl Sub<f32> for Percentage {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: f32) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Sub for Percentage {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Percentage {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Add for Percentage {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<f32> for Percentage {
    type Output = Self;
    #[inline]
    fn add(self, rhs: f32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign for Percentage {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sum for Percentage {
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Add::add)
    }
}

// Validation
impl ValidateRange<f32> for Percentage {
    fn greater_than(&self, max: f32) -> Option<bool> {
        Some(self.0 > max)
    }
    fn less_than(&self, min: f32) -> Option<bool> {
        Some(self.0 < min)
    }
}

// Serde
impl Serialize for Percentage {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let scaled = (self.0 * 100.0 * 1000.0).round() / 1000.0;
        serializer.serialize_f32(scaled)
    }
}

impl<'de> Deserialize<'de> for Percentage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let val = f64::deserialize(deserializer)? / 100.0;
        if val < 0.0 {
            return Err(serde::de::Error::custom("must be non-negative"));
        }
        if val > f32::MAX as f64 {
            return Err(serde::de::Error::custom("value too large"));
        }
        Ok(Self(val as f32))
    }
}

#[cfg(feature = "server")]
mod sqlx_impls {
    use sqlx::encode::IsNull;
    use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
    use sqlx::{Decode, Encode, Postgres, Type};

    use super::Percentage;

    impl Type<Postgres> for Percentage {
        fn type_info() -> PgTypeInfo {
            <f64 as Type<Postgres>>::type_info()
        }
    }

    impl<'q> Encode<'q, Postgres> for Percentage {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<IsNull, sqlx::error::BoxDynError> {
            let v = self.0 as f64;
            <f64 as Encode<Postgres>>::encode_by_ref(&v, buf)
        }
    }

    impl<'r> Decode<'r, Postgres> for Percentage {
        fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
            let v = <f64 as Decode<Postgres>>::decode(value)?;
            Ok(Percentage::new(v as f32))
        }
    }
}

// Percentage × Numeric = Numeric
impl Mul<Percentage> for Numeric {
    type Output = Numeric;
    #[inline]
    fn mul(self, rhs: Percentage) -> Self::Output {
        let product = (self.value() as f64 * rhs.value() as f64).round();
        Numeric::new(product as i64)
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::Percentage;

    #[derive(serde::Deserialize, Serialize)]
    struct TestInput {
        percentage: Percentage,
    }

    #[test]
    fn should_divided_when_deserialize() {
        let input = r#"{"percentage": 50.0}"#;
        let result: TestInput = serde_json::from_str(input).unwrap();
        assert_eq!(result.percentage, Percentage::new(0.5));
    }

    #[test]
    fn should_multiply_when_serialize() {
        let input = TestInput {
            percentage: Percentage::new(0.5),
        };
        let result = serde_json::to_string(&input).unwrap();
        assert_eq!(result, r#"{"percentage":50.0}"#);
    }
}
