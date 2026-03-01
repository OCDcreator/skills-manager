import { useState, useEffect, useCallback } from "react";
import {
  Search,
  LayoutGrid,
  List,
  FileText,
  CheckCircle2,
  Circle,
  Plus,
  Github,
  HardDrive,
  Globe,
  Trash2,
  Layers,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { cn } from "../utils";
import { useApp } from "../context/AppContext";
import { SkillDetailPanel } from "../components/SkillDetailPanel";
import * as api from "../lib/tauri";
import type { ManagedSkill } from "../lib/tauri";

export function MySkills() {
  const { t } = useTranslation();
  const { activeScenario, tools, refreshScenarios } = useApp();
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
  const [filterMode, setFilterMode] = useState<"all" | "enabled" | "available">("all");
  const [skills, setSkills] = useState<ManagedSkill[]>([]);
  const [search, setSearch] = useState("");
  const [selectedSkill, setSelectedSkill] = useState<ManagedSkill | null>(null);

  const installedTools = tools.filter((tool) => tool.installed);

  const loadSkills = useCallback(async () => {
    try {
      const managedSkills = await api.getManagedSkills();
      setSkills(managedSkills);
    } catch (error) {
      console.error(error);
    }
  }, []);

  useEffect(() => {
    loadSkills();
  }, [loadSkills]);

  const enabledCount = activeScenario
    ? skills.filter((skill) => skill.scenario_ids.includes(activeScenario.id)).length
    : 0;

  const filtered = skills.filter((skill) => {
    const matchesSearch =
      skill.name.toLowerCase().includes(search.toLowerCase()) ||
      (skill.description || "").toLowerCase().includes(search.toLowerCase());

    if (!matchesSearch) {
      return false;
    }

    if (!activeScenario) {
      return true;
    }

    const enabledInScenario = skill.scenario_ids.includes(activeScenario.id);

    if (filterMode === "enabled") {
      return enabledInScenario;
    }

    if (filterMode === "available") {
      return !enabledInScenario;
    }

    return true;
  });

  const handleSync = async (skill: ManagedSkill) => {
    for (const tool of installedTools) {
      if (!skill.targets.find((target) => target.tool === tool.key)) {
        await api.syncSkillToTool(skill.id, tool.key);
      }
    }
    toast.success(`${skill.name} ${t("mySkills.synced")}`);
    await loadSkills();
  };

  const handleUnsync = async (skill: ManagedSkill) => {
    for (const target of skill.targets) {
      await api.unsyncSkillFromTool(skill.id, target.tool);
    }
    toast.success(`${skill.name} ${t("mySkills.unsync")}`);
    await loadSkills();
  };

  const handleRemove = async (skill: ManagedSkill) => {
    if (!activeScenario || !skill.scenario_ids.includes(activeScenario.id)) return;

    await api.removeSkillFromScenario(skill.id, activeScenario.id);
    toast.success(`${skill.name} ${t("mySkills.remove")}`);
    await Promise.all([loadSkills(), refreshScenarios()]);
  };

  const handleToggleScenario = async (skill: ManagedSkill) => {
    if (!activeScenario) return;

    const enabledInScenario = skill.scenario_ids.includes(activeScenario.id);
    if (enabledInScenario) {
      await api.removeSkillFromScenario(skill.id, activeScenario.id);
      toast.success(`${skill.name} ${t("mySkills.disabledInScenario")}`);
    } else {
      await api.addSkillToScenario(skill.id, activeScenario.id);
      toast.success(`${skill.name} ${t("mySkills.enabledInScenario")}`);
    }

    await Promise.all([loadSkills(), refreshScenarios()]);
  };

  const sourceIcon = (type: string) => {
    switch (type) {
      case "git":
      case "skillssh":
        return <Github className="w-3.5 h-3.5" />;
      case "local":
      case "import":
        return <HardDrive className="w-3.5 h-3.5" />;
      default:
        return <Globe className="w-3.5 h-3.5" />;
    }
  };

  return (
    <div className="mx-auto flex h-full max-w-[1200px] flex-col animate-in fade-in duration-500">
      <div className="mb-8 pr-2">
        <h1 className="flex items-center gap-2 text-2xl font-bold tracking-tight text-white">
          {t("mySkills.title")}
          <span className="rounded-full border border-[#2A2A2A] bg-[#1C1C1C] px-2.5 py-1 text-sm font-medium text-zinc-400">
            {skills.length}
          </span>
        </h1>
        <p className="mt-2 text-sm text-zinc-500">
          {activeScenario
            ? t("mySkills.subtitle", { scenario: activeScenario.name, count: enabledCount })
            : t("mySkills.noScenario")}
        </p>
      </div>

      <div className="mb-6 flex items-center justify-between gap-4">
        <div className="flex flex-1 gap-3">
          <div className="relative max-w-sm w-full">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-zinc-500" />
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder={t("mySkills.searchPlaceholder")}
              className="w-full rounded-lg border border-[#2A2A2A] bg-[#121212] py-2 pl-9 pr-4 text-sm font-medium text-zinc-200 placeholder-zinc-500 transition-all focus:border-indigo-500/50 focus:outline-none focus:ring-1 focus:ring-indigo-500/50"
            />
          </div>

          <div className="flex rounded-lg border border-[#2A2A2A] bg-[#121212] p-1">
            {(["all", "enabled", "available"] as const).map((mode) => (
              <button
                key={mode}
                onClick={() => setFilterMode(mode)}
                className={cn(
                  "rounded-md px-3 py-1.5 text-sm font-medium transition-colors outline-none",
                  filterMode === mode
                    ? "bg-[#252528] text-zinc-200 shadow-sm"
                    : "text-zinc-500 hover:text-zinc-300"
                )}
              >
                {t(`mySkills.filters.${mode}`)}
              </button>
            ))}
          </div>
        </div>

        <div className="flex rounded-lg border border-[#2A2A2A] bg-[#121212] p-1">
          <button
            onClick={() => setViewMode("grid")}
            className={cn(
              "rounded-md p-1.5 transition-colors outline-none",
              viewMode === "grid"
                ? "bg-[#252528] text-zinc-200 shadow-sm"
                : "text-zinc-500 hover:text-zinc-300"
            )}
          >
            <LayoutGrid className="h-4 w-4" />
          </button>
          <button
            onClick={() => setViewMode("list")}
            className={cn(
              "rounded-md p-1.5 transition-colors outline-none",
              viewMode === "list"
                ? "bg-[#252528] text-zinc-200 shadow-sm"
                : "text-zinc-500 hover:text-zinc-300"
            )}
          >
            <List className="h-4 w-4" />
          </button>
        </div>
      </div>

      {filtered.length === 0 ? (
        <div className="flex flex-1 flex-col items-center justify-center pb-20 text-center">
          <Layers className="mb-4 h-12 w-12 text-zinc-700" />
          <h3 className="mb-2 text-lg font-semibold text-zinc-400">{t("mySkills.noSkills")}</h3>
          <p className="text-sm text-zinc-600">
            {skills.length === 0 ? t("mySkills.addFirst") : t("mySkills.noMatch")}
          </p>
        </div>
      ) : (
        <div
          className={cn(
            "pb-12",
            viewMode === "grid"
              ? "grid grid-cols-1 gap-5 md:grid-cols-2 lg:grid-cols-3"
              : "flex flex-col gap-3"
          )}
        >
          {filtered.map((skill) => {
            const isSynced = skill.targets.length > 0;
            const enabledInScenario = activeScenario
              ? skill.scenario_ids.includes(activeScenario.id)
              : false;

            return (
              <div
                key={skill.id}
                className={cn(
                  "group flex overflow-hidden rounded-xl border border-[#2A2A2A] bg-[#121212] transition-colors hover:border-[#3A3A3A]",
                  viewMode === "grid" ? "h-[250px] flex-col" : "items-center px-6 py-4"
                )}
              >
                <div className={cn("flex-1", viewMode === "grid" ? "p-5" : "flex items-center gap-6")}>
                  <div className="mb-2 flex items-start justify-between">
                    <div className="flex items-center gap-2">
                      <h3 className="text-base font-semibold text-zinc-200">{skill.name}</h3>
                      {isSynced ? (
                        <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                      ) : (
                        <Circle className="h-4 w-4 text-zinc-600" />
                      )}
                    </div>
                    {viewMode === "grid" && (
                      <button
                        onClick={() => handleRemove(skill)}
                        disabled={!enabledInScenario}
                        className={cn(
                          "focus:outline-none",
                          enabledInScenario
                            ? "text-zinc-500 hover:text-red-400"
                            : "cursor-not-allowed text-zinc-700"
                        )}
                        title={t("mySkills.remove")}
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    )}
                  </div>

                  <p className={cn("text-sm text-zinc-500 line-clamp-2", viewMode === "list" && "mb-0 flex-1")}>
                    {skill.description || "—"}
                  </p>

                  <div className="mt-3 flex flex-wrap gap-2">
                    <span
                      className={cn(
                        "inline-flex items-center rounded-full border px-2 py-1 text-[11px] font-medium",
                        enabledInScenario
                          ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-300"
                          : "border-[#2A2A2A] bg-[#161616] text-zinc-500"
                      )}
                    >
                      {enabledInScenario ? t("mySkills.inScenario") : t("mySkills.notInScenario")}
                    </span>
                    <span className="inline-flex items-center gap-1.5 rounded-full border border-[#2A2A2A] bg-[#161616] px-2 py-1 text-[11px] font-medium text-zinc-500">
                      {sourceIcon(skill.source_type)}
                      {skill.source_type}
                    </span>
                  </div>

                  {viewMode === "list" && (
                    <div className="flex w-[360px] items-center justify-end gap-4">
                      <button
                        onClick={() => handleToggleScenario(skill)}
                        disabled={!activeScenario}
                        className={cn(
                          "rounded-md border px-4 py-1.5 text-sm font-medium transition-colors",
                          enabledInScenario
                            ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-300 hover:bg-emerald-500/15"
                            : "border-[#2A2A2A] bg-[#1C1C1C] text-zinc-300 hover:bg-[#252528]"
                        )}
                      >
                        {enabledInScenario
                          ? t("mySkills.disableForScenario")
                          : t("mySkills.enableForScenario")}
                      </button>
                      <button
                        onClick={() => (isSynced ? handleUnsync(skill) : handleSync(skill))}
                        className={cn(
                          "rounded-md border px-4 py-1.5 text-sm font-medium transition-colors",
                          isSynced
                            ? "border-[#2A2A2A] bg-[#1C1C1C] text-zinc-300 hover:border-red-500/30 hover:bg-[#252528] hover:text-red-400"
                            : "border-indigo-500/30 bg-indigo-600/10 text-indigo-400 hover:bg-indigo-600/20"
                        )}
                      >
                        {isSynced ? t("mySkills.unsync") : t("mySkills.sync")}
                      </button>
                    </div>
                  )}
                </div>

                {viewMode === "grid" && (
                  <div className="flex items-center justify-between border-t border-[#1C1C1C] bg-[#0A0A0A]/50 px-5 py-3.5">
                    <div className="flex items-center gap-3">
                      {skill.targets.length > 0 && (
                        <div className="flex -space-x-1.5">
                          {skill.targets.slice(0, 3).map((target, index) => (
                            <div
                              key={index}
                              className="flex h-5 w-5 items-center justify-center rounded-full border border-[#2A2A2A] bg-[#1C1C1C] text-[8px] font-bold text-zinc-400"
                              title={target.tool}
                            >
                              {target.tool[0].toUpperCase()}
                            </div>
                          ))}
                        </div>
                      )}
                    </div>

                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => handleToggleScenario(skill)}
                        disabled={!activeScenario}
                        className={cn(
                          "rounded-md border px-3 py-1 text-xs font-medium transition-colors outline-none",
                          enabledInScenario
                            ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-300 hover:bg-emerald-500/15"
                            : "border-[#2A2A2A] bg-[#1C1C1C] text-zinc-300 hover:bg-[#252528]"
                        )}
                        title={
                          enabledInScenario
                            ? t("mySkills.disableForScenario")
                            : t("mySkills.enableForScenario")
                        }
                      >
                        {enabledInScenario ? (
                          t("mySkills.disable")
                        ) : (
                          <span className="inline-flex items-center gap-1">
                            <Plus className="h-3 w-3" />
                            {t("mySkills.enable")}
                          </span>
                        )}
                      </button>
                      <button
                        onClick={() => setSelectedSkill(skill)}
                        className="rounded p-1.5 text-zinc-500 transition-colors hover:bg-[#1C1C1C] hover:text-zinc-300"
                        title={t("mySkills.viewDoc")}
                      >
                        <FileText className="h-4 w-4" />
                      </button>
                      <button
                        onClick={() => (isSynced ? handleUnsync(skill) : handleSync(skill))}
                        className={cn(
                          "rounded-md border px-3 py-1 text-xs font-medium transition-colors outline-none",
                          isSynced
                            ? "border-[#2A2A2A] bg-[#1C1C1C] text-zinc-300 hover:bg-[#252528]"
                            : "border-indigo-500 bg-indigo-600 text-white hover:bg-indigo-500"
                        )}
                      >
                        {isSynced ? t("mySkills.synced") : t("mySkills.sync")}
                      </button>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      <SkillDetailPanel skill={selectedSkill} onClose={() => setSelectedSkill(null)} />
    </div>
  );
}
