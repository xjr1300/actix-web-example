use paste::paste;

use domain::models::primitives::{FixedPhoneNumber, MobilePhoneNumber, Remarks};

pub mod repositories;

macro_rules! optional_primitive {
    ($t1:ty, $t2:ty) => {
        paste! {
            pub fn [<optional_ $t1:snake:lower _primitive>](value: Option<$t2>) -> Option<$t1> {
                value.map(|v| $t1::new(v).unwrap())
            }

            #[allow(dead_code)]
            pub fn [<optional_ $t1:snake:lower _value>](instance: &Option<$t1>) -> Option<$t2> {
                instance.as_ref().map(|i| i.value().to_owned())
            }
        }
    };
}

optional_primitive!(FixedPhoneNumber, String);
optional_primitive!(MobilePhoneNumber, String);
optional_primitive!(Remarks, String);
