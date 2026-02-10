import { DemoCanvas } from '../canvas.ts';
import { createSlider, createDropdown, createCheckbox, createSeparator, createInfoBox, createReadout, updateReadout } from '../controls.ts';
import { createCodePanel } from '../code-display.ts';
import { inflatePaths, JoinType, EndType, makeStar, makeEllipse } from '../wasm.ts';

const RUST_CODE = `pub fn inflate_paths_64(
    paths: &Paths64,
    delta: f64,        // Positive = inflate, negative = deflate
    jt: JoinType,      // Square, Bevel, Round, Miter
    et: EndType,       // Polygon, Joined, Butt, Square, Round
    miter_limit: f64,
    arc_tolerance: f64,
) -> Paths64 {
    let mut clip_offset = ClipperOffset::new(
        miter_limit, arc_tolerance, false, false
    );
    clip_offset.add_paths(paths, jt, et);
    let mut solution = Paths64::new();
    clip_offset.execute(delta, &mut solution);
    solution
}`;

const JS_CODE = `const result = inflatePaths(
  paths,
  delta,        // e.g. 20.0 for inflate, -10.0 for deflate
  JoinType.Round,
  EndType.Polygon,
  2.0,          // miter limit
  0.0           // arc tolerance (0 = auto)
);`;

export function init(container: HTMLElement) {
  let delta = 20;
  let joinType = JoinType.Round;
  let endType = EndType.Polygon;
  let miterLimit = 2.0;
  let arcTolerance = 0.0;
  let isOpen = false;

  // Default paths
  let paths: number[][][] = [
    [[100, 100], [350, 100], [350, 200], [250, 200], [250, 350], [100, 350]]
  ];

  container.innerHTML = `
    <div class="demo-page">
      <div class="demo-header">
        <h2>Path Offsetting</h2>
        <p>Inflate or deflate paths with configurable join and end types. Drag vertices to reshape.</p>
      </div>
      <div class="demo-body">
        <div class="demo-canvas-area">
          <canvas id="demo-canvas"></canvas>
          <div class="canvas-info" id="coord-display">0, 0</div>
          <div class="canvas-hint"><span class="hint-desktop">Drag vertices to reshape · Right-click to pan · Scroll to zoom</span><span class="hint-mobile">Drag vertices · Pinch to zoom · Two-finger pan</span></div>
        </div>
        <div class="demo-controls" id="controls"></div>
      </div>
    </div>
  `;

  const canvas = new DemoCanvas(document.getElementById('demo-canvas') as HTMLCanvasElement);
  canvas.coordDisplay = document.getElementById('coord-display');
  const controls = document.getElementById('controls')!;

  controls.appendChild(createInfoBox('Adjust the delta to inflate (positive) or deflate (negative) the path.'));

  const deltaSlider = createSlider('Delta', -50, 80, delta, 1, (v) => { delta = v; redraw(); });
  controls.appendChild(deltaSlider);

  controls.appendChild(createDropdown('Join Type', [
    { value: '2', text: 'Round' },
    { value: '0', text: 'Square' },
    { value: '1', text: 'Bevel' },
    { value: '3', text: 'Miter' },
  ], (v) => { joinType = parseInt(v) as JoinType; redraw(); }));

  controls.appendChild(createDropdown('End Type', [
    { value: '0', text: 'Polygon (closed)' },
    { value: '1', text: 'Joined' },
    { value: '2', text: 'Butt' },
    { value: '3', text: 'Square' },
    { value: '4', text: 'Round' },
  ], (v) => { endType = parseInt(v) as EndType; redraw(); }));

  controls.appendChild(createSlider('Miter Limit', 1, 10, miterLimit, 0.5, (v) => { miterLimit = v; redraw(); }));
  controls.appendChild(createSlider('Arc Tolerance', 0, 10, arcTolerance, 0.5, (v) => { arcTolerance = v; redraw(); }));

  controls.appendChild(createSeparator());
  controls.appendChild(createCheckbox('Open path mode', isOpen, (v) => {
    isOpen = v;
    if (isOpen && endType === EndType.Polygon) endType = EndType.Round;
    redraw();
  }));

  controls.appendChild(createSeparator());
  const readout = createReadout();
  controls.appendChild(readout);

  // Source code
  const { container: codePanel, toggle: codeToggle } = createCodePanel([
    { label: 'Rust', code: RUST_CODE, language: 'rust' },
    { label: 'JavaScript', code: JS_CODE, language: 'typescript' },
  ]);
  const canvasArea = container.querySelector('.demo-canvas-area')!;
  canvasArea.appendChild(codeToggle);
  container.querySelector('.demo-body')!.after(codePanel);

  // Vertex dragging
  let dragIdx: [number, number] | null = null;

  canvas.enableInteraction({
    onDragStart(wx, wy) {
      for (let pi = 0; pi < paths.length; pi++) {
        for (let vi = 0; vi < paths[pi].length; vi++) {
          const [vx, vy] = paths[pi][vi];
          const [sx, sy] = canvas.worldToScreen(vx, vy);
          const [msx, msy] = canvas.worldToScreen(wx, wy);
          if (Math.abs(sx - msx) < 12 && Math.abs(sy - msy) < 12) {
            dragIdx = [pi, vi];
            return;
          }
        }
      }
    },
    onDragMove(wx, wy) {
      if (dragIdx) {
        paths[dragIdx[0]][dragIdx[1]] = [Math.round(wx), Math.round(wy)];
        redraw();
      }
    },
    onDragEnd() { dragIdx = null; },
    redraw() { redraw(); },
  });

  canvas.fitBounds(-50, -50, 500, 500);
  canvas.setResizeRedraw(() => redraw());

  function redraw() {
    canvas.clear();
    canvas.drawGrid();

    // Original path
    if (isOpen) {
      canvas.drawOpenPaths(paths, { stroke: '#999', lineWidth: 1.5, lineDash: [6, 4], vertices: true, vertexRadius: 5, vertexColor: '#666' });
    } else {
      canvas.drawPaths(paths, { stroke: '#999', lineWidth: 1.5, lineDash: [6, 4], vertices: true, vertexRadius: 5, vertexColor: '#666' });
    }

    // Offset result
    try {
      const result = inflatePaths(paths, delta, joinType, endType, miterLimit, arcTolerance);
      canvas.drawPaths(result, { fill: 'rgba(37, 99, 235, 0.20)', stroke: '#2563eb', lineWidth: 2 });

      updateReadout(readout, [
        { label: 'Result paths', value: String(result.length) },
        { label: 'Delta', value: String(delta) },
        { label: 'Vertices in', value: String(paths.reduce((s, p) => s + p.length, 0)) },
        { label: 'Vertices out', value: String(result.reduce((s, p) => s + p.length, 0)) },
      ]);
    } catch (e) {
      console.error('Offset error:', e);
    }
  }

  redraw();
}
