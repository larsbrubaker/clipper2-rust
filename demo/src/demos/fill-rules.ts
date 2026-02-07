import { DemoCanvas } from '../canvas.ts';
import { createButtonGroup, createSeparator, createInfoBox } from '../controls.ts';
import { createCodePanel } from '../code-display.ts';
import { booleanOp, ClipType, FillRule, makeStar } from '../wasm.ts';

const RUST_CODE = `// FillRule determines how self-intersecting polygons are filled
pub enum FillRule {
    EvenOdd,  // Alternate fill (default)
    NonZero,  // Winding number != 0
    Positive, // Winding number > 0
    Negative, // Winding number < 0
}

// Apply different fill rules to the same polygon:
let result = union_64(&[polygon], &[], FillRule::EvenOdd);`;

const JS_CODE = `// Union with different fill rules on same self-intersecting polygon
const polygon = makeStar(0, 0, 200, 80, 5); // Self-intersecting star

const evenOdd  = booleanOp(ClipType.Union, FillRule.EvenOdd,  [polygon], []);
const nonZero  = booleanOp(ClipType.Union, FillRule.NonZero,  [polygon], []);
const positive = booleanOp(ClipType.Union, FillRule.Positive, [polygon], []);
const negative = booleanOp(ClipType.Union, FillRule.Negative, [polygon], []);`;

