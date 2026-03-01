import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Layers, CheckCircle2, Bot, Plus, Download } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useApp } from "../context/AppContext";
import * as api from "../lib/tauri";
import type { ManagedSkill } from "../lib/tauri";

export function Dashboard() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { activeScenario, tools } = useApp();
  const [skills, setSkills] = useState<ManagedSkill[]>([]);

  const installed = tools.filter((t) => t.installed).length;
  const total = tools.length;
  const synced = skills.filter((s) => s.targets.length > 0).length;

  useEffect(() => {
    if (activeScenario) {
      api.getSkillsForScenario(activeScenario.id).then(setSkills).catch(() => {});
    }
  }, [activeScenario]);

  return (
    <div className="max-w-5xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500 pb-12">
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-white mb-2">
          {t("dashboard.greeting")}
        </h1>
        <p className="text-zinc-400">
          {t("dashboard.currentScenario")}：
          <span className="text-indigo-400 font-medium bg-indigo-500/10 px-2 py-0.5 rounded ml-1">
            {activeScenario?.name || "—"}
          </span>{" "}
          ({t("dashboard.skillsEnabled", { count: skills.length })})
        </p>
      </div>

      <div className="grid grid-cols-3 gap-4">
        {[
          {
            title: t("dashboard.scenarioSkills"),
            value: String(skills.length),
            icon: Layers,
            color: "text-blue-400",
            bg: "from-blue-500/10 to-transparent",
          },
          {
            title: t("dashboard.synced"),
            value: String(synced),
            icon: CheckCircle2,
            color: "text-emerald-400",
            bg: "from-emerald-500/10 to-transparent",
          },
          {
            title: t("dashboard.supportedAgents"),
            value: `${installed}/${total}`,
            icon: Bot,
            color: "text-purple-400",
            bg: "from-purple-500/10 to-transparent",
          },
        ].map((stat, i) => {
          const Icon = stat.icon;
          return (
            <div
              key={i}
              className="relative overflow-hidden p-6 rounded-2xl bg-[#121212] border border-[#2A2A2A] shadow-sm transform hover:-translate-y-1 transition-transform group"
            >
              <div
                className={`absolute top-0 right-0 w-32 h-32 bg-gradient-to-bl ${stat.bg} -mr-8 -mt-8 rounded-full opacity-50 transition-opacity group-hover:opacity-100`}
              />
              <div className="relative z-10 flex items-start justify-between">
                <div>
                  <p className="text-zinc-500 text-sm font-medium mb-1">{stat.title}</p>
                  <h3 className="text-3xl font-semibold text-zinc-100">{stat.value}</h3>
                </div>
                <div className={`p-2 bg-[#1C1C1C] rounded-lg ${stat.color} border border-[#2A2A2A]`}>
                  <Icon className="w-5 h-5" />
                </div>
              </div>
            </div>
          );
        })}
      </div>

      <div className="grid grid-cols-2 gap-4">
        <button
          onClick={() => navigate("/install?tab=local")}
          className="flex items-center justify-center gap-3 p-4 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors shadow-[0_0_20px_rgba(79,70,229,0.2)] hover:shadow-[0_0_30px_rgba(79,70,229,0.4)] border border-indigo-500 group outline-none"
        >
          <Download className="w-5 h-5 group-hover:-translate-y-0.5 transition-transform" />
          {t("dashboard.scanImport")}
        </button>
        <button
          onClick={() => navigate("/install")}
          className="flex items-center justify-center gap-3 p-4 rounded-xl bg-[#1C1C1C] hover:bg-[#252528] text-white font-medium transition-colors border border-[#2A2A2A] group outline-none"
        >
          <Plus className="w-5 h-5 group-hover:rotate-90 transition-transform" />
          {t("dashboard.installNew")}
        </button>
      </div>

      {skills.length > 0 && (
        <div>
          <h2 className="text-base font-semibold text-zinc-200 mb-4 flex items-center gap-2">
            {t("dashboard.recentActivity")}
          </h2>
          <div className="bg-[#121212] border border-[#2A2A2A] rounded-xl overflow-hidden divide-y divide-[#1C1C1C]">
            {skills.slice(0, 5).map((skill) => (
              <div
                key={skill.id}
                className="flex items-center justify-between p-4 hover:bg-[#151515] transition-colors"
              >
                <div className="flex items-center gap-4">
                  <div className="w-8 h-8 rounded-full flex items-center justify-center text-xs font-semibold bg-indigo-500/10 text-indigo-400">
                    {skill.name.charAt(0)}
                  </div>
                  <div>
                    <h4 className="text-zinc-200 text-sm font-medium mb-0.5 flex items-center gap-2">
                      {skill.name}
                      <span className="text-[10px] px-1.5 py-0.5 rounded bg-[#1C1C1C] text-zinc-400 border border-[#2A2A2A] font-normal">
                        {skill.source_type}
                      </span>
                    </h4>
                    <p className="text-zinc-500 text-xs">
                      {skill.targets.length > 0
                        ? `${t("dashboard.synced")} → ${skill.targets.map((t) => t.tool).join(", ")}`
                        : "未同步"}
                    </p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
