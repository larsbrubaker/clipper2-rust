import { DemoCanvas } from '../canvas.ts';
import { createSlider, createCheckbox, createSeparator, createInfoBox, createButtonGroup, createReadout, updateReadout } from '../controls.ts';
import { createCodePanel } from '../code-display.ts';
import { loadDemoState, saveDemoState } from '../persist.ts';
import { minkowskiSum, minkowskiDiff, makeEllipse } from '../wasm.ts';

const RUST_CODE = `pub fn minkowski_sum(
    pattern: &Path64,   // Small shape to sweep
    path: &Path64,      // Path to sweep along
    is_closed: bool,    // Treat path as closed polygon?
) -> Paths64 {
    union_paths(
        &minkowski_internal(pattern, path, true, is_closed),
        FillRule::NonZero,
    )
}

pub fn minkowski_diff(
    pattern: &Path64,
    path: &Path64,
    is_closed: bool,
) -> Paths64 {
    union_paths(
        &minkowski_internal(pattern, path, false, is_closed),
        FillRule::NonZero,
    )
}`;

const JS_CODE = `// Minkowski Sum: sweep pattern along path
const result = minkowskiSum(pattern, path, true);

// Minkowski Difference
const diff = minkowskiDiff(pattern, path, true);`;

export function init(container: HTMLElement) {
  const persisted = loadDemoState('minkowski', {
    isClosed: true,
    isSum: true,
    patternSize: 30,
    patternType: 'circle',
    path: [
      [100, 100], [400, 100], [400, 250], [250, 250], [250, 400], [100, 400],
    ],
  });

  let isClosed = persisted.isClosed;
  let isSum = persisted.isSum;
  let patternSize = persisted.patternSize;
  let patternType = persisted.patternType;
  let path: number[][] = persisted.path;
  function persistState() {
    saveDemoState('minkowski', { isClosed, isSum, patternSize, patternType, path });
  }


  function getPattern(): number[][] {
    if (patternType === 'circle') {
      return makeEllipse(0, 0, patternSize, patternSize, 16);
    } else if (patternType === 'square') {
      const s = patternSize;
      return [[-s, -s], [s, -s], [s, s], [-s, s]];
    } else {
      const s = patternSize;
      return [[0, -s], [s, s], [-s, s]];
    }
  }

  container.innerHTML = `
    <div class="demo-page">
      <div class="demo-header">
        <h2>Minkowski Operations</h2>
        <p>Sweep a pattern shape along a path. Sum adds, Difference subtracts.</p>
      </div>
      <div class="demo-body">
        <div class="demo-canvas-area">
          <canvas id="demo-canvas"></canvas>
          <div class="canvas-info" id="coord-display">0, 0</div>
          <div class="canvas-hint"><span class="hint-desktop">Drag path vertices to reshape · Right-click to pan · Scroll to zoom</span><span class="hint-mobile">Drag vertices · Pinch to zoom · Two-finger pan</span></div>
        </div>
        <div class="demo-controls" id="controls"></div>
      </div>
    </div>
  `;

  const canvas = new DemoCanvas(document.getElementById('demo-canvas') as HTMLCanvasElement);
  canvas.coordDisplay = document.getElementById('coord-display');
  const controls = document.getElementById('controls')!;

  controls.appendChild(createInfoBox('Minkowski Sum sweeps a pattern along a path, creating a buffered region. Useful for robot motion planning.'));

  const opLabel = document.createElement('div');
  opLabel.className = 'control-group';
  opLabel.innerHTML = '<label>Operation</label>';
  controls.appendChild(opLabel);
  controls.appendChild(createButtonGroup([
    { label: 'Sum', value: 'sum' },
    { label: 'Difference', value: 'diff' },
  ], isSum ? 'sum' : 'diff', (v) => { isSum = v === 'sum'; persistState(); redraw(); }));

  const patLabel = document.createElement('div');
  patLabel.className = 'control-group';
  patLabel.innerHTML = '<label>Pattern Shape</label>';
  controls.appendChild(patLabel);
  controls.appendChild(createButtonGroup([
    { label: 'Circle', value: 'circle' },
    { label: 'Square', value: 'square' },
    { label: 'Triangle', value: 'triangle' },
  ], patternType, (v) => { patternType = v; persistState(); redraw(); }));

  controls.appendChild(createSlider('Pattern Size', 5, 80, patternSize, 1, (v) => { patternSize = v; persistState(); redraw(); }));
  controls.appendChild(createCheckbox('Closed path', isClosed, (v) => { isClosed = v; persistState(); redraw(); }));

  controls.appendChild(createSeparator());
  const readout = createReadout();
  controls.appendChild(readout);

  const { container: codePanel, toggle: codeToggle } = createCodePanel([
    { label: 'Rust', code: RUST_CODE, language: 'rust' },
    { label: 'JavaScript', code: JS_CODE, language: 'typescript' },
  ]);
  container.querySelector('.demo-canvas-area')!.appendChild(codeToggle);
  container.querySelector('.demo-body')!.after(codePanel);

  let dragIdx: number | null = null;

  canvas.enableInteraction({
    onDragStart(wx, wy) {
      for (let i = 0; i < path.length; i++) {
        const [sx, sy] = canvas.worldToScreen(path[i][0], path[i][1]);
        const [msx, msy] = canvas.worldToScreen(wx, wy);
        if (Math.abs(sx - msx) < 12 && Math.abs(sy - msy) < 12) {
          dragIdx = i;
          return;
        }
      }
    },
    onDragMove(wx, wy) {
      if (dragIdx !== null) {
        path[dragIdx] = [Math.round(wx), Math.round(wy)];
        redraw();
      }
    },
    onDragEnd() { dragIdx = null; persistState(); },
    redraw() { redraw(); },
  });

  canvas.fitBounds(-20, -20, 520, 520);
  canvas.setResizeRedraw(() => redraw());

  function redraw() {
    canvas.clear();
    canvas.drawGrid();

    const pattern = getPattern();

    // Draw original path
    if (isClosed) {
      canvas.drawPaths([path], { stroke: '#999', lineWidth: 1.5, lineDash: [6, 4], vertices: true, vertexRadius: 5, vertexColor: '#666' });
    } else {
      canvas.drawOpenPaths([path], { stroke: '#999', lineWidth: 1.5, lineDash: [6, 4], vertices: true, vertexRadius: 5, vertexColor: '#666' });
    }

    // Draw pattern at origin (as reference, small)
    const patternAtOrigin = pattern.map(([x, y]) => [x + 50, y + 50]);
    canvas.drawPaths([patternAtOrigin], { fill: 'rgba(235, 68, 68, 0.2)', stroke: '#eb4444', lineWidth: 1 });
    canvas.drawText('pattern', 50, 50 + patternSize + 15, { color: '#eb4444', font: '11px system-ui', align: 'center' });

    // Compute Minkowski
    try {
      const result = isSum
        ? minkowskiSum(pattern, path, isClosed)
        : minkowskiDiff(pattern, path, isClosed);

      canvas.drawPaths(result, { fill: 'rgba(37, 99, 235, 0.18)', stroke: '#2563eb', lineWidth: 2 });

      updateReadout(readout, [
        { label: 'Operation', value: isSum ? 'Sum' : 'Difference' },
        { label: 'Result paths', value: String(result.length) },
        { label: 'Pattern vertices', value: String(pattern.length) },
        { label: 'Path vertices', value: String(path.length) },
      ]);
    } catch (e) {
      console.error('Minkowski error:', e);
    }
  }

  redraw();
}
