pub mod models;
pub mod repositories;

use std::borrow::Cow;

use time::macros::offset;
use time::OffsetDateTime;

/// ドメイン・エラー
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    /// 予期していないエラー
    #[error("{0}")]
    Unexpected(anyhow::Error),

    /// 検証エラー
    ///
    /// 文字列を数値に変換できない場合など、ドメイン・ルールを伴わない検証エラーを表現する。
    #[error("{0}")]
    Validation(Cow<'static, str>),

    /// ドメイン・ルールエラー
    ///
    /// ドメイン・ルールに違反したことを表現する。
    #[error("{0}")]
    DomainRule(Cow<'static, str>),

    /// リポジトリ・エラー
    ///
    /// リポジトリで発生したエラーを表現する。
    #[error("{0}")]
    Repository(anyhow::Error),
}

/// ドメイン層の結果型
pub type DomainResult<T> = Result<T, DomainError>;

/// 現在の日時を日本標準時で返す。
///
/// 世界標準時で取得した現在の日時を、+9時間オフセットした日時を返す。
///
/// # 戻り値
///
/// 日本標準時の現在日時
pub fn now_jst() -> OffsetDateTime {
    OffsetDateTime::now_utc().to_offset(offset!(+9))
}

#[cfg(test)]
mod tests {
    use time::macros::offset;
    use time::{Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, Time};

    use super::now_jst;

    const DATE_TIME_DIFF_ALLOWABLE_SECONDS: i64 = 60;

    /// 現在の日時を日本標準時で返すことを確認
    #[test]
    fn retrieve_current_date_time_at_jst() {
        let utc = OffsetDateTime::now_utc();
        let jst = now_jst();
        let allowable_diff = Duration::seconds(DATE_TIME_DIFF_ALLOWABLE_SECONDS);

        // オフセットを確認
        assert_eq!(offset!(+9), jst.offset());
        // FIXME: [検証の妥当性] 上記で取得した日時の差が1分間以内か確認
        assert!(jst - utc < allowable_diff);
        // FIXME: [検証の妥当性] オフセットを無視した日時の差が9時間と1分以内か確認
        let primitive_utc = DateTimeComponents::from(utc).primitive_date_time();
        let primitive_jst = DateTimeComponents::from(jst).primitive_date_time();
        // println!("primitive_utd: {}", primitive_utc);
        // println!("primitive_jst: {}", primitive_jst);
        assert!(primitive_jst - primitive_utc < allowable_diff + Duration::hours(9));
    }

    struct DateTimeComponents {
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    }

    impl From<OffsetDateTime> for DateTimeComponents {
        fn from(value: OffsetDateTime) -> Self {
            Self {
                year: value.year(),
                month: value.month(),
                day: value.day(),
                hour: value.hour(),
                minute: value.minute(),
                second: value.second(),
            }
        }
    }

    impl DateTimeComponents {
        fn primitive_date_time(&self) -> PrimitiveDateTime {
            let date = Date::from_calendar_date(self.year, self.month, self.day).unwrap();
            let time = Time::from_hms(self.hour, self.minute, self.second).unwrap();

            PrimitiveDateTime::new(date, time)
        }
    }
}
