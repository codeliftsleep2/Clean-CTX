use serde::{Deserialize, Serialize};
use std::fmt;

use super::{
    FLAG_ABSTRACT, FLAG_ASYNC, FLAG_EXPORT, FLAG_GEN, FLAG_IF, FLAG_LOOP, FLAG_PRIVATE,
    FLAG_PROTECTED, FLAG_RET, FLAG_STATIC, FLAG_THROW, FLAG_UNSAFE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeclarationModifier {
    #[serde(rename = "ASYNC")]
    Async,
    #[serde(rename = "GEN")]
    Generator,
    #[serde(rename = "EXPORT")]
    Export,
    #[serde(rename = "STATIC")]
    Static,
    #[serde(rename = "PRIVATE")]
    Private,
    #[serde(rename = "PROTECTED")]
    Protected,
    #[serde(rename = "ABSTRACT")]
    Abstract,
    #[serde(rename = "UNSAFE")]
    Unsafe,
}

impl DeclarationModifier {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Async => FLAG_ASYNC,
            Self::Generator => FLAG_GEN,
            Self::Export => FLAG_EXPORT,
            Self::Static => FLAG_STATIC,
            Self::Private => FLAG_PRIVATE,
            Self::Protected => FLAG_PROTECTED,
            Self::Abstract => FLAG_ABSTRACT,
            Self::Unsafe => FLAG_UNSAFE,
        }
    }

    pub fn from_serialized(value: &str) -> Option<Self> {
        match value {
            FLAG_ASYNC => Some(Self::Async),
            FLAG_GEN => Some(Self::Generator),
            FLAG_EXPORT => Some(Self::Export),
            FLAG_STATIC => Some(Self::Static),
            FLAG_PRIVATE => Some(Self::Private),
            FLAG_PROTECTED => Some(Self::Protected),
            FLAG_ABSTRACT => Some(Self::Abstract),
            FLAG_UNSAFE => Some(Self::Unsafe),
            _ => None,
        }
    }
}

impl fmt::Display for DeclarationModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlSummary {
    #[serde(rename = "IF")]
    Branch,
    #[serde(rename = "LOOP")]
    Loop,
    #[serde(rename = "RET")]
    Return,
    #[serde(rename = "THROW")]
    Throw,
}

impl ControlSummary {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Branch => FLAG_IF,
            Self::Loop => FLAG_LOOP,
            Self::Return => FLAG_RET,
            Self::Throw => FLAG_THROW,
        }
    }

    pub fn from_serialized(value: &str) -> Option<Self> {
        match value {
            FLAG_IF => Some(Self::Branch),
            FLAG_LOOP => Some(Self::Loop),
            FLAG_RET => Some(Self::Return),
            FLAG_THROW => Some(Self::Throw),
            _ => None,
        }
    }
}

impl fmt::Display for ControlSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Closed typed vocabulary for additive method-pattern evidence.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "k", content = "v")]
pub enum PatternFact {
    #[serde(rename = "CTOR")]
    Constructor,
    #[serde(rename = "OBSERVABLE")]
    Observable,
    #[serde(rename = "OVERRIDE")]
    Override,
    #[serde(rename = "GETTER")]
    Getter(String),
    #[serde(rename = "SETTER")]
    Setter(String),
}

impl PatternFact {
    pub fn is_serialized_kind(value: &str) -> bool {
        matches!(
            value,
            "CTOR" | "OBSERVABLE" | "OVERRIDE" | "GETTER" | "SETTER"
        )
    }

    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Constructor => "CTOR",
            Self::Observable => "OBSERVABLE",
            Self::Override => "OVERRIDE",
            Self::Getter(_) => "GETTER",
            Self::Setter(_) => "SETTER",
        }
    }

    pub fn append_serialized(&self, output: &mut Vec<String>) {
        output.push(self.kind().into());
        if let Self::Getter(property) | Self::Setter(property) = self {
            output.push(property.clone());
        }
    }

    pub fn parse_all(values: &[String]) -> Option<Vec<Self>> {
        let mut facts = Vec::new();
        let mut index = 0;
        while index < values.len() {
            let fact = match values[index].as_str() {
                "CTOR" => Self::Constructor,
                "OBSERVABLE" => Self::Observable,
                "OVERRIDE" => Self::Override,
                "GETTER" => {
                    let property = values.get(index + 1)?.clone();
                    if property.is_empty() {
                        return None;
                    }
                    index += 1;
                    Self::Getter(property)
                }
                "SETTER" => {
                    let property = values.get(index + 1)?.clone();
                    if property.is_empty() {
                        return None;
                    }
                    index += 1;
                    Self::Setter(property)
                }
                _ => return None,
            };
            facts.push(fact);
            index += 1;
        }
        (!facts.is_empty()).then_some(facts)
    }
}

impl fmt::Display for PatternFact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Getter(property) | Self::Setter(property) => {
                write!(f, "{}({property})", self.kind())
            }
            _ => f.write_str(self.kind()),
        }
    }
}
