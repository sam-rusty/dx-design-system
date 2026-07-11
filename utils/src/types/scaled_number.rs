// src/scaled_number.rs

use std::fmt::{self, Display, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div as StdDiv, Mul as StdMul, Sub, SubAssign};

use num_traits::{NumCast, Signed, ToPrimitive, Zero};
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use validator::ValidateRange;

use crate::{AppError, Result};

/// A zero-cost new type for an integer `T` that
///  - divides by `DIV` on (de)serialize & Display,
///  - multiplies by `MUL` only on Deserialize,
///  - enforces non-negativity if `POS` is `true`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ScaledNumber<T, const DIV: i64, const MUL: i64, const POS: bool>(pub T);

impl<T, const DIV: i64, const MUL: i64, const POS: bool> ScaledNumber<T, DIV, MUL, POS> {
    /// Construct from the raw integer
    #[inline]
    pub fn new(raw: T) -> Self {
        Self(raw)
    }

    /// Extract the raw integer
    #[inline]
    pub fn value(self) -> T {
        self.0
    }
}

impl<T, const DIV: i64, const MUL: i64, const POS: bool> ScaledNumber<T, DIV, MUL, POS>
where
    T: ToPrimitive + NumCast + Copy,
{
    /// Fallible multiplication by an `f32` factor
    pub fn try_mul(self, rhs: f32) -> Result<Self> {
        if rhs.is_nan() {
            return Err(AppError::BadRequest(
                "ScaledNumber: multiplication by NaN".into(),
            ));
        }
        let f = self
            .0
            .to_f64()
            .ok_or(AppError::BadRequest("ScaledNumber: to_f64 failed".into()))?;
        let scaled = (f * rhs as f64).round();
        let i = NumCast::from(scaled).ok_or(AppError::BadRequest(
            "ScaledNumber: NumCast from f64 failed".into(),
        ))?;
        Ok(ScaledNumber(i))
    }

    /// Fallible division by an `f32` factor
    pub fn try_div(self, rhs: f32) -> Result<Self>
    where
        T: Zero,
    {
        if rhs == 0.0 || rhs.is_nan() {
            return Ok(ScaledNumber(T::zero()));
        }
        let f = self
            .0
            .to_f64()
            .ok_or(AppError::BadRequest("ScaledNumber: to_f64 failed".into()))?;
        let scaled = (f / rhs as f64).round();
        let i = NumCast::from(scaled).ok_or(AppError::BadRequest(
            "ScaledNumber: NumCast from f64 failed".into(),
        ))?;
        Ok(ScaledNumber(i))
    }

    /// Fallible ratio of two scaled numbers → `f32`
    pub fn try_ratio(self, rhs: Self) -> Result<f32> {
        let a = self
            .0
            .to_f32()
            .ok_or(AppError::BadRequest("ScaledNumber: to_f32 failed".into()))?;
        let b = rhs
            .0
            .to_f32()
            .ok_or(AppError::BadRequest("ScaledNumber: to_f32 failed".into()))?;
        if b == 0.0 || b.is_nan() {
            return Ok(f32::zero());
        }
        Ok(a / b)
    }

    /// Fallible conversion
    pub fn try_into<U, const DIV2: i64, const MUL2: i64, const POS2: bool>(
        self,
    ) -> Result<ScaledNumber<U, DIV2, MUL2, POS2>>
    where
        U: NumCast,
    {
        // now we know T: ToPrimitive, so to_f64() is in scope
        let real = self
            .0
            .to_f64()
            .ok_or(AppError::BadRequest("ScaledNumber: to_f64 failed".into()))?
            / DIV as f64;

        let scaled = (real * DIV2 as f64).round();
        let u = NumCast::from(scaled).ok_or(AppError::BadRequest(
            "ScaledNumber: NumCast from f64 failed".into(),
        ))?;

        Ok(ScaledNumber(u))
    }

    /// Alias for symmetry if you really want `try_from`
    pub fn try_from<U, const DIV2: i64, const MUL2: i64, const POS2: bool>(
        self,
    ) -> Result<ScaledNumber<U, DIV2, MUL2, POS2>>
    where
        U: NumCast,
    {
        self.try_into::<U, DIV2, MUL2, POS2>()
    }
}

// Add / AddAssign / Sub / SubAssign / Sum
impl<T, const DIV: i64, const MUL: i64, const POS: bool> Add for ScaledNumber<T, DIV, MUL, POS>
where
    T: Add<Output = T> + Copy,
{
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl<T, const DIV: i64, const MUL: i64, const POS: bool> AddAssign
    for ScaledNumber<T, DIV, MUL, POS>
where
    T: AddAssign + Copy,
{
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<T, const DIV: i64, const MUL: i64, const POS: bool> Sub for ScaledNumber<T, DIV, MUL, POS>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl<T, const DIV: i64, const MUL: i64, const POS: bool> SubAssign
    for ScaledNumber<T, DIV, MUL, POS>
where
    T: SubAssign + Copy,
{
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl<T, const DIV: i64, const MUL: i64, const POS: bool> Sum for ScaledNumber<T, DIV, MUL, POS>
where
    T: Sum<T> + Copy,
{
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let raw: T = iter.map(|s| s.0).sum();
        Self(raw)
    }
}

// Mul<f32>
impl<T, const DIV: i64, const MUL: i64, const POS: bool> StdMul<f32>
    for ScaledNumber<T, DIV, MUL, POS>
where
    T: ToPrimitive + NumCast + Copy,
{
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        self.try_mul(rhs)
            .expect("ScaledNumber::mul: conversion failed")
    }
}

// Div<f32>
impl<T, const DIV: i64, const MUL: i64, const POS: bool> StdDiv<f32>
    for ScaledNumber<T, DIV, MUL, POS>
where
    T: ToPrimitive + NumCast + Copy + Zero,
{
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        self.try_div(rhs)
            .expect("ScaledNumber::div: conversion failed")
    }
}

