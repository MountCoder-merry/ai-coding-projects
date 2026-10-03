export type OperationType = "move" | "rename";
export type OperationStatus = "pending" | "success" | "failed";

export interface FileOperation {
  id: string;
  type: OperationType;
  source: string;
  destination: string;
  reason: string;
  status: OperationStatus;
  error: string | null;
  sizeBytes: number;
}

export interface OrganizationPlanSummary {
  operationCount: number;
  totalBytes: number;
  newFolderCount: number;
}

export interface OrganizationPlan {
  id: string;
  rootPath: string;
  createdAt: string;
  operations: FileOperation[];
  summary: OrganizationPlanSummary;
}

export interface ExecutedOperation extends FileOperation {
  status: OperationStatus;
  error: string | null;
}

export interface OperationTransaction {
  id: string;
  createdAt: string;
  rootPath: string;
  operations: ExecutedOperation[];
  completedCount: number;
  failedCount: number;
  undoneAt: string | null;
}

export interface ApplyResult {
  transaction: OperationTransaction;
  completedCount: number;
  failedCount: number;
}
