import { useEffect, useState } from "react";
import { settings } from "../core";

export const QUIET = "beaver";

// On unless the person turned it off; a failed read keeps it away rather than guessing.
export function useBeaver(): boolean {
  const [peeks, setPeeks] = useState(false);
  useEffect(() => {
    let live = true;
    const look = () =>
      settings()
        .then((kept) => {
          if (live) setPeeks(!kept.quiet.includes(QUIET));
        })
        .catch(() => {
          if (live) setPeeks(false);
        });
    look();
    window.addEventListener("focus", look);
    return () => {
      live = false;
      window.removeEventListener("focus", look);
    };
  }, []);
  return peeks;
}

export function Head({ className }: { readonly className?: string }) {
  return (
    <svg className={className} viewBox="44 24 362 302" aria-hidden="true">
      <g transform="translate(266 126) rotate(34)">
        <rect
          x="-9"
          y="-92"
          width="18"
          height="100"
          rx="3"
          fill="#E3B459"
          stroke="#4A4640"
          strokeWidth="4"
        />
        <path d="M-9 -76 L9 -76" stroke="#4A4640" strokeWidth="4" />
        <path
          d="M-9 -92 L0 -114 L9 -92 Z"
          fill="#F5F0E8"
          stroke="#4A4640"
          strokeWidth="4"
          strokeLinejoin="round"
        />
        <path d="M-3.6 -105 L0 -114 L3.6 -105 Z" fill="#4A4640" />
      </g>
      <circle cx="90" cy="108" r="24" fill="#9A6A1B" stroke="#4A4640" strokeWidth="6" />
      <circle cx="90" cy="108" r="10" fill="#F5F0E8" />
      <circle cx="310" cy="108" r="24" fill="#9A6A1B" stroke="#4A4640" strokeWidth="6" />
      <circle cx="310" cy="108" r="10" fill="#F5F0E8" />
      <path
        d="M200 90 C246 90 292 90 316 118 C344 150 346 212 326 250 C306 290 258 316 200 316 C142 316 94 290 74 250 C54 212 56 150 84 118 C108 90 154 90 200 90 Z"
        fill="#C8902F"
      />
      <path
        d="M118 118 Q150 104 192 104"
        stroke="#E3B459"
        strokeWidth="7"
        strokeLinecap="round"
        fill="none"
      />
      <path
        d="M128 232 C128 204 162 194 200 194 C238 194 272 204 272 232 C272 268 240 292 200 292 C160 292 128 268 128 232 Z"
        fill="#F5F0E8"
      />
      <path
        d="M182 250 H218 V280 Q218 290 208 290 H192 Q182 290 182 280 Z"
        fill="#F8E2A0"
        stroke="#4A4640"
        strokeWidth="4"
        strokeLinejoin="round"
      />
      <path d="M200 252 V289" stroke="#4A4640" strokeWidth="3" strokeLinecap="round" />
      <ellipse cx="200" cy="213" rx="17" ry="11" fill="#4A4640" />
      <ellipse cx="195" cy="209" rx="5" ry="2.5" fill="#F5F0E8" opacity="0.8" />
      <path
        d="M200 223 L200 248"
        stroke="#4A4640"
        strokeWidth="4"
        strokeLinecap="round"
        fill="none"
      />
      <path
        d="M160 238 Q180 258 200 248 Q224 262 246 228"
        stroke="#4A4640"
        strokeWidth="5"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
      />
      <path
        d="M248 238 L254 230"
        stroke="#4A4640"
        strokeWidth="4"
        strokeLinecap="round"
        fill="none"
      />
      <rect x="90" y="226" width="26" height="7" rx="3.5" fill="#F5F0E8" />
      <rect x="94" y="240" width="20" height="7" rx="3.5" fill="#F5F0E8" />
      <rect x="284" y="226" width="26" height="7" rx="3.5" fill="#F5F0E8" />
      <rect x="286" y="240" width="20" height="7" rx="3.5" fill="#F5F0E8" />
      <ellipse cx="146" cy="178" rx="17" ry="15" fill="#FFFFFF" stroke="#4A4640" strokeWidth="4" />
      <ellipse cx="254" cy="178" rx="17" ry="15" fill="#FFFFFF" stroke="#4A4640" strokeWidth="4" />
      <circle cx="154" cy="182" r="8.5" fill="#4A4640" />
      <circle cx="262" cy="182" r="8.5" fill="#4A4640" />
      <circle cx="157" cy="179" r="2.6" fill="#FFFFFF" />
      <circle cx="265" cy="179" r="2.6" fill="#FFFFFF" />
      <path
        d="M129 174 A17 15 0 0 1 163 174 Z"
        fill="#4A4640"
        stroke="#4A4640"
        strokeWidth="4"
        strokeLinejoin="round"
      />
      <path
        d="M237 172 A17 15 0 0 1 271 172 Z"
        fill="#4A4640"
        stroke="#4A4640"
        strokeWidth="4"
        strokeLinejoin="round"
      />
      <path
        d="M118 152 Q146 150 174 164"
        stroke="#4A4640"
        strokeWidth="8"
        strokeLinecap="round"
        fill="none"
      />
      <path
        d="M230 140 Q256 112 284 128"
        stroke="#4A4640"
        strokeWidth="8"
        strokeLinecap="round"
        fill="none"
      />
      <path
        d="M196 90 Q188 66 208 60 Q200 74 214 82"
        stroke="#4A4640"
        strokeWidth="6"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
      />
      <path
        d="M200 90 C246 90 292 90 316 118 C344 150 346 212 326 250 C306 290 258 316 200 316 C142 316 94 290 74 250 C54 212 56 150 84 118 C108 90 154 90 200 90 Z"
        fill="none"
        stroke="#4A4640"
        strokeWidth="8"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function Paws({ className }: { readonly className?: string }) {
  return (
    <span className={`paws ${className ?? ""}`} aria-hidden="true">
      <span className="paw">
        <i />
        <i />
        <i />
      </span>
      <span className="paw">
        <i />
        <i />
        <i />
      </span>
    </span>
  );
}

export function Thumb({ className }: { readonly className?: string }) {
  return (
    <svg className={className} viewBox="0 0 48 60" aria-hidden="true">
      <rect
        x="6"
        y="2"
        width="15"
        height="30"
        rx="7.5"
        fill="#C8902F"
        stroke="#4A4640"
        strokeWidth="3"
      />
      <path
        d="M4 28 Q4 22 12 22 H38 Q45 22 45 30 V48 Q45 57 36 57 H14 Q4 57 4 47 Z"
        fill="#C8902F"
        stroke="#4A4640"
        strokeWidth="3"
        strokeLinejoin="round"
      />
      <path
        d="M22 34 H40 M22 42 H40 M22 50 H38"
        stroke="#4A4640"
        strokeWidth="2.5"
        strokeLinecap="round"
      />
    </svg>
  );
}
