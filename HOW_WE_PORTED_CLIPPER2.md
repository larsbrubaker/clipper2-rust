# How We Ported Clipper2 to Rust

*A story about porting 10,000+ lines of C++ polygon clipping code to Rust in 48 hours with AI assistance.*

## Why

[MatterHackers](https://www.matterhackers.com) uses Clipper2 extensively in production for 3D printing workflows — slicing, infill generation, support structures, and more. Angus Johnson's [Clipper2](https://github.com/AngusJohnson/Clipper2) is the gold standard for polygon clipping, but our Rust-based tools needed either FFI bindings (with all the pain that entails) or a native port.

We chose to port. What follows is how that actually went.

## The First Attempt (September 2025)

On September 18, 2025, we sat down to port Clipper2 to Rust. We started ambitiously: a SQLite database cataloging all 790 C++ functions, Python scripts to analyze dependency graphs, an implementation checklist, pre-commit hooks to validate progress.

The tools we used were OpenAI's GPT and Claude, both running in Cursor, with manual commits throughout the day. We made four passes at `core.rs` and got the basic types working — `Point64`, `PointD`, `Path64`, `Rect64`, and about 40 utility functions. Around 936 lines of production code.

Then we looked at the engine.

Clipper2's sweep-line engine (`clipper.engine.cpp`) is 3,163 lines of intricate C++. It uses linked lists threaded through arena-allocated nodes, with pointers being shuffled during a horizontal sweep. The rect-clipping module is another 1,027 lines. The offset module, 661 more. Every module depends on the others in subtle ways.

We made one more commit a week later, improving the dependency database, and then... nothing. The project sat dormant for over four months. The complexity of the engine was daunting, and the function-by-function tracking approach felt like it was creating more overhead than progress.

**What we had after the first attempt:** ~10,000 lines of scaffolding and tooling, but only `core.rs` and `version.rs` actually ported. Lots of Python scripts. A very detailed SQLite database. Not much Rust.

## The Breakthrough (February 5-7, 2026)

On the evening of February 5th, we came back with a different tool and a different approach.

The tool was [Claude Code](https://docs.anthropic.com/en/docs/claude-code) (Anthropic's CLI agent, running Opus 4.6) alongside Cursor with Claude. The approach was fundamentally different: instead of tracking individual functions in a database, we worked **phase by phase**, with the AI reading the C++ source directly and writing complete Rust modules.

We wrote strict rules in `CLAUDE.md`:
- No stubs. No `todo!()`. No `unimplemented!()`. Every function complete.
- Exact behavioral match with C++. Same algorithms, same precision, same edge cases.
- Tests required for everything. Write the test, then write the code.
- Dependency-ordered: before porting any function, verify all its dependencies exist.

### Wednesday Evening (Feb 5)

The first massive commit landed at 6:57 PM: Phases 1 through 6 in a single push. Core types (enhanced from the September work), the entire rect-clipping module, the full sweep-line engine with all its data structures, 50+ helper functions, and the public API layer (Clipper64, ClipperD, PolyPath, PolyTree).

**9,800 lines of new Rust code.**

The key insight was that Claude Code could hold the full context of both the C++ source and the Rust codebase simultaneously. Instead of a human trying to understand a C++ function, translate it mentally, and type Rust — the AI read `clipper.engine.cpp`, understood the pointer-based linked list operations, and translated them to Rust's arena-based index pattern. It knew that C++ `OutPt*` became `usize` indices into a `Vec<OutPt>`, that `std::priority_queue` became `BinaryHeap`, that C++ inheritance became Rust composition.

By 9:16 PM, Phase 7 (the offset module) was done. Another 1,952 lines. The September attempt's entire output was surpassed in a single evening.

### Thursday (Feb 6)

The morning started with Minkowski sum/difference (Phase 8) and the public convenience API (Phase 9). Then the bugs started showing up — and this is where the story gets interesting.

**Bug #1: The Scanline Heap.** The Clipper2 engine processes scanlines from top to bottom (highest Y first). C++ uses `std::priority_queue`, which is a max-heap. Our Rust port used `BinaryHeap::new()` with `Reverse` wrapper — a min-heap. This single reversal caused intersection and difference operations to return empty results, offset shrink to produce nothing, and Minkowski operations to hang. The fix was one line, but finding it required understanding the entire sweep-line algorithm.

**Bug #2: Intersection Truncation.** C++ `static_cast<int64_t>(value)` truncates toward zero. Rust's `.round() as i64` rounds to nearest. For polygon clipping, this matters: a vertex at 149.7 should become 149, not 150. Six polygon test cases failed until we changed the intersection calculation to truncate.

**Bug #3: Horizontal Join Mechanism.** The trickiest one. In the C++ code, `ConvertHorzSegsToJoins` walks horizontal edges and marks join points for the PolyTree hierarchy. Our port had three subtle differences: it set a flag too early, sorted in the wrong direction, and didn't persist loop state correctly. Nine polytree tests were failing. Fixing all three issues at once brought the test suite from 435 to 444 tests, all passing.

By the end of Thursday, we had 444 tests passing across the full Clipper2 feature set: boolean operations, offsetting, rect-clipping, Minkowski, path simplification, PolyTree hierarchy, and all the utility functions.

### Friday Morning (Feb 7)

The finish line. We added four examples, six Criterion benchmarks, removed the C++ reference source (no longer needed), built an interactive WASM demo with 8 pages, set up GitHub Pages deployment, and published to crates.io as `clipper2-rust` v1.0.0.

## The Good

**AI context window is the killer feature.** The Clipper2 engine is 3,163 lines of dense C++ with dozens of interacting functions. A human porter needs to hold all of that in their head while simultaneously thinking in Rust idioms. Claude Code could read the entire C++ file, understand the relationships between functions, and produce a coherent Rust translation that preserved all the subtle invariants.

**Phase-by-phase was the right granularity.** The September attempt tried to track 790 individual functions. The February attempt worked in 12 phases, each producing a complete, testable module. This matched how the code actually fit together — you can't port `execute_clipper` without `build_intersect_list` and `process_horz_segment`, so porting the whole engine as a phase made sense.

**Strict rules prevented drift.** "No stubs" sounds harsh, but it meant every committed line was production-ready. We never had a half-working module that "mostly passed tests." Either a phase was complete or it wasn't committed.

**Tests caught real bugs.** All three critical bugs were caught by tests, not by manual inspection. The C++ test data files (ported from the upstream test suite) were invaluable — they encoded edge cases from years of bug reports.

## The Bad

**C++ semantics hide in plain sight.** Truncation vs. rounding, max-heap vs. min-heap, pointer mutation order — these are things that look identical in a casual code review but produce completely different results. Each of the three critical bugs was a single-line difference that cascaded into major failures.

**The first attempt over-engineered the process.** Building a 790-function SQLite database with Python dependency analyzers was solving the wrong problem. The bottleneck wasn't tracking which functions to port — it was understanding the C++ well enough to port them correctly. The database gave a false sense of progress.

**Debugging AI-written code is different.** When a human writes buggy code, they usually have an intuition about where the bug might be. When AI writes 3,000 lines and a test fails, you need systematic debugging. The `fix-test-failures` agent approach (instrument, log, narrow down) worked well, but it's a different skill than traditional debugging.

## The Ugly

**The scanline heap bug** produced symptoms that looked like fundamental algorithm failures. Intersections returned empty. Differences returned empty. Minkowski hung. It would have been easy to conclude "the engine port is fundamentally broken" rather than looking for a one-line fix. The lesson: when everything is broken, look for a single root cause first.

**The horizontal join bug** required reading both C++ and Rust line-by-line, comparing behavior at each step. There's no shortcut for this kind of debugging — you need to understand exactly what the C++ does and exactly what the Rust does and find where they diverge.

**C++ pointer patterns don't translate cleanly.** The Clipper2 engine uses extensive pointer-based linked lists where nodes are mutated in place. Rust's ownership model forbids this. The solution (arena allocation with index-based references) is correct and safe, but it means the Rust code looks different from the C++ even when it does the same thing. Every `*out_pt = ...` in C++ becomes `self.out_pt_list[idx].field = ...` in Rust. This makes side-by-side comparison harder.

## By the Numbers

| Metric | Value |
|--------|-------|
| C++ source lines (engine + offset + rectclip + core + minkowski) | ~6,000 |
| Rust source lines (production code) | ~12,000+ |
| Rust test lines (unit + integration) | ~6,000+ |
| Total test functions | 444 |
| Critical bugs found during port | 3 |
| Time for first attempt (Sept 2025) | 1 day, then abandoned |
| Time for successful port (Feb 2026) | ~48 hours |
| Phases in the port plan | 12 |
| Time from "git init" to crates.io publish | 48 hours (Feb 5-7) |

## Lessons Learned

1. **Match the tool to the task.** The September attempt used AI as a code-completion assistant. The February attempt used AI as a full-context translation engine. Same underlying technology, radically different results.

2. **Work at the right granularity.** Don't track 790 functions individually. Don't try to port the whole library at once. Find the natural module boundaries and work phase by phase.

3. **Strict rules scale.** "No stubs" and "exact C++ match" sound rigid, but they eliminated an entire class of "I'll fix it later" bugs that never get fixed.

4. **Port the tests too.** Clipper2's upstream test suite, built from years of real-world bug reports, was essential. Without it, the three critical bugs might have shipped.

5. **Don't over-engineer the process.** A SQLite database tracking 790 functions is a project about tracking, not about porting. The February approach — read C++, write Rust, run tests — had almost no overhead.

## Credits

- **[Angus Johnson](https://www.angusj.com)** — Author of the original Clipper2 library. An extraordinary piece of engineering.
- **[Lars Brubaker](https://github.com/larsbrubaker)** — Led the port, wrote the rules, debugged the hard ones.
- **[MatterHackers](https://www.matterhackers.com)** — Sponsored the work.
- **[Claude](https://www.anthropic.com/claude) by Anthropic** — AI that did the heavy lifting on translation.
- **[Cursor](https://cursor.com)** — IDE used throughout both attempts.

---

*Questions? Open an issue on [the GitHub repo](https://github.com/larsbrubaker/clipper2-rust) or find us at [MatterHackers](https://www.matterhackers.com).*
