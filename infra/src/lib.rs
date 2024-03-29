use paste::paste;

use domain::models::primitives::{FixedPhoneNumber, MobilePhoneNumber, Remarks};

pub mod repositories;

macro_rules! optional_primitive {
    ($t1:ty, $t2:ty) => {
        paste! {
            pub(crate) fn [<optional_ $t1:snake:lower _primitive>](value: Option<$t2>) -> Option<$t1> {
                match value {
                    Some(value) => Some($t1::new(value).unwrap()),
                    None => None,
                }
            }

            #[allow(dead_code)]
            pub(crate) fn [<optional_ $t1:snake:lower _value>](instance: &Option<$t1>) -> Option<$t2> {
                match instance {
                    Some(instance) => Some(instance.value().to_owned()),
                    None => None,
                }
            }
        }
    };
}

optional_primitive!(FixedPhoneNumber, String);
optional_primitive!(MobilePhoneNumber, String);
optional_primitive!(Remarks, String);
