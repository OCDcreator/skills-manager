import { useState, useEffect } from "react";
import { X, FileText, Folder } from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { getSkillDocument, type ManagedSkill, type SkillDocument } from "../lib/tauri";

interface Props {
  skill: ManagedSkill | null;
  onClose: () => void;
}

export function SkillDetailPanel({ skill, onClose }: Props) {
  const [doc, setDoc] = useState<SkillDocument | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!skill) {
      setDoc(null);
      return;
    }
    setLoading(true);
    getSkillDocument(skill.id)
      .then(setDoc)
      .catch(() => setDoc(null))
      .finally(() => setLoading(false));
  }, [skill]);

  if (!skill) return null;

  return (
    <div className="fixed inset-0 z-50 flex">
      <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" onClick={onClose} />
      <div className="relative ml-auto w-full max-w-2xl bg-[#0F0F0F] border-l border-[#1C1C1C] h-full flex flex-col shadow-2xl animate-in slide-in-from-right duration-300">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-[#1C1C1C]">
          <div>
            <h2 className="text-lg font-semibold text-white">{skill.name}</h2>
            {skill.description && (
              <p className="text-sm text-zinc-500 mt-1">{skill.description}</p>
            )}
          </div>
          <button onClick={onClose} className="text-zinc-500 hover:text-zinc-300 p-1.5 rounded-md hover:bg-[#1A1A1A]">
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Meta */}
        <div className="px-6 py-4 border-b border-[#1C1C1C] flex items-center gap-4 text-xs text-zinc-500">
          <div className="flex items-center gap-1.5">
            <FileText className="w-3.5 h-3.5" />
            {doc?.filename || "—"}
          </div>
          <div className="flex items-center gap-1.5">
            <Folder className="w-3.5 h-3.5" />
            <span className="font-mono truncate max-w-[300px]">{skill.central_path}</span>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 scrollbar-hide">
          {loading ? (
            <div className="text-zinc-500 text-center mt-12">加载中...</div>
          ) : doc ? (
            <article className="prose prose-invert prose-sm max-w-none prose-headings:text-zinc-200 prose-p:text-zinc-400 prose-a:text-indigo-400 prose-code:text-indigo-300 prose-code:bg-[#1C1C1C] prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-pre:bg-[#0A0A0A] prose-pre:border prose-pre:border-[#2A2A2A]">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {doc.content}
              </ReactMarkdown>
            </article>
          ) : (
            <div className="text-zinc-500 text-center mt-12">没有找到文档文件</div>
          )}
        </div>
      </div>
    </div>
  );
}
