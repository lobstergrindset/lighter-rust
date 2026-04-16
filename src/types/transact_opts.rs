use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::constants::{FEE_TICK, MAX_ACCOUNT_INDEX};
use crate::error::{Result, SdkError};

const ATTRIBUTE_TYPE_INTEGRATOR_ACCOUNT_INDEX: &str = "1";
const ATTRIBUTE_TYPE_INTEGRATOR_TAKER_FEE: &str = "2";
const ATTRIBUTE_TYPE_INTEGRATOR_MAKER_FEE: &str = "3";
const ATTRIBUTE_TYPE_SKIP_TX_NONCE: &str = "4";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct L2TxAttributes {
    pub integrator_account_index: Option<i64>,
    pub integrator_taker_fee: Option<u32>,
    pub integrator_maker_fee: Option<u32>,
    pub skip_nonce: Option<u8>,
}

impl L2TxAttributes {
    pub fn skip_nonce_enabled() -> Self {
        Self {
            skip_nonce: Some(1),
            ..Self::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.integrator_account_index.unwrap_or(0) == 0
            && self.integrator_taker_fee.unwrap_or(0) == 0
            && self.integrator_maker_fee.unwrap_or(0) == 0
            && self.skip_nonce.unwrap_or(0) == 0
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(index) = self.integrator_account_index
            && !(0..=MAX_ACCOUNT_INDEX).contains(&index)
        {
            return Err(SdkError::IntegratorAccountIndexInvalidRange);
        }

        if let Some(fee) = self.integrator_taker_fee
            && fee as i64 > FEE_TICK
        {
            return Err(SdkError::IntegratorFeeInvalidRange);
        }

        if let Some(fee) = self.integrator_maker_fee
            && fee as i64 > FEE_TICK
        {
            return Err(SdkError::IntegratorFeeInvalidRange);
        }

        if let Some(skip_nonce) = self.skip_nonce
            && skip_nonce != 1
        {
            return Err(SdkError::NonceSkipAttributeInvalid);
        }

        let has_fees = self.integrator_taker_fee.unwrap_or(0) != 0
            || self.integrator_maker_fee.unwrap_or(0) != 0;
        if has_fees && self.integrator_account_index.unwrap_or(0) == 0 {
            return Err(SdkError::IntegratorAccountIndexRequiredForNonZeroFees);
        }

        Ok(())
    }
}

impl Serialize for L2TxAttributes {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(index) = self.integrator_account_index.filter(|index| *index != 0) {
            map.serialize_entry(ATTRIBUTE_TYPE_INTEGRATOR_ACCOUNT_INDEX, &index)?;
        }
        if let Some(fee) = self.integrator_taker_fee.filter(|fee| *fee != 0) {
            map.serialize_entry(ATTRIBUTE_TYPE_INTEGRATOR_TAKER_FEE, &fee)?;
        }
        if let Some(fee) = self.integrator_maker_fee.filter(|fee| *fee != 0) {
            map.serialize_entry(ATTRIBUTE_TYPE_INTEGRATOR_MAKER_FEE, &fee)?;
        }
        if let Some(skip_nonce) = self.skip_nonce.filter(|skip_nonce| *skip_nonce != 0) {
            map.serialize_entry(ATTRIBUTE_TYPE_SKIP_TX_NONCE, &skip_nonce)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for L2TxAttributes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct L2TxAttributesVisitor;

        impl<'de> Visitor<'de> for L2TxAttributesVisitor {
            type Value = L2TxAttributes;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a L2TxAttributes map")
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut attributes = L2TxAttributes::default();

                while let Some((key, value)) = map.next_entry::<String, serde_json::Value>()? {
                    match key.as_str() {
                        ATTRIBUTE_TYPE_INTEGRATOR_ACCOUNT_INDEX | "IntegratorAccountIndex" => {
                            let value: i64 =
                                serde_json::from_value(value).map_err(de::Error::custom)?;
                            if value != 0 {
                                attributes.integrator_account_index = Some(value);
                            }
                        }
                        ATTRIBUTE_TYPE_INTEGRATOR_TAKER_FEE | "IntegratorTakerFee" => {
                            let value: u32 =
                                serde_json::from_value(value).map_err(de::Error::custom)?;
                            if value != 0 {
                                attributes.integrator_taker_fee = Some(value);
                            }
                        }
                        ATTRIBUTE_TYPE_INTEGRATOR_MAKER_FEE | "IntegratorMakerFee" => {
                            let value: u32 =
                                serde_json::from_value(value).map_err(de::Error::custom)?;
                            if value != 0 {
                                attributes.integrator_maker_fee = Some(value);
                            }
                        }
                        ATTRIBUTE_TYPE_SKIP_TX_NONCE | "SkipNonce" => {
                            let value: u8 =
                                serde_json::from_value(value).map_err(de::Error::custom)?;
                            if value != 0 {
                                attributes.skip_nonce = Some(value);
                            }
                        }
                        _ => {}
                    }
                }

                Ok(attributes)
            }
        }

