import { useState, useEffect } from "react";
import {
  Folder,
  RefreshCw,
  CheckCircle2,
  Circle,
  Globe,
  Link as LinkIcon,
  Copy,
  Settings2,
  Github,
  MessageSquare,
  Loader2,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { cn } from "../utils";
import { useApp } from "../context/AppContext";
import * as api from "../lib/tauri";

export function Settings() {
  const { t, i18n } = useTranslation();
  const { tools, scenarios, refreshTools } = useApp();
  const [syncMode, setSyncMode] = useState("symlink");
  const [defaultScenario, setDefaultScenario] = useState("");
  const [refreshing, setRefreshing] = useState(false);

  useEffect(() => {
    api.getSettings("sync_mode").then((v) => {
      if (v) setSyncMode(v);
    });
    api.getSettings("default_scenario").then((v) => {
      if (v) setDefaultScenario(v);
    });
  }, []);

  const handleRefresh = async () => {
    setRefreshing(true);
    await refreshTools();
    setRefreshing(false);
    toast.success(t("common.success"));
  };

  const handleSyncModeChange = async (mode: string) => {
    setSyncMode(mode);
    await api.setSettings("sync_mode", mode);
  };

  const handleDefaultScenarioChange = async (id: string) => {
    setDefaultScenario(id);
    await api.setSettings("default_scenario", id);
  };

  const handleLanguageChange = (lng: string) => {
    i18n.changeLanguage(lng);
    api.setSettings("language", lng);
  };

  return (
    <div className="max-w-[1000px] mx-auto h-full flex flex-col animate-in fade-in duration-500 pb-12">
      <div className="mb-8 border-b border-[#1C1C1C] pb-6">
        <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-3">
          <Settings2 className="w-6 h-6 text-indigo-400" />
          {t("settings.title")}
        </h1>
      </div>

      <div className="space-y-10">
        <section>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-zinc-200">
              {t("settings.supportedAgents")} ({tools.filter((t) => t.installed).length}/{tools.length})
            </h2>
            <button
              onClick={handleRefresh}
              disabled={refreshing}
              className="flex items-center gap-2 text-sm text-indigo-400 hover:text-indigo-300 transition-colors font-medium"
            >
              {refreshing ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <RefreshCw className="w-4 h-4" />
              )}
              {t("settings.refresh")}
            </button>
          </div>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
            {tools.map((agent, i) => (
              <div
                key={i}
                className={cn(
                  "flex items-center justify-between p-4 rounded-xl border transition-colors",
                  agent.installed
                    ? "bg-[#121212] border-[#2A2A2A] hover:border-[#3A3A3A]"
                    : "bg-[#0A0A0A] border-[#1C1C1C] opacity-60 grayscale"
                )}
              >
                <div className="flex items-center gap-3">
                  {agent.installed ? (
                    <CheckCircle2 className="w-5 h-5 text-emerald-500" />
                  ) : (
                    <Circle className="w-5 h-5 text-zinc-600" />
                  )}
                  <div>
                    <h3
                      className={cn(
                        "text-sm font-medium",
                        agent.installed ? "text-zinc-200" : "text-zinc-500"
                      )}
                    >
                      {agent.display_name}
                    </h3>
                    <p className="text-[10px] text-zinc-600 truncate max-w-[120px]" title={agent.skills_dir}>
                      {agent.installed ? agent.skills_dir.replace(/\/Users\/[^/]+/, "~") : t("settings.notInstalled")}
                    </p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </section>

        <section>
          <h2 className="text-lg font-semibold text-zinc-200 mb-4">{t("settings.globalConfig")}</h2>
          <div className="bg-[#121212] border border-[#2A2A2A] rounded-xl overflow-hidden divide-y divide-[#1C1C1C]">
            <div className="p-5 flex items-start justify-between">
              <div>
                <h3 className="text-zinc-200 font-medium mb-1">{t("settings.repoPath")}</h3>
                <p className="text-zinc-500 text-sm mb-3">{t("settings.repoPathDesc")}</p>
                <div className="flex items-center gap-2 bg-[#0A0A0A] border border-[#2A2A2A] rounded-lg px-3 py-1.5 w-fit">
                  <Folder className="w-4 h-4 text-zinc-500" />
                  <span className="text-sm font-mono text-zinc-300">~/.skills-manager/</span>
                </div>
              </div>
            </div>

            <div className="p-5 flex items-center justify-between">
              <div>
                <h3 className="text-zinc-200 font-medium mb-1">{t("settings.syncMode")}</h3>
                <p className="text-zinc-500 text-sm">{t("settings.syncModeDesc")}</p>
              </div>
              <div className="flex bg-[#0A0A0A] border border-[#2A2A2A] rounded-lg p-1">
                <button
                  onClick={() => handleSyncModeChange("symlink")}
                  className={cn(
                    "flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-colors",
                    syncMode === "symlink"
                      ? "bg-[#252528] text-zinc-200 shadow-sm"
                      : "text-zinc-500 hover:text-zinc-300"
                  )}
                >
                  <LinkIcon className="w-4 h-4" /> {t("settings.symlink")}
                </button>
                <button
                  onClick={() => handleSyncModeChange("copy")}
                  className={cn(
                    "flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-colors",
                    syncMode === "copy"
                      ? "bg-[#252528] text-zinc-200 shadow-sm"
                      : "text-zinc-500 hover:text-zinc-300"
                  )}
                >
                  <Copy className="w-4 h-4" /> {t("settings.copy")}
                </button>
              </div>
            </div>

            <div className="p-5 flex items-center justify-between">
              <div>
                <h3 className="text-zinc-200 font-medium mb-1">{t("settings.defaultScenario")}</h3>
                <p className="text-zinc-500 text-sm">{t("settings.defaultScenarioDesc")}</p>
              </div>
              <select
                value={defaultScenario}
                onChange={(e) => handleDefaultScenarioChange(e.target.value)}
                className="bg-[#0A0A0A] border border-[#2A2A2A] rounded-lg px-3 py-1.5 text-sm text-zinc-200 focus:outline-none focus:border-indigo-500/50"
              >
                <option value="">—</option>
                {scenarios.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.name}
                  </option>
                ))}
              </select>
            </div>

            <div className="p-5 flex items-center justify-between">
              <div>
                <h3 className="text-zinc-200 font-medium mb-1">{t("settings.language")}</h3>
              </div>
              <div className="flex items-center gap-2">
                <Globe className="w-4 h-4 text-zinc-500" />
                <select
                  value={i18n.language}
                  onChange={(e) => handleLanguageChange(e.target.value)}
                  className="bg-[#0A0A0A] border border-[#2A2A2A] rounded-lg px-3 py-1.5 text-sm text-zinc-200 focus:outline-none focus:border-indigo-500/50"
                >
                  <option value="zh">简体中文 (zh-CN)</option>
                  <option value="en">English (en-US)</option>
                </select>
              </div>
            </div>
          </div>
        </section>

        <section>
          <div className="bg-[#121212] border border-[#2A2A2A] rounded-xl p-6 flex flex-col items-center justify-center text-center">
            <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-lg shadow-indigo-500/20 mb-4">
              <Settings2 className="w-6 h-6 text-white" />
            </div>
            <h3 className="text-lg font-bold text-white mb-1">{t("settings.version")}</h3>
            <p className="text-zinc-500 text-sm mb-6">{t("settings.tagline")}</p>
            <div className="flex gap-4">
              <button className="flex items-center gap-2 px-4 py-2 rounded-lg bg-[#1C1C1C] hover:bg-[#252528] text-zinc-300 text-sm font-medium transition-colors border border-[#2A2A2A]">
                <Github className="w-4 h-4" /> GitHub
              </button>
              <button className="flex items-center gap-2 px-4 py-2 rounded-lg bg-[#1C1C1C] hover:bg-[#252528] text-zinc-300 text-sm font-medium transition-colors border border-[#2A2A2A]">
                <MessageSquare className="w-4 h-4" /> Feedback
              </button>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
