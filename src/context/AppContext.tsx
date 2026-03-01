import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from "react";
import type { Scenario, ToolInfo } from "../lib/tauri";
import * as api from "../lib/tauri";

interface AppState {
  scenarios: Scenario[];
  activeScenario: Scenario | null;
  tools: ToolInfo[];
  loading: boolean;
  refreshScenarios: () => Promise<void>;
  refreshTools: () => Promise<void>;
  switchScenario: (id: string) => Promise<void>;
}

const AppContext = createContext<AppState | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [scenarios, setScenarios] = useState<Scenario[]>([]);
  const [activeScenario, setActiveScenario] = useState<Scenario | null>(null);
  const [tools, setTools] = useState<ToolInfo[]>([]);
  const [loading, setLoading] = useState(true);

  const refreshScenarios = useCallback(async () => {
    try {
      const [s, active] = await Promise.all([
        api.getScenarios(),
        api.getActiveScenario(),
      ]);
      setScenarios(s);
      setActiveScenario(active);
    } catch (e) {
      console.error("Failed to load scenarios:", e);
    }
  }, []);

  const refreshTools = useCallback(async () => {
    try {
      const t = await api.getToolStatus();
      setTools(t);
    } catch (e) {
      console.error("Failed to load tools:", e);
    }
  }, []);

  const handleSwitchScenario = useCallback(
    async (id: string) => {
      try {
        await api.switchScenario(id);
        await refreshScenarios();
      } catch (e) {
        console.error("Failed to switch scenario:", e);
      }
    },
    [refreshScenarios]
  );

  useEffect(() => {
    async function init() {
      setLoading(true);
      await Promise.all([refreshScenarios(), refreshTools()]);
      setLoading(false);
    }
    init();
  }, [refreshScenarios, refreshTools]);

  return (
    <AppContext.Provider
      value={{
        scenarios,
        activeScenario,
        tools,
        loading,
        refreshScenarios,
        refreshTools,
        switchScenario: handleSwitchScenario,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useApp must be used within AppProvider");
  return ctx;
}
