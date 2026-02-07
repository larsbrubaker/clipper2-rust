// WASM module loader and typed wrappers for Clipper2 operations

let wasmModule: any = null;

export async function initWasm(): Promise<void> {
  if (wasmModule) return;
  const mod = await import('/public/pkg/clipper2_wasm.js' as any);
  await mod.default();
  wasmModule = mod;
}

function getWasm(): any {
  if (!wasmModule) throw new Error('WASM not initialized. Call initWasm() first.');
  return wasmModule;
}

// ============================================================================
// Path encoding helpers
// ============================================================================

/** Encode multiple paths into flat buffer: [n, x0, y0, x1, y1, ..., n, x0, ...] */
export function encodePaths(paths: number[][][]): Float64Array {
  let size = 0;
  for (const p of paths) size += 1 + p.length * 2;
  const buf = new Float64Array(size);
  let i = 0;
  for (const p of paths) {
    buf[i++] = p.length;
    for (const [x, y] of p) {
      buf[i++] = x;
      buf[i++] = y;
    }
  }
  return buf;
}

/** Decode flat buffer back into array of paths */
export function decodePaths(buf: Float64Array | number[]): number[][][] {
  const paths: number[][][] = [];
  let i = 0;
  while (i < buf.length) {
    const n = buf[i++];
    const path: number[][] = [];
    for (let j = 0; j < n; j++) {
      path.push([buf[i], buf[i + 1]]);
      i += 2;
    }
    paths.push(path);
  }
  return paths;
}

/** Encode a single path as flat [x0, y0, x1, y1, ...] */
export function encodeSinglePath(path: number[][]): Float64Array {
  const buf = new Float64Array(path.length * 2);
  let i = 0;
  for (const [x, y] of path) {
    buf[i++] = x;
    buf[i++] = y;
  }
  return buf;
}

/** Decode a single flat path */
export function decodeSinglePath(buf: Float64Array | number[]): number[][] {
  const path: number[][] = [];
  for (let i = 0; i + 1 < buf.length; i += 2) {
    path.push([buf[i], buf[i + 1]]);
  }
  return path;
}

// ============================================================================
// Clipper2 WASM API
// ============================================================================

export enum ClipType { Intersection = 1, Union = 2, Difference = 3, Xor = 4 }
export enum FillRule { EvenOdd = 0, NonZero = 1, Positive = 2, Negative = 3 }
export enum JoinType { Square = 0, Bevel = 1, Round = 2, Miter = 3 }
export enum EndType { Polygon = 0, Joined = 1, Butt = 2, Square = 3, Round = 4 }

export function booleanOp(
  clipType: ClipType,
  fillRule: FillRule,
  subjects: number[][][],
  clips: number[][][],
): number[][][] {
  const w = getWasm();
  const result = w.boolean_op(clipType, fillRule, encodePaths(subjects), encodePaths(clips));
  return decodePaths(result);
}

export function inflatePaths(
  paths: number[][][],
  delta: number,
  joinType: JoinType,
  endType: EndType,
  miterLimit: number = 2.0,
  arcTolerance: number = 0.0,
): number[][][] {
  const w = getWasm();
  const result = w.inflate_paths(encodePaths(paths), delta, joinType, endType, miterLimit, arcTolerance);
  return decodePaths(result);
}

export function rectClip(
  left: number, top: number, right: number, bottom: number,
  paths: number[][][],
): number[][][] {
  const w = getWasm();
  const result = w.rect_clip(left, top, right, bottom, encodePaths(paths));
  return decodePaths(result);
}

export function rectClipLines(
  left: number, top: number, right: number, bottom: number,
  lines: number[][][],
): number[][][] {
  const w = getWasm();
  const result = w.rect_clip_lines(left, top, right, bottom, encodePaths(lines));
  return decodePaths(result);
}

export function minkowskiSum(pattern: number[][], path: number[][], isClosed: boolean): number[][][] {
  const w = getWasm();
  const result = w.mink_sum(encodeSinglePath(pattern), encodeSinglePath(path), isClosed);
  return decodePaths(result);
}

export function minkowskiDiff(pattern: number[][], path: number[][], isClosed: boolean): number[][][] {
  const w = getWasm();
  const result = w.mink_diff(encodeSinglePath(pattern), encodeSinglePath(path), isClosed);
  return decodePaths(result);
}

export function simplifyPaths(paths: number[][][], epsilon: number, isClosed: boolean): number[][][] {
  const w = getWasm();
  const result = w.simplify(encodePaths(paths), epsilon, isClosed);
  return decodePaths(result);
}

export function rdpSimplify(paths: number[][][], epsilon: number): number[][][] {
  const w = getWasm();
  const result = w.rdp_simplify(encodePaths(paths), epsilon);
  return decodePaths(result);
}

export function pointInPolygon(px: number, py: number, polygon: number[][]): number {
  const w = getWasm();
  return w.pip_test(px, py, encodeSinglePath(polygon));
}

export function pathArea(paths: number[][][]): number {
  const w = getWasm();
  return w.path_area(encodePaths(paths));
}

export function singlePathArea(path: number[][]): number {
  const w = getWasm();
  return w.single_area(encodeSinglePath(path));
}

export function isPositivePath(path: number[][]): boolean {
  const w = getWasm();
  return w.is_positive_path(encodeSinglePath(path));
}

export function getBounds(paths: number[][][]): { left: number; top: number; right: number; bottom: number } {
  const w = getWasm();
  const r = w.bounds(encodePaths(paths));
  return { left: r[0], top: r[1], right: r[2], bottom: r[3] };
}

export function makeEllipse(cx: number, cy: number, rx: number, ry: number, steps: number = 0): number[][] {
  const w = getWasm();
  return decodeSinglePath(w.make_ellipse(cx, cy, rx, ry, steps));
}

export function makeStar(cx: number, cy: number, outerR: number, innerR: number, points: number): number[][] {
  const w = getWasm();
  return decodeSinglePath(w.make_star(cx, cy, outerR, innerR, points));
}

export interface PolyTreeNode {
  polygon: number[][];
  is_hole: boolean;
  depth: number;
  children: PolyTreeNode[];
}

export function polyTreeOp(
  clipType: ClipType, fillRule: FillRule,
  subjects: number[][][], clips: number[][][],
): PolyTreeNode {
  const w = getWasm();
  const json = w.polytree_op(clipType, fillRule, encodePaths(subjects), encodePaths(clips));
  return JSON.parse(json);
}
