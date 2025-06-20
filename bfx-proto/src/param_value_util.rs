use crate::{ParamValue, param_value};

impl From<param_value::ParamValue> for ParamValue {
    fn from(value: param_value::ParamValue) -> Self {
        Self {
            param_value: Some(value),
        }
    }
}
