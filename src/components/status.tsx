// Status indicators shared by the views, composed from the shadcn/ui components.
import { AlertCircle, CheckCircle2, Info, TriangleAlert } from "lucide-react";
import type { ReactNode } from "react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

export type Tone = "good" | "warn" | "bad" | "muted" | "info";

const TONE_TEXT: Record<Tone, string> = {
  good: "text-success",
  warn: "text-warning",
  bad: "text-destructive",
  muted: "text-muted-foreground",
  info: "text-info",
};

const TONE_BADGE: Record<Tone, string> = {
  good: "border-transparent bg-success/15 text-success",
  warn: "border-transparent bg-warning/15 text-warning",
  bad: "border-transparent bg-destructive/15 text-destructive",
  muted: "border-transparent bg-muted text-muted-foreground",
  info: "border-transparent bg-info/15 text-info",
};

const TONE_ICON: Record<Tone, typeof Info> = {
  good: CheckCircle2,
  warn: TriangleAlert,
  bad: AlertCircle,
  muted: Info,
  info: Info,
};

export function StatusBadge({ tone, children }: { tone: Tone; children: ReactNode }) {
  return <Badge className={TONE_BADGE[tone]}>{children}</Badge>;
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
  const Icon = TONE_ICON[tone];
  return (
    <Alert role="status" className={cn("mb-4 bg-card", TONE_TEXT[tone])}>
      <Icon />
      <AlertTitle>{title}</AlertTitle>
      {children && <AlertDescription>{children}</AlertDescription>}
    </Alert>
  );
}
