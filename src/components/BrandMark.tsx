export function BrandMark() {
  return (
    <svg className="brand-mark" viewBox="0 0 120 120" aria-hidden="true">
      <defs>
        <linearGradient id="brand-front" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stopColor="#ffffff" stopOpacity="0.98" />
          <stop offset="0.5" stopColor="#b9d2ff" stopOpacity="0.75" />
          <stop offset="1" stopColor="#4a7bff" stopOpacity="0.6" />
        </linearGradient>
        <linearGradient id="brand-depth" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#3b6bff" />
          <stop offset="1" stopColor="#0d1f7a" />
        </linearGradient>
      </defs>
      <g transform="translate(7 7)" fill="url(#brand-depth)">
        <rect x="18" y="12" width="20" height="96" rx="6" />
        <path d="M42 60 74 12h24L64 60l34 48H74z" />
      </g>
      <g fill="url(#brand-front)" stroke="#ffffff" strokeOpacity="0.7" strokeWidth="1.5">
        <rect x="18" y="12" width="20" height="96" rx="6" />
        <path d="M42 60 74 12h24L64 60l34 48H74z" strokeLinejoin="round" />
      </g>
    </svg>
  );
}
