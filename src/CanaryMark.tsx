type MarkProps = {
  singing?: boolean;
  size?: number;
  className?: string;
};

export function CanaryMark({ singing = false, size = 88, className = "" }: MarkProps) {
  return (
    <svg
      className={`canary-mark ${singing ? "is-singing" : ""} ${className}`}
      width={size}
      height={size}
      viewBox="0 0 120 120"
      fill="none"
      aria-hidden
    >
      <circle cx="60" cy="62" r="52" className="cage-glow" />
      <circle cx="60" cy="62" r="48" className="cage-ring" />
      <path
        className="bird-tail"
        d="M18 64 L8 52 L22 60 L10 72 L24 68 L14 84 L32 70 Z"
      />
      <ellipse className="bird-body" cx="52" cy="68" rx="28" ry="20" />
      <ellipse className="bird-wing" cx="46" cy="64" rx="16" ry="9" />
      <circle className="bird-head" cx="78" cy="46" r="14" />
      <path className="bird-beak" d="M90 44 L108 50 L90 56 Z" />
      <circle className="bird-eye" cx="82" cy="44" r="2.4" />
      <path className="bird-leg" d="M46 86 L44 108 M44 108 L50 107" />
      <path className="bird-leg" d="M58 86 L62 108 M62 108 L70 105" />
      {singing && (
        <g className="notes">
          <path d="M96 28 l0 14" />
          <circle cx="93" cy="42" r="3" />
          <path d="M108 18 l0 16" />
          <circle cx="105" cy="34" r="3.2" />
        </g>
      )}
    </svg>
  );
}
