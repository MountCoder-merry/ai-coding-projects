export type RuleField = "filename" | "extension" | "size" | "createdAt" | "modifiedAt" | "category";
export type RuleOperator = "equals" | "contains" | "startsWith" | "endsWith" | "greaterThan" | "lessThan" | "olderThan" | "newerThan";
export type RuleActionType = "move" | "rename";

export interface RuleCondition {
  field: RuleField;
  operator: RuleOperator;
  value: string;
}

export interface RuleAction {
  type: RuleActionType;
  destination: string;
}

export interface StructuredRule {
  id: string;
  name: string;
  conditions: RuleCondition[];
  action: RuleAction;
  enabled: boolean;
}
