use bfx_proto::translation::ConditionalString as ProtoConditionalString;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct NotificationDefinition {
    pub id: String,
    pub category: NotificationCategory,
    pub in_app: Option<InAppDefinition>,
    pub email: Option<EmailDefinition>,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationCategory {
    Announcements,
    Auth,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct InAppDefinition {
    pub title: StringSet,
    pub body: StringSet,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct EmailDefinition {
    pub subject: StringSet,
    pub body: StringSet,
    #[serde(default = "yes")]
    pub include_template: bool,
    pub is_list: bool,
}
const fn yes() -> bool {
    true
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum StringSet {
    Single(String),
    Multiple(Vec<ConditionalString>),
}

impl From<StringSet> for Vec<ProtoConditionalString> {
    fn from(value: StringSet) -> Self {
        match value {
            StringSet::Single(string) => vec![ProtoConditionalString {
                r#if: None,
                value: string,
            }],
            StringSet::Multiple(conditionals) => conditionals
                .into_iter()
                .map(|conditional| ProtoConditionalString {
                    r#if: conditional.if_,
                    value: conditional.value,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConditionalString {
    #[serde(rename = "if")]
    pub if_: Option<String>,
    pub value: String,
}
