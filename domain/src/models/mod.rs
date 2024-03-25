use std::marker::PhantomData;

use uuid::Uuid;

use macros::DomainPrimitive;

use crate::common::DomainError;

pub mod passwords;
pub mod user;

/// エンティティID
///
/// UUID v4でエンティティを識別するIDを表現する。
/// `PhantomData`でエンティティの型を識別する。
#[derive(Debug, PartialEq, Eq, Hash, DomainPrimitive)]
pub struct EntityId<T> {
    #[value_getter(ret = "val")]
    value: Uuid,
    _phantom: PhantomData<T>,
}

impl<'a, T> TryFrom<&'a str> for EntityId<T> {
    type Error = DomainError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match Uuid::parse_str(s) {
            Ok(value) => Ok(Self {
                value,
                _phantom: PhantomData,
            }),
            Err(_) => Err(DomainError::Validation(
                "could not recognize as UUID v4 format string".into(),
            )),
        }
    }
}

impl<T> Copy for EntityId<T> {}

impl<T> Clone for EntityId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Default for EntityId<T> {
    fn default() -> Self {
        Self {
            value: Uuid::new_v4(),
            _phantom: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::DomainError;
    use crate::models::EntityId;

    /// UUID v4形式の文字列からエンティティIDを構築できるか確認
    #[test]
    fn construct_entity_id_from_valid_string() {
        let expected = "27db4b5f-1ff8-4691-ba07-f54b56884241";
        let entity_id: EntityId<i32> = expected.try_into().unwrap();
        let value_str = entity_id.value.to_string();
        assert_eq!(expected, value_str);
    }

    /// UUID v4形式でない文字列からエンティティIDを構築できないことを確認
    #[test]
    fn can_not_construct_entity_id_from_invalid_string() {
        let invalid_string = "invalid uuid v4 string";
        let result: Result<EntityId<i32>, DomainError> = invalid_string.try_into();
        assert!(result.is_err());
        match result.err().unwrap() {
            DomainError::Validation(_) => {}
            _ => panic!("expected DomainError::Validation"),
        }
    }
}
