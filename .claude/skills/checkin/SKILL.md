---
name: checkin
description: "Automates the full commit workflow: analyzes changes, writes commit message, stages files, commits, and handles pre-commit hook failures by fixing test failures until the commit succeeds. Use when the user wants to commit their changes."
---

# Checkin Skill

Automates the full commit workflow: analyzes changes, writes commit message, stages files, commits, and handles pre-commit hook failures by fixing test failures until the commit succeeds.

## Workflow

### Step 1: Analyze Changes

Run these commands in parallel to understand the current state:

```bash
git status
git diff
git diff --staged
```

From this analysis:
- Identify what files changed and why
- Determine which files should be staged (exclude secrets, generated files, etc.)

### Step 2: Stage and Commit

1. Stage relevant files using `git add`
2. Write a commit message following this format:
   - **Subject line**: Imperative mood, max 50 chars, no period (e.g., "Add user authentication" not "Added user authentication.")
   - **Body** (if needed): Blank line after subject, wrap at 72 chars, explain *why* not *what*
   - End with the co-author line
3. Commit using:

```bash
git commit -m "$(cat <<'EOF'
Commit message here.

Co-Authored-By: Claude <noreply@anthropic.com>
EOF
)"
```

### Step 3: Handle Pre-Commit Failures

If the commit fails due to pre-commit hooks:

1. **Identify the failure type** from the hook output
2. **For test failures**: Launch the `fix-test-failures` agent using the Task tool:
   ```
   Task(subagent_type="fix-test-failures", prompt="<paste the test failure output here>")
   ```
   This agent will autonomously diagnose and fix the failing tests.
3. **For lint/format failures**: Run `cargo fmt` and `cargo clippy --fix` to resolve
4. **After fixing**: Attempt the commit again

### Step 4: Iterate Until Success

Repeat Step 3 until the commit succeeds. Each iteration:
- Run the commit
- If it fails, fix the issues
- Try again

Use `--amend` only when:
- The previous commit attempt succeeded but hooks modified files that need including
- The HEAD commit was created in this session
- The commit has NOT been pushed to remote

Otherwise, create a new commit with the fixes.

### Step 5: Confirm Success

After a successful commit:
- Run `git status` to verify the commit succeeded
- Report the commit hash and summary to the user

## Important Notes

- Do NOT push to remote - the user will handle that
- Do NOT commit files that contain secrets (.env, credentials, etc.)
- Do NOT use `--no-verify` to bypass hooks
- Do NOT weaken tests to make them pass - fix the actual bugs

## Quality Commitment

**Fix every single error. No exceptions.**

When errors occur during the commit process:
- Do NOT skip errors or mark them as "known issues"
- Do NOT disable tests to make them pass
- Do NOT add workarounds that hide problems
- Do NOT give up after a few attempts

Take the hard path:
- Investigate root causes, not just symptoms
- Fix the underlying bug, even if it requires significant changes
- Ensure the fix is correct, not just passing
- Keep iterating until the code is genuinely right

Quality matters. Every error is an opportunity to make the code better. The easy path creates technical debt; the hard path creates code we can be proud of.
