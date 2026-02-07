import { DemoCanvas } from '../canvas.ts';
import { createSeparator, createInfoBox, createButton, createReadout, updateReadout } from '../controls.ts';
import { createCodePanel } from '../code-display.ts';
import { pointInPolygon, singlePathArea, isPositivePath, getBounds, makeEllipse, makeStar } from '../wasm.ts';

const RUST_CODE = `// Point-in-polygon test using winding number rule
pub fn point_in_polygon(pt: Point64, polygon: &Path64)
    -> PointInPolygonResult
{
    // Returns: IsInside, IsOutside, or IsOn (boundary)
}

// Signed area (shoelace formula)
pub fn area(path: &Path<T>) -> f64 { /* ... */ }

// Positive = counter-clockwise orientation
pub fn is_positive(path: &Path64) -> bool { /* ... */ }

// Bounding rectangle of paths
pub fn get_bounds_paths(paths: &Paths64) -> Rect64 { /* ... */ }`;

const JS_CODE = `// Point-in-polygon test
const result = pointInPolygon(mouseX, mouseY, polygon);
// result: 1 = inside, -1 = outside, 0 = on boundary

// Area calculation
const area = singlePathArea(polygon);

// Orientation
const isCCW = isPositivePath(polygon);

// Bounds
const { left, top, right, bottom } = getBounds([polygon]);`;

