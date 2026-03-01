import { Search, HelpCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useApp } from "../context/AppContext";

export function Topbar() {
  const { t } = useTranslation();
  const { tools } = useApp();
  const installed = tools.filter((t) => t.installed).length;
  const total = tools.length;

  return (
    <div className="h-[52px] border-b border-[#1C1C1C] flex items-center justify-between px-6 bg-[#0A0A0A]/80 backdrop-blur-md sticky top-0 z-20">
      <div className="relative group max-w-sm w-full">
        <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
          <Search className="h-4 w-4 text-zinc-500 group-focus-within:text-indigo-400 transition-colors" />
        </div>
        <input
          type="text"
          className="block w-full pl-9 pr-3 py-1.5 border border-[#2A2A2A] rounded-md leading-5 bg-[#121212] focus:bg-[#1A1A1E] text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/50 sm:text-sm transition-all shadow-sm"
          placeholder={t("mySkills.searchPlaceholder")}
        />
        <div className="absolute inset-y-0 right-0 pr-2 flex items-center pointer-events-none">
          <span className="text-[10px] bg-[#1C1C1C] text-zinc-500 px-1.5 py-0.5 rounded border border-[#2A2A2A]">
            ⌘K
          </span>
        </div>
      </div>

      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2 cursor-pointer group px-3 py-1 rounded-full hover:bg-[#1A1A1A] transition-colors border border-transparent hover:border-[#2A2A2A]">
          <div className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-20"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]"></span>
          </div>
          <span className="text-xs font-medium text-zinc-400 group-hover:text-zinc-200 transition-colors tracking-wide">
            {t("settings.supportedAgents")} {installed}/{total}
          </span>
        </div>

        <button className="text-zinc-500 hover:text-zinc-300 transition-colors p-1.5 rounded-md hover:bg-[#1A1A1A] outline-none">
          <HelpCircle className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
