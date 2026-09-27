# Aero Design Language

> *"Light as air. Fast as thought."*

AeroOS has its own design identity — **Aero**, inspired by glass, light, and
fluid motion. It is not a copy of any existing system. It has its own rules.

## Principles

1. **Weightless** — UI elements look like they float in space.
2. **Translucent** — every panel reveals the layer beneath.
3. **Alive** — subtle motion, never static, never distracting.
4. **Precise** — every pixel has a reason; nothing is decorative.

## Colour System

| Token | Dark | Light | Aero |
|-------|------|-------|------|
| `--accent` | `#4a9eff` | `#4a9eff` | `#00d4ff` |
| `--bg-1` | `#0a0a12` | `#eef2f7` | `#051220` |
| `--bg-2` | `#12121e` | `#e4e9f0` | `#0a1e30` |
| `--bg-3` | `#0d1520` | `#dde3eb` | `#0d2840` |
| `--text` | `#ffffff` | `#1a1a2e` | `#e6f5ff` |

The **Aero theme** is our signature — a deep cyan accent, cool blue base,
electric glow. It's the "brand colour" of AeroOS.

## Typography

- System font stack (native rendering)
- No custom web fonts — for speed and self-containment
- Weight scale: 400 / 600 / 700

## Components

### Glass Panel
The fundamental building block — a translucent surface with a subtle
coloured glow that breathes over 8 seconds. This is the Aero *signature*.

### Aero Dock
Icons have a radial accent gradient that intensifies on hover.

### Cards
Flat panels with a single border and micro-inner-shadow.

### Window Controls
Three dots: red (close), yellow (minimize), green (maximize) — the classic
traffic-light metaphor, universal and instantly readable.

## Motion

- **Easing:** `cubic-bezier(.34, 1.56, .64, 1)` — slightly overshooting
- **Durations:** 150 ms micro, 250 ms transitions, 400 ms theme change
- **Reduced motion:** respects `prefers-reduced-motion`

## Sound (planned)

- Soft click on window open (0.05 s, 8 kHz, low volume)
- Subtle whoosh on snapshot restore
- No sound by default — opt-in only

## Philosophy

AeroOS is not Windows. Not macOS. Not Linux. It has its own visual voice.

When a user opens AeroOS for the first time, they should feel:
1. **Familiar** — buttons where you expect them.
2. **Distinct** — this is clearly *its own thing*.
3. **Calm** — no clutter, no noise, no ads.

Aero is design as infrastructure: consistent, invisible when it works,
striking when you look for it.

---

*— Aero Design Team, 2026*