use super::scaled_number::ScaledNumber;
use crate::from_and_to_numbers;

/// divide by 100.0 on (de)serialize, enforce non-negativity
pub type Numeric = ScaledNumber<i64, 100, 100, true>;
from_and_to_numbers!(Numeric, 100, i64);

#[cfg(test)]
mod tests {

    #[test]
    fn should_not_allow_negative_value_when_deserializing_numeric() {
        #[derive(serde::Deserialize, serde::Serialize, Debug)]
        struct NumericV {
            amount: super::Numeric,
        }
        let value = r#"{"amount": -1}"#;
        let result = serde_json::from_str::<NumericV>(value);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "must be non-negative at line 1 column 14"
        );
        // positive should pass
        let value = r#"{"amount": 1}"#;
        let result = serde_json::from_str::<NumericV>(value);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.amount, super::Numeric::new(100));
        // should divided when serializing
        let value = serde_json::to_string(&result).unwrap();
        assert_eq!(value, r#"{"amount":1.0}"#);
    }
}
