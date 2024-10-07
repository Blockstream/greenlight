use gl_client::pb::cln::{self, amount_or_any};

#[derive(Debug, Clone)]
enum AmountOrAnyValue {
    Any,
    Amount(u64),
}

#[derive(Debug, Clone)]
pub struct AmountOrAny {
    value: AmountOrAnyValue,
}

impl From<&str> for AmountOrAny {
    fn from(value: &str) -> Self {
        if value == "any" {
            return Self {
                value: AmountOrAnyValue::Any,
            };
        } else {
            return match value.parse::<u64>() {
                Ok(msat) => Self {
                    value: AmountOrAnyValue::Amount(msat),
                },
                Err(e) => panic!("{}", e),
            };
        }
    }
}

impl Into<cln::AmountOrAny> for AmountOrAny {
    fn into(self) -> cln::AmountOrAny {
        match self.value {
            AmountOrAnyValue::Any => cln::AmountOrAny {
                value: Some(amount_or_any::Value::Any(true)),
            },
            AmountOrAnyValue::Amount(msat) => cln::AmountOrAny {
                value: Some(amount_or_any::Value::Amount(cln::Amount { msat })),
            },
        }
    }
}
