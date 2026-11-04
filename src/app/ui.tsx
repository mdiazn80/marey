import type { ReactNode } from "react";

export function Page({
  title,
  actions,
  children,
}: {
  title: string;
  actions?: ReactNode;
  children: ReactNode;
}) {
  return (
    <div className="flex h-full flex-col overflow-y-auto p-6">
      <header className="mb-5 flex items-center justify-between gap-4">
        <h1 className="text-lg font-semibold text-white">{title}</h1>
        <div className="flex items-center gap-2">{actions}</div>
      </header>
      {children}
    </div>
  );
}

export function Card({ title, children }: { title?: string; children: ReactNode }) {
  return (
    <section className="mb-4 rounded-lg border border-white/10 bg-white/[0.03] p-4">
      {title && <h2 className="mb-3 text-sm font-medium text-white/80">{title}</h2>}
      {children}
    </section>
  );
}

export function Row({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="flex justify-between gap-6 border-b border-white/5 py-1.5 last:border-0">
      <span className="text-white/50">{label}</span>
      <span className="selectable text-right font-mono text-white/90">{value}</span>
    </div>
  );
}

type Tone = "good" | "warn" | "bad" | "muted" | "info";

const TONES: Record<Tone, string> = {
  good: "bg-emerald-500/15 text-emerald-300",
  warn: "bg-amber-500/15 text-amber-300",
  bad: "bg-red-500/15 text-red-300",
  muted: "bg-white/5 text-white/50",
  info: "bg-sky-500/15 text-sky-300",
};

export function Badge({ tone, children }: { tone: Tone; children: ReactNode }) {
  return <span className={`rounded px-2 py-0.5 text-xs ${TONES[tone]}`}>{children}</span>;
}

export function Notice({
  tone,
  title,
  children,
}: {
  tone: Tone;
  title: string;
  children?: ReactNode;
}) {
  return (
    <div role="status" className={`mb-4 rounded-lg px-4 py-3 ${TONES[tone]}`}>
      <p className="font-medium">{title}</p>
      {children && <div className="mt-1 opacity-80">{children}</div>}
    </div>
  );
}

export function Button({
  children,
  onClick,
  disabled,
}: {
  children: ReactNode;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      disabled={disabled}
      onClick={onClick}
      className="rounded border border-white/15 bg-white/5 px-3 py-1.5 text-xs text-white/90 hover:bg-white/10 disabled:opacity-40"
    >
      {children}
    </button>
  );
}
