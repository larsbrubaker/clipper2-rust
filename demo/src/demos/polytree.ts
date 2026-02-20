import { DemoCanvas } from '../canvas.ts';
import { createDropdown, createSeparator, createInfoBox, createButtonGroup, createReadout, updateReadout } from '../controls.ts';
import { createCodePanel } from '../code-display.ts';
import { loadDemoState, saveDemoState } from '../persist.ts';
import { polyTreeOp, ClipType, FillRule, type PolyTreeNode } from '../wasm.ts';

const RUST_CODE = `pub fn boolean_op_tree_64(
    clip_type: ClipType,
    fill_rule: FillRule,
    subjects: &Paths64,
    clips: &Paths64,
    solution: &mut PolyTree64,
) {
    let mut clipper = Clipper64::new();
    clipper.add_subject(subjects);
    clipper.add_clip(clips);
    clipper.execute_tree(
        clip_type, fill_rule, solution, &mut sol_open
    );
}

// PolyTree nodes form a hierarchy:
// - Depth 0: root (no polygon)
// - Depth 1: outer polygons
// - Depth 2: holes in outer polygons
// - Depth 3: nested polygons inside holes
// ... and so on`;

const JS_CODE = `const tree = polyTreeOp(
  ClipType.Union,
  FillRule.EvenOdd,
  subjects,
  clips
);

// tree.children is an array of PolyTreeNode:
// { polygon: [[x,y],...], is_hole: bool, depth: number, children: [...] }`;

const DEPTH_COLORS = [
  '#2563eb', '#00b48c', '#eb4444', '#9333ea', '#f59e0b', '#06b6d4',
];

