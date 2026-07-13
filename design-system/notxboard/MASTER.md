# notXboard Design System

> Applies to the public dashboard, Maintainable user portal, full admin console,
> super-admin command center, and standalone authentication pages.

## Product Context

notXboard is an operations-focused shared node platform with three audiences:

- Users purchase plans, manage subscriptions, open tickets, and request refunds.
- Node owners manage nodes, access, node plans, traffic, and node-scoped support.
- Super administrators operate the platform, review risk, manage configuration, and monitor health.

Interfaces should be quiet, data-dense, predictable, and optimized for repeated work. Do not use
marketing-page composition inside authenticated surfaces.

## Principles

1. Put status and next actions before explanation.
2. Group navigation by workflow and role, not by implementation module.
3. Keep data readable without relying on color alone.
4. Use stable dimensions so loading, error, and populated states do not shift the layout.
5. Prefer system fonts, deferred work, and cached assets over decorative effects.
6. Keep the public dashboard inspectable without requiring authentication.

## Color Tokens

### Light Surfaces

| Role | Value | Usage |
| --- | --- | --- |
| Canvas | `#F3F6F5` | Page background |
| Surface | `#FFFFFF` | Cards, forms, tool panels |
| Surface subtle | `#F8FAF9` | Secondary rows and notices |
| Text | `#172026` | Primary text |
| Muted text | `#5E6B73` | Supporting text |
| Border | `#DCE4E1` | Default separators |
| Primary | `#0F766E` | Primary actions and active navigation |
| Data blue | `#2563EB` | Neutral data series and links |
| Warning | `#B45309` | Warning text and state |
| Danger | `#B42318` | Errors and destructive actions |
| Success | `#15803D` | Healthy and completed states |

### Operations Dark Surface

| Role | Value | Usage |
| --- | --- | --- |
| Canvas | `#0C1214` | Command center background |
| Surface | `#151E21` | Monitoring panels |
| Surface strong | `#192427` | Raised controls |
| Text | `#EDF4F2` | Primary text |
| Muted text | `#9CAEAA` | Supporting text |
| Border | `#2D3A3E` | Separators |
| Primary | `#2DD4BF` | Active and healthy state |
| Data blue | `#60A5FA` | Neutral data series |
| Warning | `#FBBF24` | Warning state |
| Danger | `#FB7185` | Error state |

Use dark mode only for monitoring-heavy views or when explicitly selected. Avoid blue-only palettes,
gradients, glow, grid overlays, decorative orbs, and low-contrast glass surfaces.

## Typography

Use the native system stack to avoid blocking font downloads:

```css
font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC",
  "Microsoft YaHei", sans-serif;
```

Use `SFMono-Regular`, Consolas, or a system monospace only for tokens, paths, metrics, and code.

- Page title: `28px` to `34px`, weight `700` to `760`.
- Section title: `18px` to `22px`, weight `700`.
- Panel title: `14px` to `16px`, weight `650` to `720`.
- Body: `14px` to `16px`.
- Supporting text: `12px` to `13px`, never below WCAG contrast requirements.
- Letter spacing is always `0`.
- Do not scale font size with viewport width.

## Layouts

### Public Dashboard

- Sticky 64px header with product identity and one console action.
- Compact page heading followed by a stable five-metric grid.
- Two-column ranking panels on desktop and one column on narrow screens.
- Region list and map form one framed data tool, not a marketing illustration.
- Map code and geo data load only when the map is near the viewport.

### User Portal

- Desktop uses a fixed 248px sidebar with workflow groups.
- Mobile uses a 58px top bar and an off-canvas navigation drawer.
- Main content width is capped at 1320px with 24px desktop gutters and 14px mobile gutters.
- Authentication forms use a focused 500px panel.

### Admin Console

- Sidebar and content are full-height work surfaces, not floating page cards.
- Dark and light modes share identical spacing and component geometry.
- Tables, filters, and actions favor density and scanability.
- The command center may use the operations dark palette but must remain restrained.

## Components

### Geometry

- Cards and tool panels: maximum `8px` radius.
- Inputs and buttons: `7px` radius.
- Status badges may be pill-shaped when the shape communicates state.
- Do not nest decorative cards inside cards.

### Buttons

- Minimum height: `40px` desktop and `44px` for primary mobile actions.
- Primary: solid primary color with white text.
- Secondary: white or dark surface with a visible border.
- Destructive: danger text with a tinted background and visible border.
- Hover changes color, border, or shadow only. It must not resize or shift layout.
- Every control has a visible `:focus-visible` outline.

### Forms

- Inputs have persistent labels and a minimum height of `42px`.
- Errors use text plus color and an announced live region where content is asynchronous.
- Loading and validation states do not change the field width.

### Data Panels

- Use tabular numerals for metrics.
- Keep labels, values, and units visible at all supported widths.
- Use row separators instead of card styling for every list item.
- Loading skeletons preserve the populated layout.
- Empty and error states are explicit and scoped to their panel.

## Performance Rules

- Fixed Rust templates are compiled into the gateway binary.
- Static files are streamed, compressed when accepted, and served with ETag validation.
- Versioned assets use immutable one-year browser caching.
- Non-versioned assets use revalidation-friendly bounded caching.
- Do not add external web fonts to core surfaces.
- Defer legacy portal bundles while preserving execution order.
- Pause polling while the document is hidden.
- Lazy-load large visualizations and below-fold data.
- Avoid adding a frontend framework solely for styling existing server-rendered pages.

## Accessibility And Responsive Checks

- Contrast is at least 4.5:1 for body text.
- Color is never the only state indicator.
- All images have useful alt text or are explicitly decorative.
- Buttons and links have accessible names.
- `prefers-reduced-motion` disables non-essential animation.
- Validate at `375x812`, `768x1024`, `1024x768`, and `1440x900`.
- No horizontal page scrolling at any supported viewport.
- Long values and translations must not clip or overlap adjacent content.
