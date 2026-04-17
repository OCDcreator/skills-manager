import { useEffect, useMemo, useRef } from "react";
import { Copy, Loader2, RefreshCw, Square, ChevronDown, ChevronRight, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { useThemeContext } from "../context/ThemeContext";
import type { MySkillsTerminalState } from "../lib/tauri";

interface MySkillsTerminalPanelProps {
  terminal: MySkillsTerminalState | null;
  expanded: boolean;
  idle: boolean;
  idleText: string | null;
  onExpandedChange: (expanded: boolean) => void;
  onInput: (input: string) => void;
  onResize: (cols: number, rows: number) => void;
  onInterrupt: () => void;
  onRerun: () => void;
  onClose: () => void;
}

export function MySkillsTerminalPanel({
  terminal,
  expanded,
  idle,
  idleText,
  onExpandedChange,
  onInput,
  onResize,
  onInterrupt,
  onRerun,
  onClose,
}: MySkillsTerminalPanelProps) {
  const { t } = useTranslation();
  const { resolvedTheme } = useThemeContext();
  const hostRef = useRef<HTMLDivElement | null>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const transcriptLengthRef = useRef(0);
  const sessionIdRef = useRef<string | null>(null);
  const runningRef = useRef(false);

  const terminalTheme = useMemo(
    () =>
      resolvedTheme === "dark"
        ? {
            background: "#0B0B0E",
            foreground: "#E4E4E7",
            cursor: "#34D399",
            cursorAccent: "#0B0B0E",
            selectionBackground: "rgba(52, 211, 153, 0.28)",
          }
        : {
            background: "#FBFBFC",
            foreground: "#18181B",
            cursor: "#059669",
            cursorAccent: "#FBFBFC",
            selectionBackground: "rgba(5, 150, 105, 0.2)",
          },
    [resolvedTheme]
  );

  useEffect(() => {
    runningRef.current = !!terminal?.running;
  }, [terminal?.running]);

  useEffect(() => {
    const host = hostRef.current;
    if (!host || terminalRef.current) return;

    const fitAddon = new FitAddon();
    fitAddonRef.current = fitAddon;

    const xterm = new Terminal({
      fontFamily: "Consolas, 'SFMono-Regular', Menlo, monospace",
      fontSize: 12,
      lineHeight: 1.25,
      cursorBlink: true,
      convertEol: false,
      scrollback: 20000,
      allowProposedApi: false,
      theme: terminalTheme,
    });

    const resizeToFit = () => {
      if (!fitAddonRef.current || !terminalRef.current || !hostRef.current || !expanded) return;
      fitAddonRef.current.fit();
      const cols = terminalRef.current.cols;
      const rows = terminalRef.current.rows;
      if (cols > 0 && rows > 0) {
        onResize(cols, rows);
      }
    };

    xterm.loadAddon(fitAddon);
    xterm.open(host);
    xterm.onData((data) => {
      if (runningRef.current) {
        onInput(data);
      }
    });

    terminalRef.current = xterm;
    requestAnimationFrame(resizeToFit);

    const observer = new ResizeObserver(() => {
      requestAnimationFrame(resizeToFit);
    });
    observer.observe(host);

    return () => {
      observer.disconnect();
      xterm.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
    };
  }, [expanded, onInput, onResize, terminalTheme]);

  useEffect(() => {
    if (terminalRef.current) {
      terminalRef.current.options.theme = terminalTheme;
    }
  }, [terminalTheme]);

  useEffect(() => {
    const xterm = terminalRef.current;
    if (!xterm || !terminal) return;

    const nextSessionId = terminal.session.session_id;
    if (sessionIdRef.current !== nextSessionId) {
      sessionIdRef.current = nextSessionId;
      transcriptLengthRef.current = 0;
      xterm.reset();
    }

    const transcript = terminal.transcript ?? "";
    if (transcriptLengthRef.current > transcript.length) {
      xterm.reset();
      xterm.write(transcript);
      transcriptLengthRef.current = transcript.length;
      return;
    }

    const delta = transcript.slice(transcriptLengthRef.current);
    if (delta) {
      xterm.write(delta);
      transcriptLengthRef.current = transcript.length;
    }
  }, [terminal]);

  useEffect(() => {
    if (!expanded || !fitAddonRef.current || !terminalRef.current) return;
    requestAnimationFrame(() => {
      fitAddonRef.current?.fit();
      if (terminalRef.current) {
        onResize(terminalRef.current.cols, terminalRef.current.rows);
      }
    });
  }, [expanded, terminal?.session.session_id, onResize]);

  if (!terminal) return null;

  const statusLabel = terminal.running
    ? t("mySkills.myRepo.terminalRunning")
    : terminal.error
      ? t("mySkills.myRepo.terminalExitedError")
      : t("mySkills.myRepo.terminalExited");

  const handleCopyCommand = async () => {
    await navigator.clipboard.writeText(terminal.session.command);
  };

  return (
    <div className="mt-3 rounded-xl border border-border-subtle bg-surface">
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-border-subtle px-3 py-2">
        <div className="min-w-0">
          <button
            type="button"
            onClick={() => onExpandedChange(!expanded)}
            className="inline-flex items-center gap-2 text-left text-[12px] font-medium text-secondary"
          >
            {expanded ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
            {t("mySkills.myRepo.terminalTitle")}
          </button>
          <div className="mt-1 flex flex-wrap items-center gap-2 text-[11px] text-muted">
            <span>{statusLabel}</span>
            <span className="truncate" title={terminal.session.command_preview}>
              {terminal.session.command_preview}
            </span>
            {idle && idleText ? (
              <span className="rounded-full bg-amber-500/10 px-2 py-0.5 text-amber-600 dark:text-amber-300">
                {idleText}
              </span>
            ) : null}
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {terminal.running ? (
            <span className="inline-flex items-center gap-1 rounded-full bg-accent/10 px-2 py-1 text-[11px] text-accent">
              <Loader2 className="h-3 w-3 animate-spin" />
              {t("mySkills.myRepo.linkImportRunning")}
            </span>
          ) : null}
          <button type="button" onClick={() => void handleCopyCommand()} className="app-button-secondary px-3 py-1.5">
            <Copy className="h-3.5 w-3.5" />
            {t("mySkills.myRepo.terminalCopy")}
          </button>
          <button type="button" onClick={onRerun} className="app-button-secondary px-3 py-1.5">
            <RefreshCw className="h-3.5 w-3.5" />
            {t("mySkills.myRepo.terminalRerun")}
          </button>
          <button
            type="button"
            onClick={onInterrupt}
            disabled={!terminal.running}
            className="app-button-secondary px-3 py-1.5"
          >
            <Square className="h-3.5 w-3.5" />
            {t("mySkills.myRepo.terminalInterrupt")}
          </button>
          <button type="button" onClick={onClose} className="app-button-secondary px-3 py-1.5">
            <X className="h-3.5 w-3.5" />
            {t("mySkills.myRepo.terminalClose")}
          </button>
        </div>
      </div>

      {expanded ? (
        <div className="px-3 py-3">
          <div className="rounded-lg border border-border-subtle bg-black/95 p-2 dark:bg-black" style={{ minHeight: 360 }}>
            <div ref={hostRef} className="h-[360px] overflow-hidden rounded" />
          </div>
          {terminal.error ? (
            <p className="mt-2 text-[11px] text-danger">{terminal.error}</p>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
