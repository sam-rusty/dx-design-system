/// Impl the 4 `From<…>` conversions for any scaled type.
#[macro_export]
macro_rules! from_and_to_numbers {
    ($alias:ty, $scale:expr, $t:ty) => {
        impl From<f32> for $alias {
            fn from(value: f32) -> Self {
                // unconditionally round(value * SCALE)
                let raw = (value as f64 * $scale as f64).round();
                super::scaled_number::ScaledNumber(num_traits::NumCast::from(raw).unwrap())
            }
        }
        impl From<f64> for $alias {
            fn from(value: f64) -> Self {
                let raw = (value * $scale as f64).round();
                super::scaled_number::ScaledNumber(num_traits::NumCast::from(raw).unwrap())
            }
        }
        impl From<i32> for $alias {
            fn from(value: i32) -> Self {
                super::scaled_number::ScaledNumber(value as _)
            }
        }
        impl From<i64> for $alias {
            fn from(value: i64) -> Self {
                super::scaled_number::ScaledNumber(value as $t)
            }
        }
    };
}
