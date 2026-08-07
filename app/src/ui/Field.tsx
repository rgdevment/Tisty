interface Props {
  icon: string;
  value: string;
  hint: string;
  onChange: (value: string) => void;
  onSubmit?: () => void;
}

export default function Field({ icon, value, hint, onChange, onSubmit }: Props) {
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit?.();
      }}
      className="flex w-full items-center gap-2.5 rounded-[9px] border border-line bg-bg px-3 py-2.5 focus-within:border-accent focus-within:ring-[3px] focus-within:ring-accent-soft"
    >
      <span className="w-4 shrink-0 text-center text-[13px] text-faint">{icon}</span>
      <input
        autoFocus
        value={value}
        placeholder={hint}
        aria-label={hint}
        onChange={(e) => onChange(e.target.value)}
        className="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-faint"
      />
    </form>
  );
}
