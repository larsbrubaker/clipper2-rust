import { DemoCanvas } from '../canvas.ts';
import { createCheckbox, createSeparator, createInfoBox, createButton, createReadout, updateReadout } from '../controls.ts';
import { createCodePanel } from '../code-display.ts';
import { loadDemoState, saveDemoState } from '../persist.ts';
import { rectClip, rectClipLines, makeEllipse, makeStar } from '../wasm.ts';

const RUST_CODE = `pub fn rect_clip_64(rect: &Rect64, paths: &Paths64) -> Paths64 {
    if rect.is_empty() || paths.is_empty() {
        return Paths64::new();
    }
    let mut rc = RectClip64::new(*rect);
    rc.execute(paths)
}

pub fn rect_clip_lines_64(rect: &Rect64, lines: &Paths64) -> Paths64 {
    if rect.is_empty() || lines.is_empty() {
        return Paths64::new();
    }
    let mut rcl = RectClipLines64::new(*rect);
    rcl.execute(lines)
}`;

const JS_CODE = `// Clip closed polygons to rectangle
const closed = rectClip(100, 100, 400, 400, subjects);

// Clip open polylines to rectangle
const open = rectClipLines(100, 100, 400, 400, lines);`;

export function init(container: HTMLElement) {
  function makeDefaultShapes(): number[][][] {
    return [
      makeEllipse(200, 180, 120, 100, 0),
      makeStar(350, 300, 110, 50, 5),
      makeEllipse(150, 340, 80, 80, 0),
    ];
  }
  const persisted = loadDemoState('rect-clip', {
    linesMode: false,
    rectLeft: 120,
    rectTop: 100,
    rectRight: 420,
    rectBottom: 380,
    shapes: makeDefaultShapes(),
  });

  let linesMode = persisted.linesMode;
  let rectLeft = persisted.rectLeft, rectTop = persisted.rectTop, rectRight = persisted.rectRight, rectBottom = persisted.rectBottom;
  let shapes: number[][][] = persisted.shapes;
  function persistState() {
    saveDemoState('rect-clip', {
      linesMode,
      rectLeft,
      rectTop,
      rectRight,
      rectBottom,
      shapes,
    });
  }


  // Rect drag state
  let dragEdge: 'left' | 'right' | 'top' | 'bottom' | 'move' | null = null;
  let dragStartMouseX = 0;
  let dragStartMouseY = 0;
  let dragStartRect = { left: 0, top: 0, right: 0, bottom: 0 };

  container.innerHTML = `
    <div class="demo-page">
      <div class="demo-header">
        <h2>Rectangle Clipping</h2>
        <p>Drag the rectangle edges or body to clip shapes. Toggle between closed polygons and open lines.</p>
      </div>
      <div class="demo-body">
        <div class="demo-canvas-area">
          <canvas id="demo-canvas"></canvas>
          <div class="canvas-info" id="coord-display">0, 0</div>
          <div class="canvas-hint"><span class="hint-desktop">Drag rectangle edges to resize · Drag body to move · Right-click to pan</span><span class="hint-mobile">Drag to resize/move · Pinch to zoom · Two-finger pan</span></div>
        </div>
        <div class="demo-controls" id="controls"></div>
      </div>
    </div>
  `;

  const canvas = new DemoCanvas(document.getElementById('demo-canvas') as HTMLCanvasElement);
  canvas.coordDisplay = document.getElementById('coord-display');
  const controls = document.getElementById('controls')!;

  controls.appendChild(createInfoBox('Drag the clipping rectangle to see portions of shapes clipped in real-time.'));
  controls.appendChild(createCheckbox('Open lines mode', linesMode, (v) => { linesMode = v; persistState(); redraw(); }));
  controls.appendChild(createSeparator());

  controls.appendChild(createButton('Add Ellipse', () => {
    shapes.push(makeEllipse(
      100 + Math.random() * 350,
      100 + Math.random() * 300,
      50 + Math.random() * 80,
      50 + Math.random() * 80,
      0
    ));
    persistState();
    redraw();
  }));
  controls.appendChild(createButton('Add Star', () => {
    shapes.push(makeStar(
      100 + Math.random() * 350,
      100 + Math.random() * 300,
      60 + Math.random() * 60,
      20 + Math.random() * 30,
      3 + Math.floor(Math.random() * 5)
    ));
    persistState();
    redraw();
  }));
  controls.appendChild(createButton('Reset', () => {
    shapes = makeDefaultShapes();
    rectLeft = 120; rectTop = 100; rectRight = 420; rectBottom = 380;
    persistState();
    redraw();
  }));

  controls.appendChild(createSeparator());
  const readout = createReadout();
  controls.appendChild(readout);

  const { container: codePanel, toggle: codeToggle } = createCodePanel([
    { label: 'Rust', code: RUST_CODE, language: 'rust' },
    { label: 'JavaScript', code: JS_CODE, language: 'typescript' },
  ]);
  container.querySelector('.demo-canvas-area')!.appendChild(codeToggle);
  container.querySelector('.demo-body')!.after(codePanel);

  const EDGE_THRESHOLD = 10;

  canvas.enableInteraction({
    onDragStart(wx, wy) {
      const [sl] = canvas.worldToScreen(rectLeft, 0);
      const [sr] = canvas.worldToScreen(rectRight, 0);
      const [, st] = canvas.worldToScreen(0, rectTop);
      const [, sb] = canvas.worldToScreen(0, rectBottom);
      const [sx, sy] = canvas.worldToScreen(wx, wy);

      if (Math.abs(sx - sl) < EDGE_THRESHOLD && sy >= st && sy <= sb) dragEdge = 'left';
      else if (Math.abs(sx - sr) < EDGE_THRESHOLD && sy >= st && sy <= sb) dragEdge = 'right';
      else if (Math.abs(sy - st) < EDGE_THRESHOLD && sx >= sl && sx <= sr) dragEdge = 'top';
      else if (Math.abs(sy - sb) < EDGE_THRESHOLD && sx >= sl && sx <= sr) dragEdge = 'bottom';
      else if (sx >= sl && sx <= sr && sy >= st && sy <= sb) dragEdge = 'move';
      else dragEdge = null;

      dragStartRect = { left: rectLeft, top: rectTop, right: rectRight, bottom: rectBottom };
      dragStartMouseX = wx;
      dragStartMouseY = wy;
    },
    onDragMove(wx, wy) {
      if (!dragEdge) return;
      if (dragEdge === 'left') rectLeft = Math.round(wx);
      else if (dragEdge === 'right') rectRight = Math.round(wx);
      else if (dragEdge === 'top') rectTop = Math.round(wy);
      else if (dragEdge === 'bottom') rectBottom = Math.round(wy);
      else if (dragEdge === 'move') {
        const dx = wx - dragStartMouseX;
        const dy = wy - dragStartMouseY;
        rectLeft = Math.round(dragStartRect.left + dx);
        rectRight = Math.round(dragStartRect.right + dx);
        rectTop = Math.round(dragStartRect.top + dy);
        rectBottom = Math.round(dragStartRect.bottom + dy);
      }
      redraw();
    },
    onDragEnd() { dragEdge = null; persistState(); },
    redraw() { redraw(); },
  });

  canvas.fitBounds(-20, -20, 550, 500);
  canvas.setResizeRedraw(() => redraw());

  function redraw() {
    canvas.clear();
    canvas.drawGrid();

    // Draw original shapes (dimmed)
    canvas.drawPaths(shapes, { stroke: '#ccc', lineWidth: 1, lineDash: [4, 3] });

    // Draw clip rectangle
    canvas.drawRect(rectLeft, rectTop, rectRight, rectBottom, { fill: 'rgba(37, 99, 235, 0.06)', stroke: '#2563eb', lineWidth: 2, lineDash: [8, 4] });

    // Clip and draw result
    try {
      let result: number[][][];
      if (linesMode) {
        result = rectClipLines(rectLeft, rectTop, rectRight, rectBottom, shapes);
        canvas.drawOpenPaths(result, { stroke: '#00b48c', lineWidth: 2.5 });
      } else {
        result = rectClip(rectLeft, rectTop, rectRight, rectBottom, shapes);
        canvas.drawPaths(result, { fill: 'rgba(0, 180, 140, 0.2)', stroke: '#00b48c', lineWidth: 2 });
      }
      updateReadout(readout, [
        { label: 'Input paths', value: String(shapes.length) },
        { label: 'Result paths', value: String(result.length) },
        { label: 'Rect', value: `[${rectLeft}, ${rectTop}, ${rectRight}, ${rectBottom}]` },
      ]);
    } catch (e) {
      console.error('Rect clip error:', e);
    }
  }

  redraw();
}
