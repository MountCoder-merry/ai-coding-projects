use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::organizer::now_timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuleField {
    Filename,
    Extension,
    Size,
    CreatedAt,
    ModifiedAt,
    Category,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuleOperator {
    Equals,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    OlderThan,
    NewerThan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleActionType {
    Move,
    Rename,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleCondition {
    pub field: RuleField,
    pub operator: RuleOperator,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleAction {
    #[serde(rename = "type")]
    pub action_type: RuleActionType,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredRule {
    pub id: String,
    pub name: String,
    pub conditions: Vec<RuleCondition>,
    pub action: RuleAction,
    pub enabled: bool,
}

pub fn validate_rule(rule: &StructuredRule) -> Result<(), AppError> {
    if rule.name.trim().is_empty() {
        return Err(AppError::InvalidRule {
            message: "Rule name is required".to_string(),
        });
    }
    if rule.conditions.is_empty() {
        return Err(AppError::InvalidRule {
            message: "At least one condition is required".to_string(),
        });
    }
    validate_destination(&rule.action.destination)?;
    for condition in &rule.conditions {
        if condition.value.trim().is_empty() {
            return Err(AppError::InvalidRule {
                message: "Condition values cannot be empty".to_string(),
            });
        }
        if matches!(
            condition.operator,
            RuleOperator::GreaterThan | RuleOperator::LessThan
        ) && !condition.value.trim().parse::<u64>().is_ok()
        {
            return Err(AppError::InvalidRule {
                message: format!(
                    "{} requires a numeric value",
                    operator_label(condition.operator)
                ),
            });
        }
        if matches!(
            condition.operator,
            RuleOperator::OlderThan | RuleOperator::NewerThan
        ) && condition.value.parse::<u64>().is_err()
        {
            return Err(AppError::InvalidRule {
                message: format!(
                    "{} requires an age in days",
                    operator_label(condition.operator)
                ),
            });
        }
    }
    Ok(())
}

fn validate_destination(destination: &str) -> Result<(), AppError> {
    let trimmed = destination.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidRule {
            message: "A destination folder is required".to_string(),
        });
    }
    let path = std::path::Path::new(trimmed);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(AppError::InvalidRule {
            message: "Rule destinations must stay inside the selected folder".to_string(),
        });
    }
    Ok(())
}

fn operator_label(operator: RuleOperator) -> &'static str {
    match operator {
        RuleOperator::Equals => "equals",
        RuleOperator::Contains => "contains",
        RuleOperator::StartsWith => "startsWith",
        RuleOperator::EndsWith => "endsWith",
        RuleOperator::GreaterThan => "greaterThan",
        RuleOperator::LessThan => "lessThan",
        RuleOperator::OlderThan => "olderThan",
        RuleOperator::NewerThan => "newerThan",
    }
}

/// A deterministic parser seam for the MVP. It recognizes two examples and
/// intentionally refuses ambiguous text instead of guessing a filesystem rule.
pub fn parse_natural_language_mock(input: &str) -> Result<StructuredRule, AppError> {
    let source = input.trim();
    let lowered = source.to_ascii_lowercase();
    if source.contains("截图") || lowered.contains("screenshot") {
        let value = if source.contains("截图") {
            "截图"
        } else {
            "Screenshot"
        };
        return Ok(StructuredRule {
            id: format!("rule-{}", now_timestamp().replace([':', '-'], "")),
            name: "Screenshots".to_string(),
            conditions: vec![RuleCondition {
                field: RuleField::Filename,
                operator: RuleOperator::Contains,
                value: value.to_string(),
            }],
            action: RuleAction {
                action_type: RuleActionType::Move,
                destination: "Screenshots".to_string(),
            },
            enabled: true,
        });
    }
    if (source.contains("视频") || lowered.contains("video"))
        && (lowered.contains("1gb") || lowered.contains("1 gb") || lowered.contains("1g"))
    {
        return Ok(StructuredRule {
            id: format!("rule-{}", now_timestamp().replace([':', '-'], "")),
            name: "Large Videos".to_string(),
            conditions: vec![
                RuleCondition {
                    field: RuleField::Category,
                    operator: RuleOperator::Equals,
                    value: "Videos".to_string(),
                },
                RuleCondition {
                    field: RuleField::Size,
                    operator: RuleOperator::GreaterThan,
                    value: "1073741824".to_string(),
                },
            ],
            action: RuleAction {
                action_type: RuleActionType::Move,
                destination: "Large Videos".to_string(),
            },
            enabled: true,
        });
    }
    Err(AppError::RuleParseFailed {
        message: "The MVP parser only recognizes screenshot and large-video examples.".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screenshot_rule() -> StructuredRule {
        StructuredRule {
            id: "test".to_string(),
            name: "Screenshots".to_string(),
            conditions: vec![RuleCondition {
                field: RuleField::Filename,
                operator: RuleOperator::Contains,
                value: "Screenshot".to_string(),
            }],
            action: RuleAction {
                action_type: RuleActionType::Move,
                destination: "Screenshots".to_string(),
            },
            enabled: true,
        }
    }

    #[test]
    fn validates_safe_rule_and_rejects_escape_destination() {
        assert!(validate_rule(&screenshot_rule()).is_ok());
        let mut invalid = screenshot_rule();
        invalid.action.destination = "../Outside".to_string();
        assert!(matches!(
            validate_rule(&invalid),
            Err(AppError::InvalidRule { .. })
        ));
    }

    #[test]
    fn parses_supported_examples_without_file_access() {
        let screenshot = parse_natural_language_mock("以后所有 Screenshot 都放到 Screenshots")
            .expect("screenshot");
        assert_eq!(screenshot.action.destination, "Screenshots");
        let large_video =
            parse_natural_language_mock("所有超过 1GB 的视频移动到 Large Videos").expect("video");
        assert_eq!(large_video.conditions.len(), 2);
        assert!(parse_natural_language_mock("整理所有文件").is_err());
    }
}
