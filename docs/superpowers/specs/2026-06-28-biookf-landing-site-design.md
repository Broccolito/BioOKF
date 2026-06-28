# BioOKF landing site, design record

Date: 2026-06-28

## Goal

A static project landing site under `landing/`, mirroring the boxy, minimalist
aesthetic of the bioscratch landing site, but in BioOKF Studio's own palette
(warm grays plus a sky-blue accent) and the BioOKF wordmark's periwinkle. It
explains what BioOKF does, how to install it, and how to reach the GitHub repo,
and it embeds animated HTML mockups of the Studio UI built as HyperFrames
compositions. A GitHub Actions workflow publishes it to GitHub Pages.

## Decisions

- **Mockup delivery:** author real HyperFrames compositions (paused GSAP
  timelines on `window.__timelines[id]`) and embed them live with the official
  `<hyperframes-player>` web component from the CDN, falling back to a
  self-looping `<iframe>` if the player never registers. Confirmed working: the
  player mounts and renders all three in-page with playback controls.
- **Hosting:** GitHub Actions publishes `landing/` to GitHub Pages at
  `https://broccolito.github.io/BioOKF/`, triggered on push to `main` plus
  manual dispatch. URL is in the repo README; README and landing cross-link.
- **Palette:** app values, copied exactly. Surfaces `#fbfbfb`/`#f6f7f8`, ink
  `#0d0d0d`, muted `#5b5d66`, hairlines `rgba(13,13,13,.10)`, sky-blue accent
  `#0ea5e9`/`#0369a1`, periwinkle brand `#7EA6E0` from the wordmark. The 28
  node-type colors and 35 predicates come straight from the app and SPEC.md.

## Structure

- `landing/index.html` hero, what-it-is, three embedded demos, install, what you
  get, GitHub CTA, footer.
- `landing/docs.html` full reference: bundle, every node type and predicate
  defined, provenance, Studio, the live loop, CLI, MCP, build from source.
- `landing/frames/{graph,inspect,agents}.html` HyperFrames compositions:
  graph pulling into place; click node then edge for inspector and
  distributions; agent ingest with progress bars, terminal, and live updates.
- `landing/assets/{styles.css,site.js,logo.svg}` shared system; `logo.svg` is
  the supplied wordmark.
- `.github/workflows/deploy-pages.yml` Pages deploy.

## Revision feedback applied (user)

- Use the supplied BioOKF wordmark everywhere a brand mark appears; no invented
  placeholder marks.
- Squared-off (not fully round) pill and bullet edges.
- The "Want your agent to do it?" block flat and elegant, not card-like.
- No em dashes and no AI-tell phrasing in any user-facing copy.
- Every node type and edge predicate gets a real definition in the docs.
- Verified closely with Playwright at desktop and mobile; no misaligned layouts,
  no horizontal overflow.

## Go-live

Merge the branch to `main` (push triggers the workflow) and enable Pages with
source "GitHub Actions". The site then serves at the README URL.
