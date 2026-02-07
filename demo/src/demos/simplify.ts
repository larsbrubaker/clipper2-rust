import { DemoCanvas } from '../canvas.ts';
import { createSlider, createCheckbox, createSeparator, createInfoBox, createButton, createReadout, updateReadout } from '../controls.ts';
import { createCodePanel } from '../code-display.ts';
import { simplifyPaths, rdpSimplify } from '../wasm.ts';

const RUST_CODE = `// SimplifyPath: removes vertices within epsilon of imaginary line
pub fn simplify_path<T>(
    path: &Path<T>,
    epsilon: f64,
    is_closed_path: bool,
) -> Path<T> { /* ... */ }

// Ramer-Douglas-Peucker: classic line simplification
pub fn ramer_douglas_peucker<T>(
    path: &Path<T>,
    epsilon: f64,
) -> Path<T> { /* ... */ }`;

const JS_CODE = `// SimplifyPath
const simplified = simplifyPaths([path], epsilon, isClosed);

// Ramer-Douglas-Peucker
const rdpResult = rdpSimplify([path], epsilon);`;

export function init(container: HTMLElement) {
  let epsilon = 5;
  let isClosed = false;
  let originalPath: number[][] = generateSpiralPath();

  function generateSpiralPath(): number[][] {
    const pts: number[][] = [];
    for (let i = 0; i <= 200; i++) {
      const t = (i / 200) * Math.PI * 4;
      const r = 30 + t * 25;
      // Add noise for interesting simplification
      const noise = Math.sin(i * 0.5) * 8 + Math.cos(i * 0.7) * 5;
      pts.push([
        Math.round(250 + (r + noise) * Math.cos(t)),
        Math.round(250 + (r + noise) * Math.sin(t)),
      ]);
    }
    return pts;
  }

  container.innerHTML = `
    <div class="demo-page">
      <div class="demo-header">
        <h2>Path Simplification</h2>
        <p>Compare SimplifyPath vs Ramer-Douglas-Peucker side by side. Draw or use a preset, then adjust epsilon.</p>
      </div>
      <div class="demo-body">
        <div class="simplify-split">
          <div class="simplify-panel">
            <div class="panel-header">
              <span>SimplifyPath</span>
              <span class="count" id="sp-count">0 pts</span>
            </div>
            <canvas id="canvas-simplify"></canvas>
          </div>
          <div class="simplify-panel">
            <div class="panel-header">
              <span>Ramer-Douglas-Peucker</span>
              <span class="count" id="rdp-count">0 pts</span>
            </div>
            <canvas id="canvas-rdp"></canvas>
          </div>
        </div>
        <div class="demo-controls" id="controls"></div>
      </div>
    </div>
  `;

  const canvasSP = new DemoCanvas(document.getElementById('canvas-simplify') as HTMLCanvasElement);
  const canvasRDP = new DemoCanvas(document.getElementById('canvas-rdp') as HTMLCanvasElement);
  const spCount = document.getElementById('sp-count')!;
  const rdpCount = document.getElementById('rdp-count')!;
  const controls = document.getElementById('controls')!;

  controls.appendChild(createInfoBox('Increase epsilon to remove more points. SimplifyPath uses perpendicular distance; RDP recursively selects the most important points.'));

  controls.appendChild(createSlider('Epsilon', 0, 50, epsilon, 1, (v) => { epsilon = v; redraw(); }));
  controls.appendChild(createCheckbox('Closed path', isClosed, (v) => { isClosed = v; redraw(); }));

  controls.appendChild(createSeparator());
  controls.appendChild(createButton('Spiral (dense)', () => { originalPath = generateSpiralPath(); redraw(); }));
  controls.appendChild(createButton('Zigzag', () => {
    const pts: number[][] = [];
    for (let i = 0; i <= 100; i++) {
      const x = 50 + (i / 100) * 400;
      const y = 250 + (i % 2 === 0 ? -1 : 1) * (20 + Math.random() * 40);
      pts.push([Math.round(x), Math.round(y)]);
    }
    originalPath = pts;
    redraw();
  }));
  controls.appendChild(createButton('Random walk', () => {
    const pts: number[][] = [[250, 250]];
    for (let i = 0; i < 150; i++) {
      const prev = pts[pts.length - 1];
      pts.push([
        Math.round(prev[0] + (Math.random() - 0.5) * 30),
        Math.round(prev[1] + (Math.random() - 0.5) * 30),
      ]);
    }
    originalPath = pts;
    redraw();
  }));

  controls.appendChild(createSeparator());
  const readout = createReadout();
  controls.appendChild(readout);

  const { container: codePanel, toggle: codeToggle } = createCodePanel([
    { label: 'Rust', code: RUST_CODE, language: 'rust' },
    { label: 'JavaScript', code: JS_CODE, language: 'typescript' },
  ]);
  const splitArea = container.querySelector('.simplify-split')!;
  (splitArea as HTMLElement).style.position = 'relative';
  const toggleWrap = document.createElement('div');
  toggleWrap.style.cssText = 'position:absolute;bottom:0;left:0;right:0;z-index:10';
  toggleWrap.appendChild(codeToggle);
  splitArea.appendChild(toggleWrap);
  container.querySelector('.demo-body')!.after(codePanel);

  // Freehand drawing on left canvas
  let isDrawing = false;
  const spCanvas = document.getElementById('canvas-simplify') as HTMLCanvasElement;
  spCanvas.addEventListener('mousedown', (e) => {
    if (e.button !== 0 || e.altKey) return;
    isDrawing = true;
    originalPath = [];
    const rect = spCanvas.getBoundingClientRect();
    const [wx, wy] = canvasSP.screenToWorld(e.clientX - rect.left, e.clientY - rect.top);
    originalPath.push([Math.round(wx), Math.round(wy)]);
  });
  spCanvas.addEventListener('mousemove', (e) => {
    if (!isDrawing) return;
    const rect = spCanvas.getBoundingClientRect();
    const [wx, wy] = canvasSP.screenToWorld(e.clientX - rect.left, e.clientY - rect.top);
    originalPath.push([Math.round(wx), Math.round(wy)]);
    redraw();
  });
  spCanvas.addEventListener('mouseup', () => { isDrawing = false; redraw(); });
  spCanvas.addEventListener('mouseleave', () => { isDrawing = false; });

  function redraw() {
    for (const dc of [canvasSP, canvasRDP]) {
      dc.fitBounds(0, 0, 500, 500);
      dc.clear();
      dc.drawGrid();
    }

    // Original
    const drawOpts = { stroke: '#ccc', lineWidth: 1, lineDash: [4, 3] as number[] };
    if (isClosed) {
      canvasSP.drawPaths([originalPath], drawOpts);
      canvasRDP.drawPaths([originalPath], drawOpts);
    } else {
      canvasSP.drawOpenPaths([originalPath], drawOpts);
      canvasRDP.drawOpenPaths([originalPath], drawOpts);
    }

    // SimplifyPath
    try {
      const spResult = simplifyPaths([originalPath], epsilon, isClosed);
      const spPath = spResult[0] || [];
      if (isClosed) {
        canvasSP.drawPaths([spPath], { stroke: '#2563eb', lineWidth: 2, vertices: true, vertexRadius: 3, vertexColor: '#2563eb' });
      } else {
        canvasSP.drawOpenPaths([spPath], { stroke: '#2563eb', lineWidth: 2, vertices: true, vertexRadius: 3, vertexColor: '#2563eb' });
      }
      spCount.textContent = `${spPath.length} pts`;
    } catch (e) {
      spCount.textContent = 'error';
    }

    // RDP
    try {
      const rdpResult = rdpSimplify([originalPath], epsilon);
      const rdpPath = rdpResult[0] || [];
      if (isClosed) {
        canvasRDP.drawPaths([rdpPath], { stroke: '#00b48c', lineWidth: 2, vertices: true, vertexRadius: 3, vertexColor: '#00b48c' });
      } else {
        canvasRDP.drawOpenPaths([rdpPath], { stroke: '#00b48c', lineWidth: 2, vertices: true, vertexRadius: 3, vertexColor: '#00b48c' });
      }
      rdpCount.textContent = `${rdpPath.length} pts`;
    } catch (e) {
      rdpCount.textContent = 'error';
    }

    updateReadout(readout, [
      { label: 'Original points', value: String(originalPath.length) },
      { label: 'Epsilon', value: String(epsilon) },
    ]);
  }

  // Set resize redraw on both canvases
  canvasSP.setResizeRedraw(() => redraw());
  canvasRDP.setResizeRedraw(() => redraw());

  setTimeout(redraw, 50);
}
