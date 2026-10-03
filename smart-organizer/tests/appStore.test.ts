import { beforeEach, describe, expect, it } from "vitest";
import { useAppStore } from "../src/store/appStore";

describe("app workflow store", () => {
  beforeEach(() => {
    useAppStore.setState({
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
      recentActivity: [],
    });
  });

  it("keeps navigation and folder selection separate", () => {
    useAppStore.getState().setActiveView("organize");
    useAppStore.getState().setSelectedFolder("C:\\Users\\Demo\\Desktop");

    const state = useAppStore.getState();
    expect(state.activeView).toBe("organize");
    expect(state.selectedFolder).toBe("C:\\Users\\Demo\\Desktop");
  });

  it("keeps only the five most recent activities", () => {
    for (let index = 0; index < 7; index += 1) {
      useAppStore.getState().addActivity({
        id: String(index),
        title: `Activity ${index}`,
        detail: "Test",
        timestamp: "Now",
        tone: "neutral",
      });
    }

    expect(useAppStore.getState().recentActivity).toHaveLength(5);
    expect(useAppStore.getState().recentActivity[0]?.id).toBe("6");
  });

  it("stores a scan result without changing the selected folder", () => {
    useAppStore.getState().setSelectedFolder("C:\\Users\\Demo\\Desktop");
    useAppStore.getState().setScanResult({
      root: "C:\\Users\\Demo\\Desktop",
      scannedAt: "2026-10-02T00:00:00.000Z",
      items: [],
      summary: {
        totalFiles: 0,
        totalBytes: 0,
        skippedEntries: 1,
        byCategory: { Images: 0, Videos: 0, Documents: 0, Archives: 0, Audio: 0, Code: 0, Applications: 0, Other: 0 },
      },
    });

    const state = useAppStore.getState();
    expect(state.scanResult?.summary.skippedEntries).toBe(1);
    expect(state.selectedFolder).toContain("Demo");
  });
});
