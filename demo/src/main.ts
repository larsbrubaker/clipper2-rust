// Main entry point - SPA router and WASM initialization

import { initWasm } from './wasm.ts';

// Demo page modules (lazy loaded)
type DemoInit = (container: HTMLElement) => (() => void) | void;
const demoModules: Record<string, () => Promise<{ init: DemoInit }>> = {
  'boolean-ops': () => import('./demos/boolean-ops.ts' as any),
  'fill-rules': () => import('./demos/fill-rules.ts' as any),
  'offset': () => import('./demos/offset.ts' as any),
  'rect-clip': () => import('./demos/rect-clip.ts' as any),
  'minkowski': () => import('./demos/minkowski.ts' as any),
  'simplify': () => import('./demos/simplify.ts' as any),
  'polytree': () => import('./demos/polytree.ts' as any),
  'utilities': () => import('./demos/utilities.ts' as any),
};

let currentCleanup: (() => void) | null = null;

function getRoute(): string {
  const hash = window.location.hash.slice(2) || '';
  return hash || 'home';
}

function updateNav(route: string) {
  document.querySelectorAll('.nav-link').forEach(el => {
    const r = (el as HTMLElement).dataset.route;
    el.classList.toggle('active', r === route);
  });
}

function renderHome(container: HTMLElement) {
  container.innerHTML = `
    <div class="home-page">
      <div class="github-badge">
        <a href="https://github.com/larsbrubaker/clipper2-rust" target="_blank" class="github-badge-link">
          <svg height="20" viewBox="0 0 16 16" width="20" fill="currentColor"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>
          <span>larsbrubaker/clipper2-rust</span>
        </a>
      </div>
      <div class="hero">
        <h1>Clipper2 <span>for Rust</span></h1>
        <p>
          A complete, production-ready Rust port of the Clipper2 polygon clipping library.
          Explore interactive demos showcasing boolean operations, path offsetting,
          Minkowski operations, and more — all running in your browser via WebAssembly.
        </p>
      </div>
      <div class="feature-grid">
        <a href="#/boolean-ops" class="feature-card">
          <span class="card-icon">&#9649;</span>
          <h3>Boolean Operations</h3>
          <p>Intersection, Union, Difference, and Xor on polygons with draggable shapes and fill rule control.</p>
        </a>
        <a href="#/fill-rules" class="feature-card">
          <span class="card-icon">&#9638;</span>
          <h3>Fill Rules</h3>
          <p>Compare EvenOdd, NonZero, Positive, and Negative fill rules side-by-side on self-intersecting polygons.</p>
        </a>
        <a href="#/offset" class="feature-card">
          <span class="card-icon">&#10562;</span>
          <h3>Path Offsetting</h3>
          <p>Inflate and deflate paths with configurable JoinType, EndType, miter limit, and arc tolerance.</p>
        </a>
        <a href="#/rect-clip" class="feature-card">
          <span class="card-icon">&#9634;</span>
          <h3>Rectangle Clipping</h3>
          <p>Fast rectangle clipping of closed polygons and open polylines with a draggable clip region.</p>
        </a>
        <a href="#/minkowski" class="feature-card">
          <span class="card-icon">&#10753;</span>
          <h3>Minkowski Operations</h3>
          <p>Minkowski Sum and Difference — sweep a pattern along a path to create complex offsets.</p>
        </a>
        <a href="#/simplify" class="feature-card">
          <span class="card-icon">&#10140;</span>
          <h3>Path Simplification</h3>
          <p>Compare SimplifyPath and Ramer-Douglas-Peucker algorithms with adjustable epsilon tolerance.</p>
        </a>
        <a href="#/polytree" class="feature-card">
          <span class="card-icon">&#9776;</span>
          <h3>PolyTree</h3>
          <p>Visualize hierarchical polygon results showing parent/child/hole relationships in a tree view.</p>
        </a>
        <a href="#/utilities" class="feature-card">
          <span class="card-icon">&#9881;</span>
          <h3>Utilities</h3>
          <p>Point-in-polygon testing, area calculation, bounds detection, and orientation checks.</p>
        </a>
      </div>
      <div class="about-section">
        <h2>About This Project</h2>
        <p>
          This is a complete Rust port of Angus Johnson's
          <a href="https://github.com/AngusJohnson/Clipper2" target="_blank">Clipper2</a> C++ library,
          implementing all 857 items (790 functions, 56 classes, 11 enums) with exact behavioral matching.
          The demos above run entirely in your browser via WebAssembly compiled from the Rust source.
        </p>
        <p style="margin-top: 12px">
          Ported by <strong>Lars Brubaker</strong>, sponsored by
          <a href="https://www.matterhackers.com" target="_blank">MatterHackers</a>.
        </p>
        <div class="stats-row">
          <div class="stat">
            <div class="stat-value">857</div>
            <div class="stat-label">Items Ported</div>
          </div>
          <div class="stat">
            <div class="stat-value">444</div>
            <div class="stat-label">Tests Passing</div>
          </div>
          <div class="stat">
            <div class="stat-value">100%</div>
            <div class="stat-label">C++ Parity</div>
          </div>
          <div class="stat">
            <div class="stat-value">0</div>
            <div class="stat-label">Unsafe Blocks</div>
          </div>
        </div>
      </div>
    </div>
  `;
}

async function navigate(route: string) {
  const container = document.getElementById('main-content')!;

  // Cleanup previous demo
  if (currentCleanup) {
    currentCleanup();
    currentCleanup = null;
  }

  updateNav(route);

  if (route === 'home') {
    renderHome(container);
    return;
  }

  const loader = demoModules[route];
  if (!loader) {
    container.innerHTML = `<div class="home-page"><h2>Page not found</h2><p>Unknown route: ${route}</p></div>`;
    return;
  }

  container.innerHTML = `<div class="home-page" style="display:flex;align-items:center;justify-content:center;height:80vh;"><p style="color:var(--text-muted)">Loading demo...</p></div>`;

  try {
    await initWasm();
    const mod = await loader();
    container.innerHTML = '';
    const cleanup = mod.init(container);
    if (cleanup) currentCleanup = cleanup;
  } catch (e) {
    console.error('Failed to load demo:', e);
    container.innerHTML = `<div class="home-page"><h2>Error loading demo</h2><pre style="color:var(--clip-stroke)">${e}</pre></div>`;
  }
}

// Route on hash change
window.addEventListener('hashchange', () => navigate(getRoute()));

// Initial load
navigate(getRoute());