// Ratio Div → f32
impl<T, const DIV: i64, const MUL: i64, const POS: bool> std::ops::Div
    for ScaledNumber<T, DIV, MUL, POS>
where
    T: ToPrimitive + NumCast + Copy,
{
    type Output = f32;

    #[inline]
    fn div(self, rhs: Self) -> f32 {
        // now try_ratio is in scope because T: NumCast + Copy
        self.try_ratio(rhs)
            .expect("ScaledNumber::ratio: conversion failed")
    }
}

// Lossy conversion to f64 in real units (raw / DIV). Used for read-only math
// where loss of precision is acceptable (charts, projections, formatting).
impl<T, const DIV: i64, const MUL: i64, const POS: bool> From<ScaledNumber<T, DIV, MUL, POS>>
    for f64
where
    T: ToPrimitive,
{
    fn from(value: ScaledNumber<T, DIV, MUL, POS>) -> Self {
        value.0.to_f64().expect("ScaledNumber: to_f64 failed") / DIV as f64
    }
}

// Display
impl<T, const DIV: i64, const MUL: i64, const POS: bool> Display for ScaledNumber<T, DIV, MUL, POS>
where
    T: ToPrimitive,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let v = self.0.to_f64().expect("ScaledNumber::fmt: to_f64 failed") / DIV as f64;
        write!(f, "{v:.2}")
    }
}

// Serde Serialize
impl<T, const DIV: i64, const MUL: i64, const POS: bool> Serialize
    for ScaledNumber<T, DIV, MUL, POS>
where
    T: ToPrimitive,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let v = self
            .0
            .to_f64()
            .expect("ScaledNumber::serialize: to_f64 failed")
            / DIV as f64;
        if POS && v < 0.0 {
            return Err(<S::Error as serde::ser::Error>::custom(
                "must be non-negative",
            ));
        }
        serializer.serialize_f64(v)
    }
}

// Serde Deserialize
impl<'de, T, const DIV: i64, const MUL: i64, const POS: bool> Deserialize<'de>
    for ScaledNumber<T, DIV, MUL, POS>
where
    T: NumCast + Zero + PartialOrd,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = f64::deserialize(deserializer)?;
        let value = (raw * MUL as f64).round();
        let inner: T = NumCast::from(value).ok_or_else(|| DeError::custom("cast failed"))?;
        // in future, if want to allow negative values form the user, we can add condition here on POS
        if POS && inner < T::zero() {
            return Err(DeError::custom("must be non-negative"));
        }
        Ok(Self(inner))
    }
}

// Signed helpers
impl<T, const DIV: i64, const MUL: i64, const POS: bool> ScaledNumber<T, DIV, MUL, POS>
where
    T: Signed + Copy,
{
    #[inline]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    #[inline]
    pub fn positive(&mut self) {
        if self.0.is_negative() {
            self.0 = self.0.abs();
        }
    }
}

impl<T, const DIV: i64, const MUL: i64, const POS: bool> ScaledNumber<T, DIV, MUL, POS>
where
    T: Zero + Copy + PartialOrd,
{
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    #[inline]
    pub fn is_positive(&self) -> bool {
        self.0 > T::zero()
    }

    #[inline]
    pub fn is_negative(&self) -> bool {
        self.0 < T::zero()
    }
}

// Validator range
impl<T, const DIV: i64, const MUL: i64, const POS: bool> ValidateRange<T>
    for ScaledNumber<T, DIV, MUL, POS>
where
    T: PartialOrd + Copy,
{
    fn greater_than(&self, max: T) -> Option<bool> {
        Some(self.0 > max)
    }
    fn less_than(&self, min: T) -> Option<bool> {
        Some(self.0 < min)
    }
}

#[cfg(feature = "server")]
mod sqlx_impls {
    use sqlx::encode::IsNull;
    use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
    use sqlx::{Decode, Encode, Postgres, Type};

    use super::ScaledNumber;

    impl<const DIV: i64, const MUL: i64, const POS: bool> Type<Postgres>
        for ScaledNumber<i64, DIV, MUL, POS>
    {
        fn type_info() -> PgTypeInfo {
            <i64 as Type<Postgres>>::type_info()
        }
    }

    impl<'q, const DIV: i64, const MUL: i64, const POS: bool> Encode<'q, Postgres>
        for ScaledNumber<i64, DIV, MUL, POS>
    {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> std::result::Result<IsNull, sqlx::error::BoxDynError> {
            <i64 as Encode<Postgres>>::encode_by_ref(&self.0, buf)
        }
    }

    impl<'r, const DIV: i64, const MUL: i64, const POS: bool> Decode<'r, Postgres>
        for ScaledNumber<i64, DIV, MUL, POS>
    {
        fn decode(value: PgValueRef<'r>) -> std::result::Result<Self, sqlx::error::BoxDynError> {
            let v = <i64 as Decode<Postgres>>::decode(value)?;
            Ok(Self(v))
        }
    }
}
