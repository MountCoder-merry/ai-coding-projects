import { create } from "zustand";
import type { AppInfo, EngineStatus, RecentActivity, ScanResult, ViewId } from "../types/app";
import type { ApplyResult, OperationTransaction, OrganizationPlan } from "../types/organization";
import type { StructuredRule } from "../types/rules";

interface AppState {
  activeView: ViewId;
  engineStatus: EngineStatus;
  appInfo: AppInfo | null;
  selectedFolder: string | null;
  scanResult: ScanResult | null;
  isScanning: boolean;
  scanError: string | null;
  organizationPlan: OrganizationPlan | null;
  isPlanning: boolean;
  planError: string | null;
  lastApply: ApplyResult | null;
  transactions: OperationTransaction[];
  rules: StructuredRule[];
  recentActivity: RecentActivity[];
  setActiveView: (view: ViewId) => void;
  setEngineStatus: (status: EngineStatus) => void;
  setAppInfo: (info: AppInfo | null) => void;
  setSelectedFolder: (folder: string | null) => void;
  setScanResult: (result: ScanResult | null) => void;
  setIsScanning: (value: boolean) => void;
  setScanError: (message: string | null) => void;
  setOrganizationPlan: (plan: OrganizationPlan | null) => void;
  setIsPlanning: (value: boolean) => void;
  setPlanError: (message: string | null) => void;
  setLastApply: (result: ApplyResult | null) => void;
  addTransaction: (transaction: OperationTransaction) => void;
  setTransactions: (transactions: OperationTransaction[]) => void;
  updateTransaction: (transaction: OperationTransaction) => void;
  setRules: (rules: StructuredRule[]) => void;
  addRule: (rule: StructuredRule) => void;
  addActivity: (activity: RecentActivity) => void;
}

export const useAppStore = create<AppState>((set) => ({
  activeView: "home",
  engineStatus: "checking",
  appInfo: null,
  selectedFolder: null,
  scanResult: null,
  isScanning: false,
  scanError: null,
  organizationPlan: null,
  isPlanning: false,
  planError: null,
  lastApply: null,
  transactions: [],
  rules: [],
  recentActivity: [
    {
      id: "welcome",
      title: "Workspace ready",
      detail: "Choose a folder to prepare your first plan",
      timestamp: "Just now",
      tone: "neutral",
    },
  ],
  setActiveView: (activeView) => set({ activeView }),
  setEngineStatus: (engineStatus) => set({ engineStatus }),
  setAppInfo: (appInfo) => set({ appInfo }),
  setSelectedFolder: (selectedFolder) => set({ selectedFolder }),
  setScanResult: (scanResult) => set({ scanResult }),
  setIsScanning: (isScanning) => set({ isScanning }),
  setScanError: (scanError) => set({ scanError }),
  setOrganizationPlan: (organizationPlan) => set({ organizationPlan }),
  setIsPlanning: (isPlanning) => set({ isPlanning }),
  setPlanError: (planError) => set({ planError }),
  setLastApply: (lastApply) => set({ lastApply }),
  addTransaction: (transaction) => set((state) => ({ transactions: [transaction, ...state.transactions].slice(0, 20) })),
  setTransactions: (transactions) => set({ transactions: transactions.slice(0, 20) }),
  updateTransaction: (transaction) => set((state) => ({ transactions: state.transactions.map((item) => item.id === transaction.id ? transaction : item) })),
  setRules: (rules) => set({ rules }),
  addRule: (rule) => set((state) => ({ rules: [rule, ...state.rules.filter((item) => item.id !== rule.id)] })),
  addActivity: (activity) =>
    set((state) => ({ recentActivity: [activity, ...state.recentActivity].slice(0, 5) })),
}));
