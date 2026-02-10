// Canvas rendering engine with zoom, pan, and coordinate transforms

export interface DrawOptions {
  fill?: string;
  stroke?: string;
  lineWidth?: number;
  lineDash?: number[];
  alpha?: number;
  vertices?: boolean;
  vertexRadius?: number;
  vertexColor?: string;
  closed?: boolean;
}

const GRID_COLOR = '#e8eaed';
const GRID_LABEL_COLOR = '#a0a6b2';
const AXIS_COLOR = '#c5c9d1';

export class DemoCanvas {
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  private dpr: number;

  // Transform state
  private offsetX = 0;
  private offsetY = 0;
  private scale = 1;

  // Interaction state
  private isPanning = false;
  private panStartX = 0;
  private panStartY = 0;
  private panStartOffX = 0;
  private panStartOffY = 0;

  // Drag callback
  private onDragMove: ((wx: number, wy: number) => void) | null = null;
  private onDragStart: ((wx: number, wy: number) => void) | null = null;
  private onDragEnd: (() => void) | null = null;
  private onMouseMove: ((wx: number, wy: number) => void) | null = null;
  private isDragging = false;

  // Touch interaction state
  private touchDragging = false;
  private touchPanning = false;
  private lastTouchDist = 0;
  private lastTouchMidX = 0;
  private lastTouchMidY = 0;

  // Coordinate display callback
  coordDisplay: HTMLElement | null = null;

  // Stored bounds for re-fitting on resize
  private savedBounds: { left: number; top: number; right: number; bottom: number } | null = null;
  private onResizeRedraw: (() => void) | null = null;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d')!;
    this.dpr = window.devicePixelRatio || 1;
    this.resize();

