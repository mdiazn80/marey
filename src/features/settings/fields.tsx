import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";

interface FieldProps {
  id: string;
  label: string;
  hint?: string;
}

export function Field({ id, label, hint, children }: FieldProps & { children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-6 border-b border-border py-2.5 last:border-0">
      <div className="space-y-0.5">
        <Label htmlFor={id}>{label}</Label>
        {hint && <p className="text-xs text-muted-foreground">{hint}</p>}
      </div>
      {children}
    </div>
  );
}

export function SwitchField({
  checked,
  onChange,
  ...field
}: FieldProps & { checked: boolean; onChange: (checked: boolean) => void }) {
  return (
    <Field {...field}>
      <Switch id={field.id} checked={checked} onCheckedChange={onChange} />
    </Field>
  );
}

/** A rate in Hz. Input that is not a usable rate is ignored instead of being saved. */
export function RateField({
  max,
  value,
  onChange,
  ...field
}: Omit<FieldProps, "hint"> & { max: number; value: number; onChange: (rate: number) => void }) {
  const { t } = useTranslation();
  return (
    <Field {...field} hint={t("settings.rateRange", { max })}>
      <Input
        id={field.id}
        type="number"
        min={1}
        max={max}
        className="w-20"
        value={value}
        onChange={(event) => {
          const rate = Number(event.target.value);
          if (Number.isFinite(rate) && rate >= 1) onChange(Math.round(rate));
        }}
      />
    </Field>
  );
}