        deserializer.deserialize_map(L2TxAttributesVisitor)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TransactOpts {
    pub from_account_index: Option<i64>,
    pub api_key_index: Option<u8>,
    pub expired_at: i64,
    pub nonce: Option<i64>,
    pub tx_attributes: Option<L2TxAttributes>,
    pub dry_run: bool,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::L2TxAttributes;
    use crate::error::SdkError;

    #[test]
    fn serializes_skip_nonce_as_numeric_attribute_key() {
        let attributes = L2TxAttributes {
            integrator_account_index: Some(7),
            skip_nonce: Some(1),
            ..L2TxAttributes::default()
        };

        let value = serde_json::to_value(&attributes).unwrap();

        assert_eq!(value, json!({ "1": 7, "4": 1 }));
    }

    #[test]
    fn deserializes_numeric_and_named_attribute_keys() {
        let numeric: L2TxAttributes = serde_json::from_value(json!({ "2": 10, "4": 1 })).unwrap();
        assert_eq!(
            numeric,
            L2TxAttributes {
                integrator_taker_fee: Some(10),
                skip_nonce: Some(1),
                ..L2TxAttributes::default()
            }
        );

        let named: L2TxAttributes = serde_json::from_value(json!({
            "IntegratorAccountIndex": 11,
            "IntegratorMakerFee": 9,
            "SkipNonce": 1
        }))
        .unwrap();
        assert_eq!(
            named,
            L2TxAttributes {
                integrator_account_index: Some(11),
                integrator_maker_fee: Some(9),
                skip_nonce: Some(1),
                ..L2TxAttributes::default()
            }
        );

        let zero_normalized: L2TxAttributes =
            serde_json::from_value(json!({ "1": 0, "2": 0, "3": 0, "4": 1 })).unwrap();
        assert_eq!(
            zero_normalized,
            L2TxAttributes {
                skip_nonce: Some(1),
                ..L2TxAttributes::default()
            }
        );
    }

    #[test]
    fn validates_skip_nonce_and_integrator_fee_rules() {
        let invalid_skip_nonce = L2TxAttributes {
            skip_nonce: Some(2),
            ..L2TxAttributes::default()
        };
        assert!(matches!(
            invalid_skip_nonce.validate(),
            Err(SdkError::NonceSkipAttributeInvalid)
        ));

        let fee_without_integrator = L2TxAttributes {
            integrator_taker_fee: Some(1),
            ..L2TxAttributes::default()
        };
        assert!(matches!(
            fee_without_integrator.validate(),
            Err(SdkError::IntegratorAccountIndexRequiredForNonZeroFees)
        ));

        let valid = L2TxAttributes {
            integrator_account_index: Some(12),
            integrator_taker_fee: Some(1),
            integrator_maker_fee: Some(2),
            skip_nonce: Some(1),
        };
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn zero_valued_attributes_are_treated_as_empty() {
        let attributes = L2TxAttributes {
            integrator_account_index: Some(0),
            integrator_taker_fee: Some(0),
            integrator_maker_fee: Some(0),
            skip_nonce: Some(0),
        };

        assert!(attributes.is_empty());
        assert_eq!(serde_json::to_value(&attributes).unwrap(), json!({}));
    }
}
