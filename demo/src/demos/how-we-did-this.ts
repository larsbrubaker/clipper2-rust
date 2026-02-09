export function init(container: HTMLElement) {
  container.innerHTML = `
    <div class="home-page" style="max-width: 860px; padding-bottom: 80px;">
      <div class="hero" style="margin-bottom: 32px;">
        <h1>How We <span>Ported Clipper2</span></h1>
        <p>
          The story of porting 10,000+ lines of C++ polygon clipping code to Rust in 48 hours
          with AI assistance.
        </p>
      </div>

      <div class="story-section">
        <h2>Why</h2>
        <p>
          <a href="https://www.matterhackers.com" target="_blank">MatterHackers</a> uses Clipper2
          extensively in production for 3D printing workflows &mdash; slicing, infill generation, support
          structures, and more. Angus Johnson's
          <a href="https://github.com/AngusJohnson/Clipper2" target="_blank">Clipper2</a> is the gold
          standard for polygon clipping, but our Rust-based tools needed either FFI bindings (with all
          the pain that entails) or a native port. We chose to port.
        </p>
      </div>

      <div class="story-section">
        <h2>The First Attempt</h2>
        <p class="date-tag">September 18, 2025</p>
        <p>
          We sat down to port Clipper2 to Rust using OpenAI's GPT and Claude, both running in
          <a href="https://cursor.com" target="_blank">Cursor</a>. We started ambitiously: a SQLite
          database cataloging all 790 C++ functions, Python scripts to analyze dependency graphs,
          implementation checklists, and pre-commit validation scripts.
        </p>
        <p>
          We made four passes at <code>core.rs</code> and got the basic types working &mdash;
          <code>Point64</code>, <code>PointD</code>, <code>Path64</code>, <code>Rect64</code>, and
          about 40 utility functions. Around 936 lines of production code.
        </p>
        <p>
          Then we looked at the engine.
        </p>
        <p>
          Clipper2's sweep-line engine (<code>clipper.engine.cpp</code>) is 3,163 lines of intricate C++.
          It uses linked lists threaded through arena-allocated nodes, with pointers being shuffled during
          a horizontal sweep. The rect-clipping module is another 1,027 lines. The offset module, 661 more.
        </p>
        <p>
          We made one more commit a week later, then&hellip; nothing. The project sat dormant for over four months.
          The complexity of the engine was daunting, and the function-by-function tracking approach created more
          overhead than progress.
        </p>
        <div class="result-box result-bad">
          <strong>Result:</strong> ~10,000 lines of scaffolding and tooling, but only <code>core.rs</code>
          and <code>version.rs</code> actually ported.
        </div>
      </div>

      <div class="story-section">
        <h2>The Breakthrough</h2>
        <p class="date-tag">February 5&ndash;7, 2026</p>
        <p>
          We came back with a different tool and a different approach. The tool was
          <a href="https://docs.anthropic.com/en/docs/claude-code" target="_blank">Claude Code</a>
          (Anthropic's CLI agent, running Opus 4.6) alongside Cursor with Claude. The approach was
          fundamentally different: instead of tracking individual functions in a database, we worked
          <strong>phase by phase</strong>, with the AI reading the C++ source directly and writing
          complete Rust modules.
        </p>
        <p>We wrote strict rules:</p>
        <ul>
          <li>No stubs. No <code>todo!()</code>. No <code>unimplemented!()</code>. Every function complete.</li>
          <li>Exact behavioral match with C++. Same algorithms, same precision, same edge cases.</li>
          <li>Tests required for everything. Write the test, then write the code.</li>
          <li>Dependency-ordered: before porting any function, verify all its dependencies exist.</li>
        </ul>
      </div>

      <div class="timeline">
        <h2>The 48-Hour Timeline</h2>
        <div class="timeline-track">
          <div class="timeline-event">
            <div class="timeline-dot"></div>
            <div class="timeline-content">
              <div class="timeline-date">Wed Feb 5, 6:57 PM</div>
              <div class="timeline-title">Phases 1&ndash;6 Land</div>
              <div class="timeline-desc">
                Core types, rect-clipping, full sweep-line engine, helper functions, public API layer.
                <strong>9,800 lines</strong> of new Rust in a single commit.
              </div>
            </div>
          </div>
          <div class="timeline-event">
            <div class="timeline-dot"></div>
            <div class="timeline-content">
              <div class="timeline-date">Wed Feb 5, 9:16 PM</div>
              <div class="timeline-title">Phase 7: Offset Module</div>
              <div class="timeline-desc">
                Polygon inflation/deflation with all join types. 1,952 more lines. September's entire
                output surpassed in a single evening.
              </div>
            </div>
          </div>
          <div class="timeline-event">
            <div class="timeline-dot"></div>
            <div class="timeline-content">
              <div class="timeline-date">Thu Feb 6, Morning</div>
              <div class="timeline-title">Phases 8&ndash;10</div>
              <div class="timeline-desc">
                Minkowski sum/difference, public convenience API, utility modules (SVG, timer, colors, file I/O).
              </div>
            </div>
          </div>
          <div class="timeline-event timeline-event-bug">
            <div class="timeline-dot" style="background: var(--clip-stroke);"></div>
            <div class="timeline-content">
              <div class="timeline-date">Thu Feb 6, 11:09 AM</div>
              <div class="timeline-title">Bug #1: Scanline Heap</div>
              <div class="timeline-desc">
                C++ uses <code>std::priority_queue</code> (max-heap). We used a min-heap. One-line fix,
                but it caused intersections and differences to return empty results.
              </div>
            </div>
          </div>
          <div class="timeline-event timeline-event-bug">
            <div class="timeline-dot" style="background: var(--clip-stroke);"></div>
            <div class="timeline-content">
              <div class="timeline-date">Thu Feb 6, 7:03 PM</div>
              <div class="timeline-title">Bug #2: Intersection Truncation</div>
              <div class="timeline-desc">
                C++ <code>static_cast&lt;int64_t&gt;</code> truncates. Rust's <code>.round() as i64</code> rounds.
                A vertex at 149.7 became 150 instead of 149. Six polygon tests failed.
              </div>
            </div>
          </div>
          <div class="timeline-event timeline-event-bug">
            <div class="timeline-dot" style="background: var(--clip-stroke);"></div>
            <div class="timeline-content">
              <div class="timeline-date">Thu Feb 6, 9:37 PM</div>
              <div class="timeline-title">Bug #3: Horizontal Joins</div>
              <div class="timeline-desc">
                Three subtle differences in horizontal edge merging broke PolyTree hierarchy.
                Required line-by-line C++ vs Rust comparison. Nine tests fixed at once.
              </div>
            </div>
          </div>
          <div class="timeline-event">
            <div class="timeline-dot" style="background: var(--subject-stroke);"></div>
            <div class="timeline-content">
              <div class="timeline-date">Thu Feb 6, 9:37 PM</div>
              <div class="timeline-title">444 Tests Passing</div>
              <div class="timeline-desc">
                Full Clipper2 feature set verified: boolean ops, offsetting, rect-clipping, Minkowski,
                simplification, PolyTree hierarchy, and all utility functions.
              </div>
            </div>
          </div>
          <div class="timeline-event">
            <div class="timeline-dot" style="background: var(--subject-stroke);"></div>
            <div class="timeline-content">
              <div class="timeline-date">Fri Feb 7</div>
              <div class="timeline-title">Ship It</div>
              <div class="timeline-desc">
                Examples, benchmarks, this WASM demo, GitHub Pages deployment,
                and published to crates.io as <code>clipper2-rust</code> v1.0.0.
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="story-section">
        <h2>The Good, The Bad, The Ugly</h2>

        <h3 class="subsection-header" style="color: var(--subject-stroke);">The Good</h3>
        <p>
          <strong>AI context window is the killer feature.</strong> The Clipper2 engine is 3,163 lines
          of dense C++ with dozens of interacting functions. Claude Code could read the entire C++ file,
          understand the relationships, and produce coherent Rust that preserved all the subtle invariants.
          It knew that C++ <code>OutPt*</code> became <code>usize</code> indices into a <code>Vec&lt;OutPt&gt;</code>,
          that <code>std::priority_queue</code> became <code>BinaryHeap</code>, and that C++ inheritance
          became Rust composition.
        </p>
        <p>
          <strong>Phase-by-phase was the right granularity.</strong> The September attempt tried to track
          790 individual functions. February worked in 12 phases, each producing a complete, testable module.
        </p>
        <p>
          <strong>Strict rules prevented drift.</strong> &ldquo;No stubs&rdquo; meant every committed line was
          production-ready. We never had a half-working module that &ldquo;mostly passed tests.&rdquo;
        </p>

        <h3 class="subsection-header" style="color: #b45309;">The Bad</h3>
        <p>
          <strong>C++ semantics hide in plain sight.</strong> Truncation vs. rounding, max-heap vs. min-heap,
          pointer mutation order &mdash; things that look identical in a casual review but produce completely
          different results.
        </p>
        <p>
          <strong>The first attempt over-engineered the process.</strong> A 790-function SQLite database with
          Python dependency analyzers was solving the wrong problem. The bottleneck wasn't tracking functions
          &mdash; it was understanding the C++ well enough to port them.
        </p>

        <h3 class="subsection-header" style="color: var(--clip-stroke);">The Ugly</h3>
        <p>
          <strong>The scanline heap bug</strong> produced symptoms that looked like fundamental algorithm failures.
          Everything was broken, but the cause was a single line. Lesson: when everything is broken, look for
          a single root cause first.
        </p>
        <p>
          <strong>C++ pointer patterns don't translate cleanly.</strong> The engine uses extensive pointer-based
          linked lists mutated in place. Rust's ownership model forbids this. The arena-allocation solution
          is correct and safe, but makes side-by-side comparison harder.
        </p>
      </div>

      <div class="stats-row" style="margin-top: 32px; flex-wrap: wrap; gap: 24px; justify-content: center;">
        <div class="stat">
          <div class="stat-value">48h</div>
          <div class="stat-label">Time to Port</div>
        </div>
        <div class="stat">
          <div class="stat-value">12</div>
          <div class="stat-label">Phases</div>
        </div>
        <div class="stat">
          <div class="stat-value">~30k</div>
          <div class="stat-label">Lines Written</div>
        </div>
        <div class="stat">
          <div class="stat-value">444</div>
          <div class="stat-label">Tests Passing</div>
        </div>
        <div class="stat">
          <div class="stat-value">3</div>
          <div class="stat-label">Critical Bugs Fixed</div>
        </div>
        <div class="stat">
          <div class="stat-value">0</div>
          <div class="stat-label">Unsafe Blocks</div>
        </div>
      </div>
      <p style="margin-top: 16px; color: var(--text-secondary); font-size: 0.95rem;">
        The library is <strong>100% safe Rust</strong> &mdash; zero <code>unsafe</code> blocks,
        enforced at compile time with <code>#![forbid(unsafe_code)]</code>.
      </p>

      <div class="story-section" style="margin-top: 40px;">
        <h2>Lessons Learned</h2>
        <ol class="lessons-list">
          <li><strong>Match the tool to the task.</strong> The September attempt used AI as a code-completion
            assistant. February used AI as a full-context translation engine. Same underlying technology,
            radically different results.</li>
          <li><strong>Work at the right granularity.</strong> Don't track 790 functions individually.
            Don't try to port the whole library at once. Find the natural module boundaries and work phase
            by phase.</li>
          <li><strong>Strict rules scale.</strong> &ldquo;No stubs&rdquo; and &ldquo;exact C++ match&rdquo;
            eliminated an entire class of &ldquo;I'll fix it later&rdquo; bugs.</li>
          <li><strong>Port the tests too.</strong> Clipper2's upstream test suite, built from years of
            real-world bug reports, was essential. Without it, the three critical bugs might have shipped.</li>
          <li><strong>Don't over-engineer the process.</strong> Read C++, write Rust, run tests &mdash;
            that's the whole workflow. A database tracking 790 functions is a project about tracking,
            not about porting.</li>
        </ol>
      </div>

      <div class="about-section">
        <h2>Credits</h2>
        <p>
          <a href="https://www.angusj.com" target="_blank"><strong>Angus Johnson</strong></a> &mdash;
          Author of the original Clipper2 library. An extraordinary piece of engineering.
        </p>
        <p style="margin-top: 8px;">
          <a href="https://github.com/larsbrubaker" target="_blank"><strong>Lars Brubaker</strong></a> &mdash;
          Led the port, wrote the rules, debugged the hard ones.
        </p>
        <p style="margin-top: 8px;">
          <a href="https://www.matterhackers.com" target="_blank"><strong>MatterHackers</strong></a> &mdash;
          Sponsored the work.
        </p>
        <p style="margin-top: 8px;">
          <a href="https://www.anthropic.com/claude" target="_blank"><strong>Claude</strong></a> by Anthropic &mdash;
          AI that did the heavy lifting on translation.
        </p>
        <p style="margin-top: 8px;">
          <a href="https://cursor.com" target="_blank"><strong>Cursor</strong></a> &mdash;
          IDE used throughout both attempts.
        </p>
        <p style="margin-top: 24px;">
          <em>Read the full writeup:
          <a href="https://github.com/larsbrubaker/clipper2-rust/blob/main/HOW_WE_PORTED_CLIPPER2.md" target="_blank">
            HOW_WE_PORTED_CLIPPER2.md
          </a></em>
        </p>
      </div>
    </div>

    <style>
      .story-section {
        margin-top: 40px;
      }
      .story-section h2 {
        font-size: 20px;
        font-weight: 700;
        margin-bottom: 12px;
        letter-spacing: -0.3px;
      }
      .story-section p {
        font-size: 14px;
        color: var(--text-secondary);
        line-height: 1.7;
        max-width: 720px;
        margin-bottom: 12px;
      }
      .story-section ul, .story-section ol {
        font-size: 14px;
        color: var(--text-secondary);
        line-height: 1.7;
        max-width: 720px;
        padding-left: 24px;
        margin-bottom: 12px;
      }
      .story-section li {
        margin-bottom: 6px;
      }
      .story-section a {
        color: var(--accent);
        text-decoration: none;
      }
      .story-section a:hover {
        text-decoration: underline;
      }
      .story-section code {
        background: var(--surface-alt);
        padding: 1px 5px;
        border-radius: 3px;
        font-family: var(--font-mono);
        font-size: 12.5px;
      }
      .date-tag {
        display: inline-block;
        font-size: 11px !important;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: var(--text-muted) !important;
        margin-bottom: 8px !important;
      }
      .result-box {
        border-radius: var(--radius);
        padding: 14px 18px;
        font-size: 13px;
        line-height: 1.5;
        margin-top: 12px;
        max-width: 720px;
      }
      .result-bad {
        background: #fef2f2;
        border: 1px solid #fecaca;
        color: #991b1b;
      }
      .result-box code {
        background: rgba(0,0,0,0.06);
        padding: 1px 5px;
        border-radius: 3px;
        font-family: var(--font-mono);
        font-size: 12px;
      }
      .subsection-header {
        font-size: 15px;
        font-weight: 700;
        margin-top: 20px;
        margin-bottom: 8px;
      }
      .lessons-list {
        padding-left: 24px;
      }
      .lessons-list li {
        margin-bottom: 10px;
      }

      /* Timeline */
      .timeline {
        margin-top: 40px;
      }
      .timeline h2 {
        font-size: 20px;
        font-weight: 700;
        margin-bottom: 24px;
        letter-spacing: -0.3px;
      }
      .timeline-track {
        position: relative;
        padding-left: 28px;
      }
      .timeline-track::before {
        content: '';
        position: absolute;
        left: 7px;
        top: 4px;
        bottom: 4px;
        width: 2px;
        background: var(--border);
      }
      .timeline-event {
        position: relative;
        margin-bottom: 24px;
      }
      .timeline-dot {
        position: absolute;
        left: -28px;
        top: 4px;
        width: 16px;
        height: 16px;
        border-radius: 50%;
        background: var(--accent);
        border: 3px solid var(--bg);
        box-shadow: 0 0 0 2px var(--border);
      }
      .timeline-content {
        padding-left: 4px;
      }
      .timeline-date {
        font-size: 11px;
        font-weight: 600;
        color: var(--text-muted);
        text-transform: uppercase;
        letter-spacing: 0.3px;
        margin-bottom: 2px;
      }
      .timeline-title {
        font-size: 15px;
        font-weight: 700;
        color: var(--text);
        margin-bottom: 4px;
      }
      .timeline-desc {
        font-size: 13px;
        color: var(--text-secondary);
        line-height: 1.6;
        max-width: 600px;
      }
      .timeline-desc code {
        background: var(--surface-alt);
        padding: 1px 5px;
        border-radius: 3px;
        font-family: var(--font-mono);
        font-size: 11.5px;
      }
    </style>
  `;
}
