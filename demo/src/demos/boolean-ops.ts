import { DemoCanvas } from '../canvas.ts';
import { createDropdown, createCheckbox, createSeparator, createInfoBox, createButtonGroup, createReadout, updateReadout } from '../controls.ts';
import { createCodePanel } from '../code-display.ts';
import { loadDemoState, saveDemoState } from '../persist.ts';
import { booleanOp, ClipType, FillRule, makeStar, makeEllipse, singlePathArea, pointInPolygon } from '../wasm.ts';

const RUST_CODE = `pub fn boolean_op_64(
    clip_type: ClipType,
    fill_rule: FillRule,
    subjects: &Paths64,
    clips: &Paths64,
) -> Paths64 {
    let mut result = Paths64::new();
    let mut clipper = Clipper64::new();
    clipper.add_subject(subjects);
    clipper.add_clip(clips);
    clipper.execute(clip_type, fill_rule, &mut result, None);
    result
}`;

const JS_CODE = `import { booleanOp, ClipType, FillRule, encodePaths } from './wasm';

const subjects = [[[100,100],[300,100],[300,300],[100,300]]];
const clips = [[[200,200],[400,200],[400,400],[200,400]]];

const result = booleanOp(
  ClipType.Intersection,
  FillRule.EvenOdd,
  subjects,
  clips
);`;

