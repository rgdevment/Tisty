import type { JSX } from "@solidjs/web";

const Icon = (props: { readonly children: JSX.Element; readonly class?: string }) => (
  <svg class={`icon ${props.class ?? ""}`} viewBox="0 0 16 16" aria-hidden="true">
    {props.children}
  </svg>
);

export const Search = () => (
  <Icon>
    <circle cx="7" cy="7" r="4.5" />
    <path d="m10.5 10.5 3 3" />
  </Icon>
);

export const ToDark = () => (
  <Icon class="theme-to-dark">
    <path d="M13.5 9.6A6 6 0 0 1 6.4 2.5a6 6 0 1 0 7.1 7.1z" />
  </Icon>
);

export const ToLight = () => (
  <Icon class="theme-to-light">
    <circle cx="8" cy="8" r="2.8" />
    <path d="M8 1.5v1.4M8 13.1v1.4M1.5 8h1.4M13.1 8h1.4M3.4 3.4l1 1M11.6 11.6l1 1M3.4 12.6l1-1M11.6 4.4l1-1" />
  </Icon>
);

export const Dots = () => (
  <Icon>
    <circle cx="3.5" cy="8" r="0.6" />
    <circle cx="8" cy="8" r="0.6" />
    <circle cx="12.5" cy="8" r="0.6" />
  </Icon>
);

export const Lines = () => (
  <Icon>
    <path d="M3 4.5h10M3 8h10M3 11.5h6" />
  </Icon>
);

export const Crossed = () => (
  <Icon>
    <path d="m4.5 4.5 7 7M11.5 4.5l-7 7" />
  </Icon>
);

export const Back = () => (
  <Icon>
    <path d="M10 3.5 5.5 8l4.5 4.5" />
  </Icon>
);

export const Up = () => (
  <Icon>
    <path d="m4.5 10 3.5-3.5 3.5 3.5" />
  </Icon>
);

export const Down = () => (
  <Icon>
    <path d="m4.5 6 3.5 3.5L11.5 6" />
  </Icon>
);

export const Logo = () => (
  <svg
    class="brand-mark"
    viewBox="31.5 8.5 156 156"
    aria-hidden="true"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <rect
      class="logo-back"
      x="70"
      y="22"
      width="104"
      height="128"
      rx="14"
      transform="rotate(7 120 90)"
    />
    <g transform="rotate(-6 100 90)">
      <path
        class="logo-front"
        d="M58 24h74l26 26v86a14 14 0 0 1-14 14H58a14 14 0 0 1-14-14V38a14 14 0 0 1 14-14Z"
      />
      <path class="logo-fold" d="M132 24v14a12 12 0 0 0 12 12h14" />
      <path class="logo-lines" d="M62 68h56M62 86h72M62 104h44" />
      <path class="logo-note" d="M62 122h28" />
    </g>
    <circle class="logo-badge" cx="150" cy="134" r="20" />
    <path class="logo-check" d="M140.5 134.5l6.5 6.5 12.5-13" />
  </svg>
);