    // Resize observer - re-fit bounds and redraw on resize
    const ro = new ResizeObserver(() => {
      this.resize();
      if (this.savedBounds) {
        this.fitBounds(this.savedBounds.left, this.savedBounds.top, this.savedBounds.right, this.savedBounds.bottom);
      }
      this.onResizeRedraw?.();
    });
    ro.observe(canvas.parentElement!);
  }

  /** Set a callback to be called when the canvas resizes */
  setResizeRedraw(fn: () => void) {
    this.onResizeRedraw = fn;
  }

  resize() {
    const parent = this.canvas.parentElement!;
    const w = parent.clientWidth;
    const h = parent.clientHeight;
    this.dpr = window.devicePixelRatio || 1;
    this.canvas.width = w * this.dpr;
    this.canvas.height = h * this.dpr;
    this.canvas.style.width = w + 'px';
    this.canvas.style.height = h + 'px';
    this.ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
  }

  get width() { return this.canvas.width / this.dpr; }
  get height() { return this.canvas.height / this.dpr; }

  /** Set view to show a world-space bounding box with padding */
  fitBounds(left: number, top: number, right: number, bottom: number, padding = 40) {
    this.savedBounds = { left, top, right, bottom };
    const worldW = right - left;
    const worldH = bottom - top;
    if (worldW <= 0 || worldH <= 0) return;

    const canvasW = this.width - padding * 2;
    const canvasH = this.height - padding * 2;
    if (canvasW <= 0 || canvasH <= 0) return; // not laid out yet

    this.scale = Math.min(canvasW / worldW, canvasH / worldH);
    const cx = (left + right) / 2;
    const cy = (top + bottom) / 2;
    this.offsetX = this.width / 2 - cx * this.scale;
    this.offsetY = this.height / 2 - cy * this.scale;
  }

  /** Convert screen coords to world coords */
  screenToWorld(sx: number, sy: number): [number, number] {
    return [
      (sx - this.offsetX) / this.scale,
      (sy - this.offsetY) / this.scale,
    ];
  }

  /** Convert world coords to screen coords */
  worldToScreen(wx: number, wy: number): [number, number] {
    return [
      wx * this.scale + this.offsetX,
      wy * this.scale + this.offsetY,
    ];
  }

  /** Enable pan (right/middle drag) and zoom (scroll) */
  enableInteraction(opts?: {
    onDragStart?: (wx: number, wy: number) => void;
    onDragMove?: (wx: number, wy: number) => void;
    onDragEnd?: () => void;
    onMouseMove?: (wx: number, wy: number) => void;
    redraw?: () => void;
  }) {
    this.onDragStart = opts?.onDragStart ?? null;
    this.onDragMove = opts?.onDragMove ?? null;
    this.onDragEnd = opts?.onDragEnd ?? null;
    this.onMouseMove = opts?.onMouseMove ?? null;
    const redraw = opts?.redraw ?? (() => {});

    this.canvas.addEventListener('wheel', (e) => {
      e.preventDefault();
      const rect = this.canvas.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;

      const zoomFactor = e.deltaY < 0 ? 1.1 : 1 / 1.1;
      const newScale = this.scale * zoomFactor;

      // Zoom towards mouse position
      this.offsetX = mx - (mx - this.offsetX) * (newScale / this.scale);
      this.offsetY = my - (my - this.offsetY) * (newScale / this.scale);
      this.scale = newScale;
      redraw();
    }, { passive: false });

    this.canvas.addEventListener('mousedown', (e) => {
      const rect = this.canvas.getBoundingClientRect();
      const sx = e.clientX - rect.left;
      const sy = e.clientY - rect.top;

      if (e.button === 1 || e.button === 2 || (e.button === 0 && e.altKey)) {
        // Pan
        this.isPanning = true;
        this.panStartX = sx;
        this.panStartY = sy;
        this.panStartOffX = this.offsetX;
        this.panStartOffY = this.offsetY;
        e.preventDefault();
      } else if (e.button === 0) {
        // Left click drag
        this.isDragging = true;
        const [wx, wy] = this.screenToWorld(sx, sy);
        this.onDragStart?.(wx, wy);
      }
    });

    this.canvas.addEventListener('mousemove', (e) => {
      const rect = this.canvas.getBoundingClientRect();
      const sx = e.clientX - rect.left;
      const sy = e.clientY - rect.top;
      const [wx, wy] = this.screenToWorld(sx, sy);

      if (this.isPanning) {
        this.offsetX = this.panStartOffX + (sx - this.panStartX);
        this.offsetY = this.panStartOffY + (sy - this.panStartY);
        redraw();
      } else if (this.isDragging) {
        this.onDragMove?.(wx, wy);
      } else {
        this.onMouseMove?.(wx, wy);
      }

      // Update coordinate display
      if (this.coordDisplay) {
        this.coordDisplay.textContent = `${Math.round(wx)}, ${Math.round(wy)}`;
      }
    });

    this.canvas.addEventListener('mouseup', (e) => {
      if (this.isPanning) {
        this.isPanning = false;
      }
      if (this.isDragging) {
        this.isDragging = false;
        this.onDragEnd?.();
      }
    });

    this.canvas.addEventListener('mouseleave', () => {
      this.isPanning = false;
      if (this.isDragging) {
        this.isDragging = false;
        this.onDragEnd?.();
      }
    });

    this.canvas.addEventListener('contextmenu', (e) => e.preventDefault());

    // ---- Touch events for mobile ----
    // If drag callbacks are provided, single-finger captures for dragging and
    // we set touch-action:none to block page scroll. Otherwise, single-finger
    // passes through to allow normal page scrolling while two-finger still
    // handles pan/zoom.
    const hasDragCallbacks = !!(opts?.onDragStart || opts?.onDragMove);

    if (hasDragCallbacks) {
      // Block all browser touch handling — we control everything
      this.canvas.style.touchAction = 'none';
    } else {
      // Allow single-finger page scrolling; we only capture two-finger gestures.
      // pinch-zoom is handled by our JS, so block browser's native pinch.
      this.canvas.style.touchAction = 'pan-x pan-y';
    }

    this.canvas.addEventListener('touchstart', (e) => {
      const rect = this.canvas.getBoundingClientRect();

      if (e.touches.length === 1) {
        if (hasDragCallbacks) {
          // Single finger: drag shapes (only when drag callbacks exist)
          e.preventDefault();
          const t = e.touches[0];
          const sx = t.clientX - rect.left;
          const sy = t.clientY - rect.top;
          const [wx, wy] = this.screenToWorld(sx, sy);
          this.touchDragging = true;
          this.touchPanning = false;
          this.onDragStart?.(wx, wy);
        }
        // No drag callbacks: let the touch pass through for page scrolling
      } else if (e.touches.length === 2) {
        // Two fingers: always capture for pan/zoom
        e.preventDefault();
        if (this.touchDragging) {
          this.touchDragging = false;
          this.onDragEnd?.();
        }
        this.touchPanning = true;
        const t0 = e.touches[0];
        const t1 = e.touches[1];
        const mx = (t0.clientX + t1.clientX) / 2 - rect.left;
        const my = (t0.clientY + t1.clientY) / 2 - rect.top;
        this.lastTouchMidX = mx;
        this.lastTouchMidY = my;
        this.lastTouchDist = Math.hypot(t1.clientX - t0.clientX, t1.clientY - t0.clientY);
      }
    }, { passive: false });

    this.canvas.addEventListener('touchmove', (e) => {
      const rect = this.canvas.getBoundingClientRect();

      if (e.touches.length === 1 && this.touchDragging) {
        // Single finger drag (only active when hasDragCallbacks)
        e.preventDefault();
        const t = e.touches[0];
        const sx = t.clientX - rect.left;
        const sy = t.clientY - rect.top;
        const [wx, wy] = this.screenToWorld(sx, sy);
        this.onDragMove?.(wx, wy);

        if (this.coordDisplay) {
          this.coordDisplay.textContent = `${Math.round(wx)}, ${Math.round(wy)}`;
        }
      } else if (e.touches.length === 2) {
        // Two-finger pan + pinch zoom
        e.preventDefault();
        if (!this.touchPanning) {
          // Transition from 1-finger drag to 2-finger pan
          if (this.touchDragging) {
            this.touchDragging = false;
            this.onDragEnd?.();
          }
          this.touchPanning = true;
          const t0 = e.touches[0];
          const t1 = e.touches[1];
          this.lastTouchMidX = (t0.clientX + t1.clientX) / 2 - rect.left;
          this.lastTouchMidY = (t0.clientY + t1.clientY) / 2 - rect.top;
          this.lastTouchDist = Math.hypot(t1.clientX - t0.clientX, t1.clientY - t0.clientY);
          return;
        }

        const t0 = e.touches[0];
        const t1 = e.touches[1];
        const mx = (t0.clientX + t1.clientX) / 2 - rect.left;
        const my = (t0.clientY + t1.clientY) / 2 - rect.top;
        const dist = Math.hypot(t1.clientX - t0.clientX, t1.clientY - t0.clientY);

        // Pan: translate by midpoint delta
        const dx = mx - this.lastTouchMidX;
        const dy = my - this.lastTouchMidY;
        this.offsetX += dx;
        this.offsetY += dy;

        // Pinch zoom: scale towards midpoint
        if (this.lastTouchDist > 0) {
          const zoomFactor = dist / this.lastTouchDist;
          const newScale = this.scale * zoomFactor;
          this.offsetX = mx - (mx - this.offsetX) * (newScale / this.scale);
          this.offsetY = my - (my - this.offsetY) * (newScale / this.scale);
          this.scale = newScale;
        }

        this.lastTouchMidX = mx;
        this.lastTouchMidY = my;
        this.lastTouchDist = dist;
        redraw();
      }
      // Single finger without drag: don't preventDefault, let page scroll
    }, { passive: false });

    this.canvas.addEventListener('touchend', (e) => {
      if (e.touches.length === 0) {
        if (this.touchDragging) {
          this.touchDragging = false;
          this.onDragEnd?.();
        }
        this.touchPanning = false;
      } else if (e.touches.length === 1 && this.touchPanning) {
        this.touchPanning = false;
      }
    });

    this.canvas.addEventListener('touchcancel', () => {
      if (this.touchDragging) {
        this.touchDragging = false;
        this.onDragEnd?.();
      }
      this.touchPanning = false;
    });
  }

  clear() {
    this.ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    this.ctx.clearRect(0, 0, this.width, this.height);
    this.ctx.fillStyle = '#ffffff';
    this.ctx.fillRect(0, 0, this.width, this.height);
  }

  /** Draw background grid */
  drawGrid() {
    const ctx = this.ctx;

    // Calculate grid spacing in world coordinates
    let gridStep = 50;
    const screenStep = gridStep * this.scale;
    if (screenStep < 30) gridStep *= 5;
    else if (screenStep < 60) gridStep *= 2;
    if (screenStep > 300) gridStep /= 5;
    else if (screenStep > 150) gridStep /= 2;

    const [wLeft, wTop] = this.screenToWorld(0, 0);
    const [wRight, wBottom] = this.screenToWorld(this.width, this.height);

    const startX = Math.floor(wLeft / gridStep) * gridStep;
    const startY = Math.floor(wTop / gridStep) * gridStep;

    ctx.save();

    // Grid lines
    ctx.strokeStyle = GRID_COLOR;
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    for (let x = startX; x <= wRight; x += gridStep) {
      const [sx] = this.worldToScreen(x, 0);
      ctx.moveTo(sx, 0);
      ctx.lineTo(sx, this.height);
    }
    for (let y = startY; y <= wBottom; y += gridStep) {
      const [, sy] = this.worldToScreen(0, y);
      ctx.moveTo(0, sy);
      ctx.lineTo(this.width, sy);
    }
    ctx.stroke();

    // Axis lines (x=0, y=0)
    ctx.strokeStyle = AXIS_COLOR;
    ctx.lineWidth = 1;
    ctx.beginPath();
    const [axisX] = this.worldToScreen(0, 0);
    const [, axisY] = this.worldToScreen(0, 0);
    ctx.moveTo(axisX, 0); ctx.lineTo(axisX, this.height);
    ctx.moveTo(0, axisY); ctx.lineTo(this.width, axisY);
    ctx.stroke();

    // Grid labels
    ctx.fillStyle = GRID_LABEL_COLOR;
    ctx.font = '10px system-ui, sans-serif';
    ctx.textAlign = 'center';
    for (let x = startX; x <= wRight; x += gridStep) {
      if (x === 0) continue;
      const [sx, sy] = this.worldToScreen(x, 0);
      ctx.fillText(String(Math.round(x)), sx, Math.min(Math.max(sy + 14, 14), this.height - 4));
    }
    ctx.textAlign = 'right';
    for (let y = startY; y <= wBottom; y += gridStep) {
      if (y === 0) continue;
      const [sx, sy] = this.worldToScreen(0, y);
      ctx.fillText(String(Math.round(y)), Math.min(Math.max(sx - 6, 30), this.width - 4), sy + 3);
    }

    ctx.restore();
  }

  /** Draw closed paths (polygons) */
  drawPaths(paths: number[][][], opts: DrawOptions = {}) {
    if (!paths.length) return;
    const ctx = this.ctx;
    ctx.save();
    if (opts.alpha !== undefined) ctx.globalAlpha = opts.alpha;
    if (opts.lineDash) ctx.setLineDash(opts.lineDash);

    for (const path of paths) {
      if (path.length < 2) continue;
      ctx.beginPath();
      const [sx, sy] = this.worldToScreen(path[0][0], path[0][1]);
      ctx.moveTo(sx, sy);
      for (let i = 1; i < path.length; i++) {
        const [px, py] = this.worldToScreen(path[i][0], path[i][1]);
        ctx.lineTo(px, py);
      }
      if (opts.closed !== false) ctx.closePath();

      if (opts.fill) {
        ctx.fillStyle = opts.fill;
        ctx.fill(opts.fill?.includes('evenodd') ? 'evenodd' : 'nonzero');
      }
      if (opts.stroke) {
        ctx.strokeStyle = opts.stroke;
        ctx.lineWidth = opts.lineWidth ?? 1.5;
        ctx.stroke();
      }
    }

    // Draw vertices
    if (opts.vertices) {
      const r = opts.vertexRadius ?? 3;
      ctx.fillStyle = opts.vertexColor ?? opts.stroke ?? '#333';
      for (const path of paths) {
        for (const pt of path) {
          const [px, py] = this.worldToScreen(pt[0], pt[1]);
          ctx.beginPath();
          ctx.arc(px, py, r, 0, Math.PI * 2);
          ctx.fill();
        }
      }
    }

    ctx.globalAlpha = 1;
    ctx.setLineDash([]);
    ctx.restore();
  }

  /** Draw open paths (lines) */
  drawOpenPaths(paths: number[][][], opts: DrawOptions = {}) {
    this.drawPaths(paths, { ...opts, closed: false });
  }

  /** Draw a rectangle outline */
  drawRect(left: number, top: number, right: number, bottom: number, opts: DrawOptions = {}) {
    const ctx = this.ctx;
    const [sx, sy] = this.worldToScreen(left, top);
    const [ex, ey] = this.worldToScreen(right, bottom);
    ctx.save();
    if (opts.alpha !== undefined) ctx.globalAlpha = opts.alpha;
    if (opts.lineDash) ctx.setLineDash(opts.lineDash);

    if (opts.fill) {
      ctx.fillStyle = opts.fill;
      ctx.fillRect(sx, sy, ex - sx, ey - sy);
    }
    if (opts.stroke) {
      ctx.strokeStyle = opts.stroke;
      ctx.lineWidth = opts.lineWidth ?? 1.5;
      ctx.strokeRect(sx, sy, ex - sx, ey - sy);
    }

    ctx.globalAlpha = 1;
    ctx.setLineDash([]);
    ctx.restore();
  }

  /** Draw a single point */
  drawPoint(wx: number, wy: number, radius: number, color: string) {
    const [sx, sy] = this.worldToScreen(wx, wy);
    this.ctx.fillStyle = color;
    this.ctx.beginPath();
    this.ctx.arc(sx, sy, radius, 0, Math.PI * 2);
    this.ctx.fill();
  }

  /** Draw text at world coordinates */
  drawText(text: string, wx: number, wy: number, opts?: { color?: string; font?: string; align?: CanvasTextAlign }) {
    const [sx, sy] = this.worldToScreen(wx, wy);
    this.ctx.fillStyle = opts?.color ?? '#333';
    this.ctx.font = opts?.font ?? '12px system-ui';
    this.ctx.textAlign = opts?.align ?? 'left';
    this.ctx.fillText(text, sx, sy);
  }
}