export function init(container: HTMLElement) {
  let polygon: number[][] = makeStar(250, 250, 200, 80, 5);

  container.innerHTML = `
    <div class="demo-page">
      <div class="demo-header">
        <h2>Fill Rules</h2>
        <p>See how different fill rules interpret the same self-intersecting polygon. Each quadrant uses a different rule.</p>
      </div>
      <div class="demo-body">
        <div class="fill-rules-grid">
          <div class="fill-rule-cell">
            <div class="cell-label">EvenOdd</div>
            <canvas id="canvas-evenodd"></canvas>
          </div>
          <div class="fill-rule-cell">
            <div class="cell-label">NonZero</div>
            <canvas id="canvas-nonzero"></canvas>
          </div>
          <div class="fill-rule-cell">
            <div class="cell-label">Positive</div>
            <canvas id="canvas-positive"></canvas>
          </div>
          <div class="fill-rule-cell">
            <div class="cell-label">Negative</div>
            <canvas id="canvas-negative"></canvas>
          </div>
        </div>
        <div class="demo-controls" id="controls"></div>
      </div>
    </div>
  `;

  const canvases: { dc: DemoCanvas; fillRule: FillRule; id: string }[] = [
    { dc: new DemoCanvas(document.getElementById('canvas-evenodd') as HTMLCanvasElement), fillRule: FillRule.EvenOdd, id: 'evenodd' },
    { dc: new DemoCanvas(document.getElementById('canvas-nonzero') as HTMLCanvasElement), fillRule: FillRule.NonZero, id: 'nonzero' },
    { dc: new DemoCanvas(document.getElementById('canvas-positive') as HTMLCanvasElement), fillRule: FillRule.Positive, id: 'positive' },
    { dc: new DemoCanvas(document.getElementById('canvas-negative') as HTMLCanvasElement), fillRule: FillRule.Negative, id: 'negative' },
  ];

  const controls = document.getElementById('controls')!;
  controls.appendChild(createInfoBox('Each quadrant applies a different fill rule to the same self-intersecting path. Try each shape to see how winding numbers affect the result.'));

  controls.appendChild(createSeparator());
  const shapeLabel = document.createElement('div');
  shapeLabel.className = 'control-group';
  shapeLabel.innerHTML = '<label>Shape</label>';
  controls.appendChild(shapeLabel);
  controls.appendChild(createButtonGroup([
    { label: 'Pentagram', value: 'star5' },
    { label: 'Heptagram', value: 'star73' },
    { label: 'Butterfly', value: 'butterfly' },
    { label: 'Rings', value: 'rings' },
  ], 'star5', (v) => {
    const cx = 250, cy = 250;
    switch (v) {
      case 'star5':
        // Classic {5/2} pentagram — center has winding 2
        // EvenOdd: hole in center; NonZero/Positive: solid
        polygon = makeStar(cx, cy, 200, 80, 5);
        break;
      case 'star73': {
        // {7/3} heptagram — connect every 3rd vertex of a heptagon
        // Creates 3 winding levels (1, 2, 3) for dramatic EvenOdd pattern
        const pts: number[][] = [];
        for (let i = 0; i < 7; i++) {
          const vertexIdx = (i * 3) % 7;
          const angle = (vertexIdx / 7) * Math.PI * 2 - Math.PI / 2;
          pts.push([Math.round(cx + 200 * Math.cos(angle)), Math.round(cy + 200 * Math.sin(angle))]);
        }
        polygon = pts;
        break;
      }
      case 'butterfly': {
        // Figure-8 (Lissajous) — left and right lobes have opposite winding
        // Positive fills one lobe, Negative fills the other
        const pts: number[][] = [];
        for (let i = 0; i < 120; i++) {
          const t = (i / 120) * Math.PI * 2;
          pts.push([Math.round(cx + 200 * Math.sin(t)), Math.round(cy + 180 * Math.sin(2 * t))]);
        }
        polygon = pts;
        break;
      }
      case 'rings': {
        // Two overlapping circles as one path — overlap region has winding 2
        // EvenOdd: vesica piscis (overlap excluded); NonZero: solid union
        const pts: number[][] = [];
        const steps = 64;
        for (let i = 0; i < steps; i++) {
          const t = (i / steps) * Math.PI * 2;
          pts.push([Math.round(170 + 150 * Math.cos(t)), Math.round(cy + 150 * Math.sin(t))]);
        }
        for (let i = 0; i < steps; i++) {
          const t = (i / steps) * Math.PI * 2;
          pts.push([Math.round(330 + 150 * Math.cos(t)), Math.round(cy + 150 * Math.sin(t))]);
        }
        polygon = pts;
        break;
      }
    }
    redraw();
  }));

  // Source code
  const { container: codePanel, toggle: codeToggle } = createCodePanel([
    { label: 'Rust', code: RUST_CODE, language: 'rust' },
    { label: 'JavaScript', code: JS_CODE, language: 'typescript' },
  ]);
  const demoBody = container.querySelector('.demo-body')!;
  const gridArea = container.querySelector('.fill-rules-grid')!;
  demoBody.after(codePanel);

  // We'll attach the toggle to the first canvas cell
  const firstCell = gridArea.querySelector('.fill-rule-cell') as HTMLElement;
  const toggleWrapper = document.createElement('div');
  toggleWrapper.style.cssText = 'position:absolute;bottom:0;left:0;right:0;z-index:10';
  toggleWrapper.appendChild(codeToggle);
  // Place toggle at bottom of grid
  (gridArea as HTMLElement).style.position = 'relative';
  (gridArea as HTMLElement).appendChild(toggleWrapper);

  function redraw() {
    for (const { dc, fillRule } of canvases) {
      dc.fitBounds(0, 0, 500, 500);
      dc.clear();
      dc.drawGrid();

      // Draw original polygon outline
      dc.drawPaths([polygon], { stroke: '#ccc', lineWidth: 1, lineDash: [4, 3] });

      // Apply fill rule via union
      try {
        const result = booleanOp(ClipType.Union, fillRule, [polygon], []);
        dc.drawPaths(result, { fill: 'rgba(37, 99, 235, 0.25)', stroke: '#2563eb', lineWidth: 2 });
      } catch (e) {
        console.error('Fill rule error:', e);
      }
    }
  }

  // Set resize redraw on all canvases
  for (const { dc } of canvases) {
    dc.setResizeRedraw(() => redraw());
  }

  // Initial draw with delay for canvas sizing
  setTimeout(redraw, 50);
}