export function init(container: HTMLElement) {
  const persisted = loadDemoState('boolean-ops', {
    clipType: ClipType.Intersection,
    fillRule: FillRule.EvenOdd,
    showSubject: true,
    showClip: true,
    showResult: true,
    subjectPreset: 'star',
    clipPreset: 'circle',
    subjectPaths: [makeStar(200, 200, 180, 80, 5)],
    clipPaths: [makeEllipse(280, 220, 150, 130, 0)],
  });

  // State
  let clipType = persisted.clipType as ClipType;
  let fillRule = persisted.fillRule as FillRule;
  let showSubject = persisted.showSubject;
  let showClip = persisted.showClip;
  let showResult = persisted.showResult;
  let subjectPreset = persisted.subjectPreset;
  let clipPreset = persisted.clipPreset;
  let subjectPaths: number[][][] = persisted.subjectPaths;
  let clipPaths: number[][][] = persisted.clipPaths;

  // Dragging
  let dragTarget: 'subject' | 'clip' | null = null;
  let dragOffsetX = 0;
  let dragOffsetY = 0;

  // Layout
  container.innerHTML = `
    <div class="demo-page">
      <div class="demo-header">
        <h2>Boolean Operations</h2>
        <p>Drag shapes to move them. Right-click or Alt+drag to pan, scroll to zoom.</p>
      </div>
      <div class="demo-body">
        <div class="demo-canvas-area">
          <canvas id="demo-canvas"></canvas>
          <div class="canvas-info" id="coord-display">0, 0</div>
          <div class="canvas-hint"><span class="hint-desktop">Right-click to pan · Scroll to zoom · Drag shapes to move</span><span class="hint-mobile">Drag shapes · Pinch to zoom · Two-finger pan</span></div>
        </div>
        <div class="demo-controls" id="controls"></div>
      </div>
    </div>
  `;

  const canvas = new DemoCanvas(document.getElementById('demo-canvas') as HTMLCanvasElement);
  canvas.coordDisplay = document.getElementById('coord-display');
  const controls = document.getElementById('controls')!;

  function persistState() {
    saveDemoState('boolean-ops', {
      clipType,
      fillRule,
      showSubject,
      showClip,
      showResult,
      subjectPreset,
      clipPreset,
      subjectPaths,
      clipPaths,
    });
  }

  // Controls
  controls.appendChild(createInfoBox('Select a clip operation and fill rule, then drag shapes to explore.'));

  controls.appendChild(createDropdown('Clip Type', [
    { value: '1', text: 'Intersection' },
    { value: '2', text: 'Union' },
    { value: '3', text: 'Difference' },
    { value: '4', text: 'Xor' },
  ], String(clipType), (v) => { clipType = parseInt(v) as ClipType; persistState(); redraw(); }));

  controls.appendChild(createDropdown('Fill Rule', [
    { value: '0', text: 'EvenOdd' },
    { value: '1', text: 'NonZero' },
    { value: '2', text: 'Positive' },
    { value: '3', text: 'Negative' },
  ], String(fillRule), (v) => { fillRule = parseInt(v) as FillRule; persistState(); redraw(); }));

  controls.appendChild(createSeparator());

  const presetLabel = document.createElement('div');
  presetLabel.className = 'control-group';
  presetLabel.innerHTML = '<label>Subject Shape</label>';
  controls.appendChild(presetLabel);
  controls.appendChild(createButtonGroup([
    { label: 'Star', value: 'star' },
    { label: 'Heptagram', value: 'heptagram' },
    { label: 'Square', value: 'square' },
    { label: 'Circle', value: 'circle' },
  ], subjectPreset, (v) => {
    subjectPreset = v;
    const cx = 200, cy = 200;
    if (v === 'star') subjectPaths = [makeStar(cx, cy, 180, 80, 5)];
    else if (v === 'heptagram') {
      // {7/3} heptagram — self-intersecting with 3 winding levels
      const pts: number[][] = [];
      for (let i = 0; i < 7; i++) {
        const vertexIdx = (i * 3) % 7;
        const angle = (vertexIdx / 7) * Math.PI * 2 - Math.PI / 2;
        pts.push([Math.round(cx + 180 * Math.cos(angle)), Math.round(cy + 180 * Math.sin(angle))]);
      }
      subjectPaths = [pts];
    }
    else if (v === 'square') subjectPaths = [[[cx-150,cy-150],[cx+150,cy-150],[cx+150,cy+150],[cx-150,cy+150]]];
    else subjectPaths = [makeEllipse(cx, cy, 160, 160, 0)];
    persistState();
    redraw();
  }));

  const clipLabel = document.createElement('div');
  clipLabel.className = 'control-group';
  clipLabel.innerHTML = '<label>Clip Shape</label>';
  controls.appendChild(clipLabel);
  controls.appendChild(createButtonGroup([
    { label: 'Circle', value: 'circle' },
    { label: 'Square', value: 'square' },
    { label: 'Star', value: 'star' },
  ], clipPreset, (v) => {
    clipPreset = v;
    const cx = 280, cy = 220;
    if (v === 'circle') clipPaths = [makeEllipse(cx, cy, 150, 130, 0)];
    else if (v === 'square') clipPaths = [[[cx-130,cy-130],[cx+130,cy-130],[cx+130,cy+130],[cx-130,cy+130]]];
    else clipPaths = [makeStar(cx, cy, 150, 60, 6)];
    persistState();
    redraw();
  }));

  controls.appendChild(createSeparator());
  controls.appendChild(createCheckbox('Show Subject', showSubject, (v) => { showSubject = v; persistState(); redraw(); }));
  controls.appendChild(createCheckbox('Show Clip', showClip, (v) => { showClip = v; persistState(); redraw(); }));
  controls.appendChild(createCheckbox('Show Result', showResult, (v) => { showResult = v; persistState(); redraw(); }));

  controls.appendChild(createSeparator());
  const readout = createReadout();
  controls.appendChild(readout);

  // Source code
  const { container: codePanel, toggle: codeToggle } = createCodePanel([
    { label: 'Rust', code: RUST_CODE, language: 'rust' },
    { label: 'JavaScript', code: JS_CODE, language: 'typescript' },
  ]);
  const demoBody = container.querySelector('.demo-body')!;
  const canvasArea = container.querySelector('.demo-canvas-area')!;
  canvasArea.appendChild(codeToggle);
  demoBody.after(codePanel);

  function centroid(paths: number[][][]): [number, number] {
    let sx = 0, sy = 0, n = 0;
    for (const p of paths) for (const [x, y] of p) { sx += x; sy += y; n++; }
    return n > 0 ? [sx / n, sy / n] : [0, 0];
  }

  function translatePaths(paths: number[][][], dx: number, dy: number): number[][][] {
    return paths.map(p => p.map(([x, y]) => [x + dx, y + dy]));
  }

  function hitTest(wx: number, wy: number, paths: number[][][]): boolean {
    const px = Math.round(wx), py = Math.round(wy);
    for (const p of paths) {
      // Point-in-polygon: 1 = inside, 0 = on boundary
      if (pointInPolygon(px, py, p) >= 0) return true;
      // Fallback: check near any edge/vertex in screen space (handles thin shapes)
      for (const [x, y] of p) {
        const [sx, sy] = canvas.worldToScreen(x, y);
        const [msx, msy] = canvas.worldToScreen(wx, wy);
        if (Math.abs(sx - msx) < 12 && Math.abs(sy - msy) < 12) return true;
      }
    }
    return false;
  }

  // Interaction
  canvas.enableInteraction({
    onDragStart(wx, wy) {
      if (hitTest(wx, wy, subjectPaths)) {
        dragTarget = 'subject';
        const [cx, cy] = centroid(subjectPaths);
        dragOffsetX = cx - wx;
        dragOffsetY = cy - wy;
      } else if (hitTest(wx, wy, clipPaths)) {
        dragTarget = 'clip';
        const [cx, cy] = centroid(clipPaths);
        dragOffsetX = cx - wx;
        dragOffsetY = cy - wy;
      }
    },
    onDragMove(wx, wy) {
      if (!dragTarget) return;
      const paths = dragTarget === 'subject' ? subjectPaths : clipPaths;
      const [cx, cy] = centroid(paths);
      const dx = (wx + dragOffsetX) - cx;
      const dy = (wy + dragOffsetY) - cy;
      const moved = translatePaths(paths, dx, dy);
      if (dragTarget === 'subject') subjectPaths = moved;
      else clipPaths = moved;
      redraw();
    },
    onDragEnd() {
      dragTarget = null;
      persistState();
    },
    redraw() { redraw(); },
  });

  canvas.fitBounds(-50, -50, 500, 500);
  canvas.setResizeRedraw(() => redraw());

  function redraw() {
    canvas.clear();
    canvas.drawGrid();

    // Compute result
    let result: number[][][] = [];
    try {
      result = booleanOp(clipType, fillRule, subjectPaths, clipPaths);
    } catch (e) {
      console.error('Boolean op error:', e);
    }

    // Draw
    if (showSubject) {
      canvas.drawPaths(subjectPaths, { fill: 'rgba(0, 180, 140, 0.15)', stroke: '#00b48c', lineWidth: 2, vertices: true, vertexRadius: 3, vertexColor: '#00b48c' });
    }
    if (showClip) {
      canvas.drawPaths(clipPaths, { fill: 'rgba(235, 68, 68, 0.15)', stroke: '#eb4444', lineWidth: 2, vertices: true, vertexRadius: 3, vertexColor: '#eb4444' });
    }
    if (showResult && result.length) {
      canvas.drawPaths(result, { fill: 'rgba(37, 99, 235, 0.25)', stroke: '#2563eb', lineWidth: 2.5 });
    }

    // Readout
    let totalArea = 0;
    for (const p of result) totalArea += Math.abs(singlePathArea(p));
    updateReadout(readout, [
      { label: 'Result paths', value: String(result.length) },
      { label: 'Total area', value: totalArea.toLocaleString() },
      { label: 'Subject vertices', value: String(subjectPaths.reduce((s, p) => s + p.length, 0)) },
      { label: 'Clip vertices', value: String(clipPaths.reduce((s, p) => s + p.length, 0)) },
    ]);
  }

  redraw();
}