export function init(container: HTMLElement) {
  const defaultSubjects: number[][][] = [
    [[50, 50], [450, 50], [450, 450], [50, 450]],
    [[100, 100], [400, 100], [400, 400], [100, 400]],
    [[150, 150], [350, 150], [350, 350], [150, 350]],
    [[200, 200], [300, 200], [300, 300], [200, 300]],
  ];
  const persisted = loadDemoState('polytree', {
    fillRule: FillRule.EvenOdd,
    selectedNodeIdx: -1,
    preset: 'nested',
    subjects: defaultSubjects,
    clips: [] as number[][][],
  });

  let fillRule = persisted.fillRule as FillRule;
  let selectedNodeIdx = persisted.selectedNodeIdx;
  let preset = persisted.preset;
  let subjects: number[][][] = persisted.subjects;
  let clips: number[][][] = persisted.clips;
  function persistState() {
    saveDemoState('polytree', { fillRule, selectedNodeIdx, preset, subjects, clips });
  }


  container.innerHTML = `
    <div class="demo-page">
      <div class="demo-header">
        <h2>PolyTree Hierarchy</h2>
        <p>Visualize parent/child/hole relationships. Click tree nodes to highlight polygons.</p>
      </div>
      <div class="demo-body">
        <div class="polytree-layout">
          <div class="tree-panel" id="tree-panel">
            <div style="font-size:12px;color:var(--text-muted);margin-bottom:12px;">Click a node to highlight</div>
          </div>
          <div class="demo-canvas-area" style="flex:1">
            <canvas id="demo-canvas"></canvas>
            <div class="canvas-info" id="coord-display">0, 0</div>
          </div>
        </div>
        <div class="demo-controls" id="controls"></div>
      </div>
    </div>
  `;

  const canvas = new DemoCanvas(document.getElementById('demo-canvas') as HTMLCanvasElement);
  canvas.coordDisplay = document.getElementById('coord-display');
  const treePanel = document.getElementById('tree-panel')!;
  const controls = document.getElementById('controls')!;

  controls.appendChild(createInfoBox('PolyTree captures the nesting hierarchy: outer polygons contain holes, which may contain nested polygons.'));

  controls.appendChild(createDropdown('Fill Rule', [
    { value: '0', text: 'EvenOdd' },
    { value: '1', text: 'NonZero' },
  ], String(fillRule), (v) => { fillRule = parseInt(v) as FillRule; persistState(); redraw(); }));

  controls.appendChild(createSeparator());
  const presetLabel = document.createElement('div');
  presetLabel.className = 'control-group';
  presetLabel.innerHTML = '<label>Preset</label>';
  controls.appendChild(presetLabel);
  controls.appendChild(createButtonGroup([
    { label: 'Nested rects', value: 'nested' },
    { label: 'Two shapes', value: 'two' },
    { label: 'Complex', value: 'complex' },
  ], preset, (v) => {
    preset = v;
    clips = [];
    if (v === 'nested') {
      subjects = [
        [[50, 50], [450, 50], [450, 450], [50, 450]],
        [[100, 100], [400, 100], [400, 400], [100, 400]],
        [[150, 150], [350, 150], [350, 350], [150, 350]],
        [[200, 200], [300, 200], [300, 300], [200, 300]],
      ];
    } else if (v === 'two') {
      subjects = [
        [[50, 50], [250, 50], [250, 250], [50, 250]],
        [[80, 80], [220, 80], [220, 220], [80, 220]],
      ];
      clips = [
        [[200, 200], [450, 200], [450, 450], [200, 450]],
        [[230, 230], [420, 230], [420, 420], [230, 420]],
      ];
    } else {
      subjects = [
        [[30, 30], [470, 30], [470, 470], [30, 470]],
        [[60, 60], [440, 60], [440, 440], [60, 440]],
        [[100, 100], [230, 100], [230, 230], [100, 230]],
        [[120, 120], [210, 120], [210, 210], [120, 210]],
        [[270, 100], [400, 100], [400, 230], [270, 230]],
        [[290, 120], [380, 120], [380, 210], [290, 210]],
        [[100, 270], [400, 270], [400, 400], [100, 400]],
        [[130, 290], [370, 290], [370, 380], [130, 380]],
      ];
    }
    selectedNodeIdx = -1;
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

  canvas.enableInteraction({ redraw() { redraw(); } });
  canvas.fitBounds(-20, -20, 520, 520);
  canvas.setResizeRedraw(() => redraw());

  let allNodes: { node: PolyTreeNode; flatIdx: number }[] = [];

  function flattenTree(node: PolyTreeNode, list: { node: PolyTreeNode; flatIdx: number }[]): void {
    const idx = list.length;
    list.push({ node, flatIdx: idx });
    for (const child of node.children) {
      flattenTree(child, list);
    }
  }

  function renderTree(root: PolyTreeNode) {
    allNodes = [];
    // Flatten children of root
    for (const child of root.children) {
      flattenTree(child, allNodes);
    }

    treePanel.innerHTML = '<div style="font-size:12px;color:var(--text-muted);margin-bottom:12px;">Click a node to highlight</div>';

    for (let i = 0; i < allNodes.length; i++) {
      const { node } = allNodes[i];
      const depth = node.depth;
      const el = document.createElement('div');
      el.className = 'tree-node' + (i === selectedNodeIdx ? ' selected' : '');
      const indent = (depth - 1) * 16;
      const color = DEPTH_COLORS[(depth - 1) % DEPTH_COLORS.length];
      el.innerHTML = `
        <span class="depth-indent" style="width:${indent}px"></span>
        <span class="node-icon" style="background:${color}"></span>
        ${node.is_hole ? 'Hole' : 'Polygon'} (${node.polygon.length} pts, depth ${depth})
      `;
      el.addEventListener('click', () => {
        selectedNodeIdx = i;
        persistState();
        redraw();
      });
      treePanel.appendChild(el);
    }
  }

  function redraw() {
    canvas.clear();
    canvas.drawGrid();

    let tree: PolyTreeNode;
    try {
      tree = polyTreeOp(ClipType.Union, fillRule, subjects, clips);
    } catch (e) {
      console.error('PolyTree error:', e);
      return;
    }

    renderTree(tree);

    // Draw all polygons with depth-based coloring
    allNodes = [];
    for (const child of tree.children) {
      flattenTree(child, allNodes);
    }

    for (let i = 0; i < allNodes.length; i++) {
      const { node } = allNodes[i];
      if (node.polygon.length < 3) continue;
      const depth = node.depth;
      const color = DEPTH_COLORS[(depth - 1) % DEPTH_COLORS.length];
      const isSelected = i === selectedNodeIdx;

      const fillAlpha = isSelected ? 0.4 : 0.15;
      const strokeWidth = isSelected ? 3 : 1.5;

      // Convert hex to rgba
      const r = parseInt(color.slice(1, 3), 16);
      const g = parseInt(color.slice(3, 5), 16);
      const b = parseInt(color.slice(5, 7), 16);

      canvas.drawPaths([node.polygon], {
        fill: `rgba(${r},${g},${b},${fillAlpha})`,
        stroke: color,
        lineWidth: strokeWidth,
        vertices: isSelected,
        vertexRadius: 3,
        vertexColor: color,
      });

      if (node.is_hole) {
        // Draw hatch pattern for holes
        canvas.drawPaths([node.polygon], {
          stroke: color,
          lineWidth: 0.5,
          lineDash: [3, 3],
        });
      }
    }

    let totalNodes = allNodes.length;
    let holes = allNodes.filter(n => n.node.is_hole).length;
    let maxDepth = Math.max(0, ...allNodes.map(n => n.node.depth));

    updateReadout(readout, [
      { label: 'Total nodes', value: String(totalNodes) },
      { label: 'Outer polygons', value: String(totalNodes - holes) },
      { label: 'Holes', value: String(holes) },
      { label: 'Max depth', value: String(maxDepth) },
    ]);
  }

  setTimeout(redraw, 50);
}
