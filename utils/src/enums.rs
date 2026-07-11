use macros::DbEnum;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

// `FormOptions` cannot be derived here: its expansion emits `impl components::FormSchema`,
// and `components` already depends on `utils`, so deriving it would create a `utils -> components`
// cycle. Consumers build Select/Radio options from the `EnumIter` derive (`ActivityKind::iter()`)
// until the schema trait lives somewhere both crates can depend on.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, DbEnum, EnumIter, Default)]
pub enum ActivityKind {
    #[default]
    #[db_enum(rename = "Phone Call")]
    #[serde(rename = "Phone Call")]
    PhoneCall,
    Email,
    #[serde(rename = "SMS")]
    #[db_enum(rename = "SMS")]
    Sms,
    WhatsApp,
    Meeting,
}
