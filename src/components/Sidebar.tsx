import { useState } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import {
  Hammer,
  LayoutDashboard,
  Layers,
  Download,
  Settings,
  Plus,
  Hash,
  Pencil,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { cn } from "../utils";
import { useApp } from "../context/AppContext";
import { CreateScenarioDialog } from "./CreateScenarioDialog";
import * as api from "../lib/tauri";

export function Sidebar() {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const { scenarios, activeScenario, switchScenario, refreshScenarios } = useApp();
  const [showCreate, setShowCreate] = useState(false);

  const NAV_ITEMS = [
    { name: t("sidebar.dashboard"), path: "/", icon: LayoutDashboard },
    { name: t("sidebar.mySkills"), path: "/my-skills", icon: Layers },
    { name: t("sidebar.installSkills"), path: "/install", icon: Download },
  ];

  const handleSwitchScenario = async (id: string) => {
    await switchScenario(id);
    const s = scenarios.find((s) => s.id === id);
    if (location.pathname === "/settings") {
      navigate("/my-skills");
    }
    if (s) toast.success(t("scenario.switched", { name: s.name }));
  };

  const handleCreateScenario = async (name: string, description?: string) => {
    await api.createScenario(name, description);
    await refreshScenarios();
    toast.success(t("scenario.created"));
  };

  const handleContextMenu = (e: React.MouseEvent, scenarioId: string) => {
    e.preventDefault();
    const scenario = scenarios.find((s) => s.id === scenarioId);
    if (!scenario) return;

    const newName = prompt(t("common.rename"), scenario.name);
    if (newName && newName.trim() && newName !== scenario.name) {
      api
        .updateScenario(scenarioId, newName.trim(), scenario.description || undefined)
        .then(() => refreshScenarios())
        .then(() => toast.success(t("scenario.renamed")));
    }
  };

  const handleRenameClick = (event: React.MouseEvent, scenarioId: string) => {
    event.preventDefault();
    event.stopPropagation();
    handleContextMenu(event, scenarioId);
  };

  return (
    <>
      <div className="w-[240px] flex-shrink-0 bg-[#0F0F0F] border-r border-[#1C1C1C] h-full flex flex-col select-none relative z-10">
        <div className="h-10 w-full" style={{ WebkitAppRegion: "drag" } as any} />

        <div className="px-4 pb-4">
          <div className="flex items-center gap-2 mb-6 text-zinc-300 font-semibold text-sm">
            <div className="w-6 h-6 rounded-md bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-lg shadow-indigo-500/20">
              <Hammer className="w-3.5 h-3.5 text-white" />
            </div>
            {t("app.name")}
          </div>

          <div className="space-y-1">
            {NAV_ITEMS.map((item) => {
              const Icon = item.icon;
              const isActive = location.pathname === item.path;
              return (
                <Link
                  key={item.path}
                  to={item.path}
                  className={cn(
                    "flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-[13px] font-medium transition-colors outline-none",
                    isActive
                      ? "bg-[#252528] text-white"
                      : "text-zinc-400 hover:text-zinc-200 hover:bg-[#1A1A1A]"
                  )}
                >
                  <Icon className={cn("w-4 h-4", isActive ? "text-indigo-400" : "text-zinc-500")} />
                  {item.name}
                </Link>
              );
            })}
          </div>
        </div>

        <div className="px-4 py-3 flex-1 overflow-y-auto scrollbar-hide">
          <div className="text-[11px] font-semibold text-zinc-500 mb-2 px-2.5 tracking-wider">
            {t("sidebar.scenarios").toUpperCase()}
          </div>
          <div className="space-y-[2px]">
            {scenarios.map((scenario) => {
              const isActive = activeScenario?.id === scenario.id;
              return (
                <button
                  key={scenario.id}
                  onClick={() => handleSwitchScenario(scenario.id)}
                  onContextMenu={(e) => handleContextMenu(e, scenario.id)}
                  className={cn(
                    "w-full flex items-center gap-2 px-2.5 py-1.5 rounded-md text-[13px] text-left transition-colors group outline-none",
                    isActive
                      ? "bg-[#252528] text-white font-medium"
                      : "text-zinc-400 hover:text-zinc-200 hover:bg-[#1A1A1A]"
                  )}
                >
                  <Hash className="w-3.5 h-3.5 text-zinc-600 group-hover:text-zinc-400 shrink-0" />
                  <span className="flex-1 truncate">{scenario.name}</span>
                  <span
                    onClick={(event) => handleRenameClick(event, scenario.id)}
                    className="rounded p-1 text-zinc-600 opacity-0 transition hover:bg-[#1C1C1C] hover:text-zinc-300 group-hover:opacity-100"
                    title={t("common.rename")}
                  >
                    <Pencil className="h-3 w-3" />
                  </span>
                  {scenario.skill_count > 0 && (
                    <span
                      className={cn(
                        "text-[10px] px-1.5 rounded-full",
                        isActive
                          ? "bg-indigo-500/20 text-indigo-300"
                          : "bg-[#1C1C1C] text-zinc-500 group-hover:bg-[#252528]"
                      )}
                    >
                      {scenario.skill_count}
                    </span>
                  )}
                </button>
              );
            })}
          </div>

          <button
            onClick={() => setShowCreate(true)}
            className="flex items-center gap-2 px-2.5 py-1.5 mt-2 rounded-md text-[13px] text-zinc-500 hover:text-zinc-300 hover:bg-[#1A1A1A] transition-colors w-full outline-none"
          >
            <Plus className="w-3.5 h-3.5" />
            {t("sidebar.newScenario")}
          </button>
        </div>

        <div className="p-4 mt-auto">
          <Link
            to="/settings"
            className={cn(
              "flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-[13px] font-medium transition-colors outline-none",
              location.pathname === "/settings"
                ? "bg-[#252528] text-white"
                : "text-zinc-400 hover:text-zinc-200 hover:bg-[#1A1A1A]"
            )}
          >
            <Settings
              className={cn(
                "w-4 h-4",
                location.pathname === "/settings" ? "text-indigo-400" : "text-zinc-500"
              )}
            />
            {t("sidebar.settings")}
          </Link>
        </div>
      </div>

      <CreateScenarioDialog
        open={showCreate}
        onClose={() => setShowCreate(false)}
        onCreate={handleCreateScenario}
      />
    </>
  );
}
