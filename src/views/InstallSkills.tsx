import { useState, useEffect, useCallback } from "react";
import {
  DownloadCloud,
  UploadCloud,
  Github,
  Box,
  Star,
  TrendingUp,
  Clock,
  Plus,
  FolderUp,
  Loader2,
  RefreshCw,
  FolderSearch,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { cn } from "../utils";
import { useApp } from "../context/AppContext";
import * as api from "../lib/tauri";
import type { ScanResult, SkillsShSkill } from "../lib/tauri";
import { open } from "@tauri-apps/plugin-dialog";
import { useSearchParams } from "react-router-dom";

export function InstallSkills() {
  const { t } = useTranslation();
  const { refreshScenarios } = useApp();
  const [searchParams, setSearchParams] = useSearchParams();
  const [activeTab, setActiveTab] = useState<"market" | "local" | "git">("market");
  const [marketTab, setMarketTab] = useState<"hot" | "trending" | "alltime">("hot");
  const [marketSkills, setMarketSkills] = useState<SkillsShSkill[]>([]);
  const [marketLoading, setMarketLoading] = useState(false);
  const [installing, setInstalling] = useState<string | null>(null);
  const [gitUrl, setGitUrl] = useState("");
  const [gitName, setGitName] = useState("");
  const [gitLoading, setGitLoading] = useState(false);
  const [scanResult, setScanResult] = useState<ScanResult | null>(null);
  const [scanLoading, setScanLoading] = useState(false);
  const [importingPaths, setImportingPaths] = useState<Set<string>>(new Set());
  const [importingAll, setImportingAll] = useState(false);
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set());

  useEffect(() => {
    const tab = searchParams.get("tab");
    if (tab === "market" || tab === "local" || tab === "git") {
      setActiveTab(tab);
    }
  }, [searchParams]);

  const switchTab = (tab: "market" | "local" | "git") => {
    setActiveTab(tab);
    setSearchParams({ tab });
  };

  const runScan = useCallback(async () => {
    setScanLoading(true);
    try {
      const result = await api.scanLocalSkills();
      setScanResult(result);
    } catch (e: any) {
      console.error(e);
      toast.error(e.toString());
    } finally {
      setScanLoading(false);
    }
  }, []);

  useEffect(() => {
    if (activeTab === "market") {
      setMarketLoading(true);
      api
        .fetchLeaderboard(marketTab)
        .then(setMarketSkills)
        .catch((e) => {
          console.error(e);
          toast.error(t("common.error"));
        })
        .finally(() => setMarketLoading(false));
    }
  }, [activeTab, marketTab, t]);

  useEffect(() => {
    if (activeTab === "local" && !scanResult && !scanLoading) {
      runScan();
    }
  }, [activeTab, scanLoading, scanResult, runScan]);

  const handleInstallSkillssh = async (skill: SkillsShSkill) => {
    setInstalling(skill.id);
    try {
      await api.installFromSkillssh(skill.source, skill.skill_id);
      toast.success(`${skill.name} ${t("common.success")}`);
      refreshScenarios();
    } catch (e: any) {
      toast.error(e.toString());
    } finally {
      setInstalling(null);
    }
  };

  const handleLocalInstall = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        filters: [{ name: "Skills", extensions: ["zip", "skill"] }],
      });
      if (!selected) return;
      await api.installLocal(selected as string);
      toast.success(t("common.success"));
      await refreshScenarios();
      await runScan();
    } catch (e: any) {
      toast.error(e.toString());
    }
  };

  const handleGitInstall = async () => {
    if (!gitUrl.trim()) return;
    setGitLoading(true);
    try {
      await api.installGit(gitUrl.trim(), gitName.trim() || undefined);
      toast.success(t("common.success"));
      setGitUrl("");
      setGitName("");
      refreshScenarios();
    } catch (e: any) {
      toast.error(e.toString());
    } finally {
      setGitLoading(false);
    }
  };

  const toggleGroup = (name: string) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(name)) {
        next.delete(name);
      } else {
        next.add(name);
      }
      return next;
    });
  };

  const handleImportDiscovered = async (sourcePath: string, name: string) => {
    setImportingPaths((prev) => new Set(prev).add(sourcePath));
    try {
      await api.importExistingSkill(sourcePath, name);
      toast.success(t("install.scan.importedOne", { name }));
      await refreshScenarios();
      await runScan();
    } catch (e: any) {
      toast.error(e.toString());
    } finally {
      setImportingPaths((prev) => {
        const next = new Set(prev);
        next.delete(sourcePath);
        return next;
      });
    }
  };

  const handleImportAllDiscovered = async () => {
    setImportingAll(true);
    try {
      await api.importAllDiscovered();
      toast.success(t("install.scan.importedAll"));
      await refreshScenarios();
      await runScan();
    } catch (e: any) {
      toast.error(e.toString());
    } finally {
      setImportingAll(false);
    }
  };

  return (
    <div className="max-w-[1200px] mx-auto h-full flex flex-col animate-in fade-in duration-500">
      <div className="mb-8 border-b border-[#1C1C1C]">
        <h1 className="text-2xl font-bold tracking-tight text-white mb-6">{t("install.title")}</h1>
        <div className="flex gap-8">
          {[
            { id: "market" as const, label: t("install.browseMarket"), icon: Box },
            { id: "local" as const, label: t("install.localInstall"), icon: UploadCloud },
            { id: "git" as const, label: t("install.gitInstall"), icon: Github },
          ].map((tab) => {
            const Icon = tab.icon;
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => switchTab(tab.id)}
                className={cn(
                  "pb-4 text-sm font-medium flex items-center gap-2 border-b-2 transition-colors outline-none",
                  isActive
                    ? "border-indigo-500 text-indigo-400"
                    : "border-transparent text-zinc-500 hover:text-zinc-300"
                )}
              >
                <Icon className="w-4 h-4" />
                {tab.label}
              </button>
            );
          })}
        </div>
      </div>

      {activeTab === "market" && (
        <div className="flex-1 animate-in fade-in duration-300">
          <div className="flex items-center justify-between mb-6">
            <div className="flex bg-[#121212] border border-[#2A2A2A] rounded-lg p-1">
              {[
                { id: "hot" as const, label: t("install.hot"), icon: Star },
                { id: "trending" as const, label: t("install.trending"), icon: TrendingUp },
                { id: "alltime" as const, label: t("install.all"), icon: Clock },
              ].map((tab) => {
                const Icon = tab.icon;
                const isActive = marketTab === tab.id;
                return (
                  <button
                    key={tab.id}
                    onClick={() => setMarketTab(tab.id)}
                    className={cn(
                      "flex items-center gap-1.5 px-4 py-1.5 rounded-md text-sm transition-colors outline-none",
                      isActive
                        ? "bg-[#252528] text-zinc-200 shadow-sm"
                        : "text-zinc-500 hover:text-zinc-300"
                    )}
                  >
                    <Icon className="w-3.5 h-3.5" />
                    {tab.label}
                  </button>
                );
              })}
            </div>
          </div>

          {marketLoading ? (
            <div className="flex items-center justify-center py-20">
              <Loader2 className="w-6 h-6 text-zinc-500 animate-spin" />
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5 pb-12">
              {marketSkills.map((skill) => (
                <div
                  key={skill.id}
                  className="bg-[#121212] border border-[#2A2A2A] rounded-xl p-5 hover:border-[#3A3A3A] transition-colors group flex flex-col h-[200px]"
                >
                  <div className="flex justify-between items-start mb-2">
                    <h3 className="font-semibold text-zinc-200 text-base">{skill.name || skill.skill_id}</h3>
                  </div>
                  <div className="flex items-center gap-2 mb-3">
                    <span className="text-xs text-indigo-400 font-medium bg-indigo-500/10 px-2 py-0.5 rounded">
                      @{skill.source}
                    </span>
                    <span className="text-xs text-zinc-500 flex items-center gap-1">
                      <DownloadCloud className="w-3.5 h-3.5" />{" "}
                      {skill.installs > 1000
                        ? `${(skill.installs / 1000).toFixed(0)}k`
                        : skill.installs}
                    </span>
                  </div>
                  <p className="text-sm text-zinc-500 line-clamp-2 mb-auto">{skill.skill_id}</p>
                  <div className="pt-4 flex justify-end">
                    <button
                      onClick={() => handleInstallSkillssh(skill)}
                      disabled={installing === skill.id}
                      className="flex items-center gap-2 px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium transition-colors shadow-sm w-full justify-center group-hover:shadow-[0_0_15px_rgba(79,70,229,0.2)] disabled:opacity-50"
                    >
                      {installing === skill.id ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : (
                        <Plus className="w-4 h-4" />
                      )}
                      {installing === skill.id ? t("install.installing") : t("install.oneClickInstall")}
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {activeTab === "local" && (
        <div className="flex-1 animate-in fade-in duration-300 space-y-8 pb-12">
          <div
            onClick={handleLocalInstall}
            className="w-full bg-[#121212] border-2 border-dashed border-[#2A2A2A] rounded-2xl p-16 flex flex-col items-center justify-center text-center hover:bg-[#151515] hover:border-indigo-500/50 transition-colors cursor-pointer group"
          >
            <div className="w-16 h-16 bg-[#1C1C1C] rounded-full flex items-center justify-center mb-6 group-hover:scale-110 group-hover:bg-indigo-500/10 transition-all">
              <FolderUp className="w-8 h-8 text-zinc-500 group-hover:text-indigo-400" />
            </div>
            <h3 className="text-xl font-semibold text-zinc-200 mb-2">{t("install.dragDrop")}</h3>
            <p className="text-zinc-500 mb-8 max-w-sm">{t("install.dragDropDesc")}</p>
            <button className="px-6 py-2.5 rounded-lg bg-[#252528] hover:bg-[#2A2A2E] border border-[#3A3A3A] text-zinc-200 text-sm font-medium transition-colors">
              {t("install.selectLocal")}
            </button>
          </div>

          <section className="bg-[#121212] border border-[#2A2A2A] rounded-2xl overflow-hidden">
            <div className="px-6 py-5 border-b border-[#1C1C1C] flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
              <div>
                <h2 className="text-lg font-semibold text-white">{t("install.scan.title")}</h2>
                <p className="text-sm text-zinc-500 mt-1">
                  {scanResult
                    ? t("install.scan.summary", {
                        tools: scanResult.tools_scanned,
                        skills: scanResult.skills_found,
                      })
                    : t("install.scan.initial")}
                </p>
              </div>

              <div className="flex items-center gap-3">
                <button
                  onClick={runScan}
                  disabled={scanLoading}
                  className="flex items-center gap-2 px-4 py-2 rounded-lg bg-[#1C1C1C] hover:bg-[#252528] border border-[#2A2A2A] text-zinc-200 text-sm font-medium transition-colors disabled:opacity-50"
                >
                  <RefreshCw className={cn("w-4 h-4", scanLoading && "animate-spin")} />
                  {t("install.scan.rescan")}
                </button>
                <button
                  onClick={handleImportAllDiscovered}
                  disabled={
                    scanLoading ||
                    importingAll ||
                    !scanResult ||
                    scanResult.groups.length === 0
                  }
                  className="flex items-center gap-2 px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 border border-indigo-500 text-white text-sm font-medium transition-colors disabled:opacity-50"
                >
                  {importingAll ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    <DownloadCloud className="w-4 h-4" />
                  )}
                  {t("install.scan.importAll")}
                </button>
              </div>
            </div>

            <div className="p-6">
              {scanLoading ? (
                <div className="py-16 flex items-center justify-center gap-3 text-zinc-500">
                  <Loader2 className="w-5 h-5 animate-spin" />
                  <span>{t("install.scan.scanning")}</span>
                </div>
              ) : scanResult && scanResult.groups.length === 0 ? (
                <div className="py-16 flex flex-col items-center justify-center text-center">
                  <div className="w-12 h-12 rounded-xl bg-[#1C1C1C] border border-[#2A2A2A] flex items-center justify-center mb-4">
                    <FolderSearch className="w-6 h-6 text-zinc-500" />
                  </div>
                  <h3 className="text-lg font-semibold text-zinc-200 mb-2">
                    {t("install.scan.noResults")}
                  </h3>
                  <p className="text-sm text-zinc-500">{t("install.scan.noResultsHint")}</p>
                </div>
              ) : (
                <div className="space-y-4">
                  {scanResult?.groups.map((group) => {
                    const isExpanded = expandedGroups.has(group.name);
                    return (
                      <article
                        key={group.name}
                        className="bg-[#0D0D0D] border border-[#1C1C1C] rounded-xl overflow-hidden"
                      >
                        <button
                          type="button"
                          onClick={() => toggleGroup(group.name)}
                          className="w-full px-5 py-4 flex items-center justify-between hover:bg-[#141414] transition-colors"
                        >
                          <div className="flex items-center gap-4">
                            <div className="w-10 h-10 rounded-lg bg-[#161616] border border-[#232323] flex items-center justify-center text-zinc-500">
                              {isExpanded ? (
                                <ChevronDown className="w-5 h-5" />
                              ) : (
                                <ChevronRight className="w-5 h-5" />
                              )}
                            </div>
                            <div className="text-left">
                              <div className="flex items-center gap-2">
                                <h3 className="text-sm font-semibold text-zinc-100">{group.name}</h3>
                                {group.imported && (
                                  <span className="px-2 py-0.5 rounded text-[10px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                                    {t("install.scan.imported")}
                                  </span>
                                )}
                              </div>
                              <p className="text-xs text-zinc-500 mt-1">
                                {t("install.scan.locations", { count: group.locations.length })}
                              </p>
                            </div>
                          </div>
                        </button>

                        {isExpanded && (
                          <div className="px-5 pb-5 space-y-3 border-t border-[#1C1C1C] bg-[#0A0A0A]">
                            {group.locations.map((location) => (
                              <div
                                key={location.id}
                                className="mt-4 p-4 rounded-xl bg-[#121212] border border-[#232323] flex flex-col gap-4 lg:flex-row lg:items-center"
                              >
                                <div className="min-w-0 flex-1">
                                  <span className="inline-flex px-2 py-0.5 rounded text-[11px] font-medium bg-white/5 text-zinc-400 border border-white/5 mb-2">
                                    {location.tool}
                                  </span>
                                  <code className="block text-[12px] leading-5 break-all bg-[#0A0A0A] border border-[#1C1C1C] rounded-lg px-3 py-2 text-zinc-300">
                                    {location.found_path}
                                  </code>
                                </div>

                                {group.imported ? (
                                  <span className="px-3 py-2 rounded-lg text-xs font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                                    {t("install.scan.imported")}
                                  </span>
                                ) : (
                                  <button
                                    onClick={() =>
                                      handleImportDiscovered(location.found_path, group.name)
                                    }
                                    disabled={importingPaths.has(location.found_path)}
                                    className="shrink-0 flex items-center justify-center gap-2 px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 border border-indigo-500 text-white text-sm font-medium transition-colors disabled:opacity-50"
                                  >
                                    {importingPaths.has(location.found_path) ? (
                                      <Loader2 className="w-4 h-4 animate-spin" />
                                    ) : (
                                      <DownloadCloud className="w-4 h-4" />
                                    )}
                                    {t("install.scan.importOne")}
                                  </button>
                                )}
                              </div>
                            ))}
                          </div>
                        )}
                      </article>
                    );
                  })}
                </div>
              )}
            </div>
          </section>
        </div>
      )}

      {activeTab === "git" && (
        <div className="flex-1 animate-in fade-in duration-300">
          <div className="max-w-xl bg-[#121212] border border-[#2A2A2A] rounded-xl p-8">
            <div className="mb-6 flex items-center justify-center w-12 h-12 bg-[#1C1C1C] rounded-lg border border-[#2A2A2A]">
              <Github className="w-6 h-6 text-zinc-400" />
            </div>
            <h2 className="text-lg font-semibold text-white mb-2">{t("install.gitTitle")}</h2>
            <p className="text-zinc-500 text-sm mb-6">{t("install.gitDesc")}</p>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-zinc-400 mb-1.5">
                  {t("install.repoUrl")}
                </label>
                <input
                  type="text"
                  value={gitUrl}
                  onChange={(e) => setGitUrl(e.target.value)}
                  placeholder={t("install.repoUrlPlaceholder")}
                  className="w-full bg-[#0A0A0A] border border-[#2A2A2A] rounded-lg px-4 py-2.5 text-sm text-zinc-200 focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/50 transition-all placeholder-zinc-600"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-zinc-400 mb-1.5 flex items-center gap-2">
                  {t("install.customName")}
                  <span className="text-xs text-zinc-600 font-normal">{t("install.customNameOptional")}</span>
                </label>
                <input
                  type="text"
                  value={gitName}
                  onChange={(e) => setGitName(e.target.value)}
                  placeholder={t("install.customNamePlaceholder")}
                  className="w-full bg-[#0A0A0A] border border-[#2A2A2A] rounded-lg px-4 py-2.5 text-sm text-zinc-200 focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/50 transition-all placeholder-zinc-600"
                />
              </div>
              <div className="pt-4">
                <button
                  onClick={handleGitInstall}
                  disabled={!gitUrl.trim() || gitLoading}
                  className="flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium transition-colors w-full shadow-[0_0_15px_rgba(79,70,229,0.2)] border border-indigo-500 disabled:opacity-50"
                >
                  {gitLoading ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    <DownloadCloud className="w-4 h-4" />
                  )}
                  {gitLoading ? t("install.installing") : t("install.installClone")}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
