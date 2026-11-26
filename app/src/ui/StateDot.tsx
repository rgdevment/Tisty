/// On or off by shape as well as colour: a filled green dot, or a hollow grey ring. Its words go on the row it marks.
export default function StateDot({
  on,
  className = "",
}: Readonly<{ on: boolean; className?: string }>) {
  const shape = on ? "bg-hue-green" : "border border-faint";
  return (
    <span
      aria-hidden="true"
      data-on={on}
      className={`inline-block h-2 w-2 shrink-0 rounded-full ${shape} ${className}`}
    />
  );
}