export function init(container: HTMLElement) {
  let polygon: number[][] = [
    [100, 80], [400, 80], [450, 250], [350, 420], [150, 420], [50, 250]
  ];
  let testPoints: { x: number; y: number; result: number }[] = [];
  let mouseWorld: [number, number] = [0, 0];
  let mouseResult = 0;

  container.innerHTML = `
    <div class="demo-page">
      <div class="demo-header">
        <h2>Geometric Utilities</h2>
        <p>Move your mouse to test point-in-polygon. Click to place persistent test points. Drag vertices to reshape.</p>
      </div>
      <div class="demo-body">
        <div class="demo-canvas-area">
          <canvas id="demo-canvas"></canvas>
          <div class="canvas-info" id="coord-display">0, 0</div>
          <div class="canvas-hint">Move mouse to test point-in-polygon · Click to place markers · Drag vertices</div>
        </div>
        <div class="demo-controls" id="controls"></div>
      </div>
    </div>
  `;

  const canvas = new DemoCanvas(document.getElementById('demo-canvas') as HTMLCanvasElement);
  canvas.coordDisplay = document.getElementById('coord-display');
  const controls = document.getElementById('controls')!;

  controls.appendChild(createInfoBox('Move your mouse over the polygon. Green = inside, red = outside, yellow = on boundary.'));

  const shapeLabel = document.createElement('div');
  shapeLabel.className = 'control-group';
  shapeLabel.innerHTML = '<label>Polygon</label>';
  controls.appendChild(shapeLabel);
  controls.appendChild(createButton('Hexagon', () => {
    polygon = [[100, 80], [400, 80], [450, 250], [350, 420], [150, 420], [50, 250]];
    testPoints = [];
    redraw();
  }));
  controls.appendChild(createButton('Star', () => {
    polygon = makeStar(250, 250, 200, 80, 5);
    testPoints = [];
    redraw();
  }));
  controls.appendChild(createButton('Circle', () => {
    polygon = makeEllipse(250, 250, 180, 180, 0);
    testPoints = [];
    redraw();
  }));
  controls.appendChild(createButton('Clear points', () => {
    testPoints = [];
    redraw();
  }));

  controls.appendChild(createSeparator());
  const readout = createReadout();
  controls.appendChild(readout);

  controls.appendChild(createSeparator());

  // PIP result legend
  const legend = document.createElement('div');
  legend.className = 'info-readout';
  legend.innerHTML = `
    <span style="color:#16a34a">&#9679;</span> Inside (1)&nbsp;&nbsp;
    <span style="color:#dc2626">&#9679;</span> Outside (-1)&nbsp;&nbsp;
    <span style="color:#ca8a04">&#9679;</span> On boundary (0)
  `;
  controls.appendChild(legend);

  const { container: codePanel, toggle: codeToggle } = createCodePanel([
    { label: 'Rust', code: RUST_CODE, language: 'rust' },
    { label: 'JavaScript', code: JS_CODE, language: 'typescript' },
  ]);
  container.querySelector('.demo-canvas-area')!.appendChild(codeToggle);
  container.querySelector('.demo-body')!.after(codePanel);

  let dragIdx: number | null = null;

  canvas.enableInteraction({
    onDragStart(wx, wy) {
      // Check if near a vertex
      for (let i = 0; i < polygon.length; i++) {
        const [sx, sy] = canvas.worldToScreen(polygon[i][0], polygon[i][1]);
        const [msx, msy] = canvas.worldToScreen(wx, wy);
        if (Math.abs(sx - msx) < 12 && Math.abs(sy - msy) < 12) {
          dragIdx = i;
          return;
        }
      }
      // Click to add test point
      const x = Math.round(wx);
      const y = Math.round(wy);
      const result = pointInPolygon(x, y, polygon);
      testPoints.push({ x, y, result });
      redraw();
    },
    onDragMove(wx, wy) {
      if (dragIdx !== null) {
        polygon[dragIdx] = [Math.round(wx), Math.round(wy)];
        // Recompute test point results
        for (const tp of testPoints) {
          tp.result = pointInPolygon(tp.x, tp.y, polygon);
        }
        redraw();
      }
    },
    onDragEnd() { dragIdx = null; },
    onMouseMove(wx, wy) {
      mouseWorld = [wx, wy];
      mouseResult = pointInPolygon(Math.round(wx), Math.round(wy), polygon);
      redraw();
    },
    redraw() { redraw(); },
  });

  canvas.fitBounds(-20, -20, 520, 500);
  canvas.setResizeRedraw(() => redraw());

  function pipColor(result: number): string {
    if (result === 1) return '#16a34a';   // green - inside
    if (result === -1) return '#dc2626';  // red - outside
    return '#ca8a04';                     // yellow - on boundary
  }

  function redraw() {
    canvas.clear();
    canvas.drawGrid();

    // Draw polygon
    canvas.drawPaths([polygon], {
      fill: 'rgba(37, 99, 235, 0.10)',
      stroke: '#2563eb',
      lineWidth: 2,
      vertices: true,
      vertexRadius: 5,
      vertexColor: '#2563eb',
    });

    // Draw bounding box
    const bounds = getBounds([polygon]);
    canvas.drawRect(bounds.left, bounds.top, bounds.right, bounds.bottom, {
      stroke: '#ccc',
      lineWidth: 1,
      lineDash: [4, 4],
    });

    // Draw test points
    for (const tp of testPoints) {
      canvas.drawPoint(tp.x, tp.y, 5, pipColor(tp.result));
    }

    // Draw mouse cursor indicator
    const [mx, my] = mouseWorld;
    canvas.drawPoint(mx, my, 7, pipColor(mouseResult));
    // Ring around cursor
    const [sx, sy] = canvas.worldToScreen(mx, my);
    canvas.ctx.strokeStyle = pipColor(mouseResult);
    canvas.ctx.lineWidth = 2;
    canvas.ctx.beginPath();
    canvas.ctx.arc(sx, sy, 12, 0, Math.PI * 2);
    canvas.ctx.stroke();

    // Update readout
    const areaVal = singlePathArea(polygon);
    const isPos = isPositivePath(polygon);
    const pathLen = polygon.reduce((sum, _, i) => {
      const next = polygon[(i + 1) % polygon.length];
      const dx = next[0] - polygon[i][0];
      const dy = next[1] - polygon[i][1];
      return sum + Math.sqrt(dx * dx + dy * dy);
    }, 0);

    updateReadout(readout, [
      { label: 'Cursor PIP', value: mouseResult === 1 ? 'Inside' : mouseResult === -1 ? 'Outside' : 'On boundary' },
      { label: 'Area', value: Math.abs(areaVal).toLocaleString() },
      { label: 'Orientation', value: isPos ? 'CCW (positive)' : 'CW (negative)' },
      { label: 'Perimeter', value: Math.round(pathLen).toLocaleString() },
      { label: 'Vertices', value: String(polygon.length) },
      { label: 'Bounds', value: `[${bounds.left}, ${bounds.top}] to [${bounds.right}, ${bounds.bottom}]` },
      { label: 'Test points', value: String(testPoints.length) },
    ]);
  }

  redraw();
}
