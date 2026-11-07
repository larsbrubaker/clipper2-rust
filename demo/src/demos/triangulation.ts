// Constrained triangulation with the two C++ issue #12 regression examples.
import { DemoCanvas } from '../canvas.ts';
import { createButton, createCheckbox, createDropdown, createInfoBox, createReadout, createSeparator, updateReadout } from '../controls.ts';
import { createCodePanel } from '../code-display.ts';
import { loadDemoState, saveDemoState } from '../persist.ts';
import { singlePathArea, triangulate } from '../wasm.ts';

type Paths = number[][][];
const samples: Record<string, { text: string; paths: Paths }> = {
  concave: { text: 'Concave polygon', paths: [[[0,0],[300,0],[300,100],[120,100],[120,300],[0,300]]] },
  hole: { text: 'Polygon with a hole', paths: [
    [[0,0],[400,0],[400,300],[0,300]], [[100,80],[100,220],[300,220],[300,80]],
  ] },
  vertex: { text: 'Collinear vertex', paths: [
    [[-209,3155],[716,3534],[3324,2439],[298,9844],[507,6689]],
  ] },
  holeVertices: { text: 'Hole with collinear vertices', paths: [
    [[320313,10000],[310000,160313],[99688,150000],[-313,90000],[10000,59688]],
    [[210000,150000],[220000,130000],[220000,150000],[310000,150000],[10000,70000]],
  ] },
};
const rustCode = `use clipper2_rust::{triangulate_64, TriangulateResult};

let mut triangles = Vec::new();
let result = triangulate_64(&paths, &mut triangles, true);
assert_eq!(result, TriangulateResult::Success);
// Positive-area outer boundaries, negative-area holes.`;
const jsCode = `const triangles = triangulate(paths, true);
// Integer coordinates; throws if paths intersect.`;

export function init(container: HTMLElement) {
  const saved = loadDemoState('triangulation', { sample: 'vertex', delaunay: true });
  let sample = Object.hasOwn(samples, saved.sample) ? saved.sample : 'vertex';
  let delaunay = saved.delaunay;
  let paths: Paths = structuredClone(samples[sample].paths);
  let dragged: [number, number] | null = null;

  container.innerHTML = `
    <div class="demo-page">
      <div class="demo-header">
        <h2>Triangulation</h2>
        <p>Explore a polygon's triangles and holes. Drag a vertex to recompute the mesh.</p>
      </div>
      <div class="demo-body">
        <div class="demo-canvas-area">
          <canvas id="demo-canvas" aria-label="Polygon triangulation; drag vertices to edit"></canvas>
          <div class="canvas-info" id="coord-display">Cartesian coordinates · Y upward</div>
          <div class="canvas-hint">Drag vertices · Scroll to zoom · Right-click to pan · Y upward</div>
        </div>
        <div class="demo-controls" id="controls"></div>
      </div>
    </div>`;
  const canvas = new DemoCanvas(container.querySelector('canvas')!);
  const controls = container.querySelector<HTMLElement>('#controls')!;
  const coord = container.querySelector<HTMLElement>('#coord-display')!;
  const readout = createReadout();
  readout.setAttribute('role', 'status');
  readout.setAttribute('aria-live', 'polite');
  function persist() { saveDemoState('triangulation', { sample, delaunay }); }
  function fit() {
    const points = paths.flat();
    canvas.fitBounds(Math.min(...points.map(p => p[0])), -Math.max(...points.map(p => p[1])),
      Math.max(...points.map(p => p[0])), -Math.min(...points.map(p => p[1])), 65);
  }
  controls.appendChild(createInfoBox('Outer boundaries run counterclockwise; holes clockwise. Dashed red lines mark hole boundaries.'));
  controls.appendChild(createDropdown('Example', Object.entries(samples).map(([value, s]) => ({ value, text: s.text })), sample, value => {
    sample = value; paths = structuredClone(samples[value].paths); dragged = null;
    persist(); fit(); redraw();
  }));
  controls.appendChild(createCheckbox('Delaunay edge refinement', delaunay, value => {
    delaunay = value; persist(); redraw();
  }));
  controls.appendChild(createButton('Reset vertices', () => { paths = structuredClone(samples[sample].paths); fit(); redraw(); }));
  controls.appendChild(createButton('Fit to view', () => { fit(); redraw(); }));
  controls.appendChild(createSeparator());
  controls.appendChild(readout);
  const code = createCodePanel([
    { label: 'Rust', code: rustCode, language: 'rust' },
    { label: 'JavaScript', code: jsCode, language: 'typescript' },
  ]);
  container.querySelector('.demo-canvas-area')!.appendChild(code.toggle);
  container.querySelector('.demo-body')!.after(code.container);

  // Negate Y only for display: the WASM call receives the original coordinates.
  const display = (ps: Paths): Paths => ps.map(p => p.map(([x, y]) => [x, -y]));
  function redraw() {
    canvas.clear();
    try {
      const triangles = triangulate(paths, delaunay);
      triangles.forEach((t, i) => canvas.drawPaths(display([t]), {
        fill: `hsla(${(i * 137.5 + 165) % 360}, 62%, 52%, 0.24)`,
        stroke: '#526779', lineWidth: 1,
      }));
      const inputArea = Math.abs(paths.reduce((sum, p) => sum + singlePathArea(p), 0));
      const triangleArea = triangles.reduce((sum, t) => sum + Math.abs(singlePathArea(t)), 0);
      updateReadout(readout, [
        { label: 'Status', value: 'Success' },
        { label: 'Triangles', value: String(triangles.length) },
        { label: 'Polygon area', value: inputArea.toLocaleString('en-US', { maximumFractionDigits: 1 }) },
        { label: 'Triangle area', value: triangleArea.toLocaleString('en-US', { maximumFractionDigits: 1 }) },
        { label: 'Area difference', value: Math.abs(triangleArea - inputArea).toLocaleString('en-US', { maximumFractionDigits: 1 }) },
      ]);
    } catch (e) {
      readout.textContent = `Cannot triangulate: ${String(e)}`;
    }
    paths.forEach(p => canvas.drawPaths(display([p]), {
      stroke: singlePathArea(p) < 0 ? '#c43f4b' : '#183f45', lineWidth: 2,
      lineDash: singlePathArea(p) < 0 ? [5, 4] : [], vertices: true, vertexRadius: 5,
    }));
  }
  canvas.enableInteraction({
    onDragStart(wx, wy) {
      dragged = null;
      const [sx, sy] = canvas.worldToScreen(wx, wy);
      let best = 14;
      paths.forEach((p, pi) => p.forEach(([x, y], vi) => {
        const [px, py] = canvas.worldToScreen(x, -y);
        const d = Math.hypot(px - sx, py - sy);
        if (d < best) { best = d; dragged = [pi, vi]; }
      }));
    },
    onDragMove(wx, wy) {
      if (!dragged) return;
      paths[dragged[0]][dragged[1]] = [Math.round(wx), -Math.round(wy)]; redraw();
    },
    onDragEnd() { dragged = null; },
    onMouseMove(wx, wy) { coord.textContent = `${Math.round(wx)}, ${-Math.round(wy)} · Y upward`; },
    redraw,
  });
  canvas.setResizeRedraw(redraw);
  fit(); redraw();
}
