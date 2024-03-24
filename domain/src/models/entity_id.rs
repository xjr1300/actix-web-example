use std::marker::PhantomData;

use uuid::Uuid;

use macros::ValueDisplay;

use crate::common::error::DomainError;

/// エンティティID
///
/// UUID v4でエンティティを識別するIDを表現する。
/// `PhantomData`でエンティティの型を識別する。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, ValueDisplay)]
pub struct EntityId<T> {
    value: Uuid,
    _phantom: PhantomData<T>,
}

impl<'a, T> TryFrom<&'a str> for EntityId<T> {
    type Error = DomainError<'a>;

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

#[cfg(test)]
mod tests {
    use crate::common::error::DomainError;
    use crate::models::entity_id::EntityId;

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
