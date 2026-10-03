import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { AppInfo, ScanResult } from "../types/app";
import type { ApplyResult, OperationTransaction, OrganizationPlan } from "../types/organization";
import type { StructuredRule } from "../types/rules";

export async function loadAppInfo(): Promise<AppInfo | null> {
  try {
    return await invoke<AppInfo>("get_app_info");
  } catch {
    return null;
  }
}

export async function checkEngine(): Promise<"ready" | "browser" | "offline"> {
  try {
    const status = await invoke<string>("health_check");
    return status === "ready" ? "ready" : "offline";
  } catch {
    return "browser";
  }
}

export async function pickDirectory(): Promise<string | null> {
  try {
    const result = await open({
      directory: true,
      multiple: false,
      title: "Choose a folder to organize",
    });
    return typeof result === "string" ? result : null;
  } catch {
    return null;
  }
}

export async function getDesktopFolder(): Promise<string | null> {
  try {
    return await invoke<string>("get_desktop_folder");
  } catch {
    return null;
  }
}

export async function scanFolder(root: string): Promise<ScanResult> {
  return invoke<ScanResult>("scan_folder", { root });
}

export async function generatePlan(root: string, scan: ScanResult): Promise<OrganizationPlan> {
  return invoke<OrganizationPlan>("generate_plan", { root, scan });
}

export async function applyPlan(plan: OrganizationPlan): Promise<ApplyResult> {
  return invoke<ApplyResult>("apply_plan", { plan });
}

export async function undoTransaction(transaction: OperationTransaction): Promise<ApplyResult> {
  return invoke<ApplyResult>("undo_transaction", { transaction });
}

export async function loadHistory(): Promise<OperationTransaction[]> {
  return invoke<OperationTransaction[]>("load_history");
}

export async function validateRule(rule: StructuredRule): Promise<void> {
  await invoke("validate_rule", { rule });
}

export async function parseRuleMock(input: string): Promise<StructuredRule> {
  return invoke<StructuredRule>("parse_rule_mock", { input });
}

export async function loadRules(): Promise<StructuredRule[]> {
  return invoke<StructuredRule[]>("load_rules");
}

export async function saveRule(rule: StructuredRule): Promise<StructuredRule[]> {
  return invoke<StructuredRule[]>("save_rule", { rule });
}
