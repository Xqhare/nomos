# Update README with CLI Help and Library Usage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Update `README.md` to match the updated CLI help page information and document the usage of the parser library.

**Architecture:** Update the `README.md` file in-place with comprehensive sections covering CLI help and library usage. Verify cargo test output.

**Tech Stack:** Markdown, Rust

---

### Task 1: Update README.md Content

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Replace README.md content with updated details**
  Rewrite the `README.md` file to include:
  - Welcome and Overview
  - File Syntax (List Markers, Tasks, Notes, Hierarchy, Dates, Tags, Dependencies)
  - CLI Configuration & Crawling Behavior (`config.json`, `nomos.json`, discovery fallback)
  - Library API usage with a Rust code snippet that compiles (using `no_run` attribute)

- [ ] **Step 2: Run cargo test to verify it compiles and doctests pass**
  Run: `cargo test`
  Expected: PASS

- [ ] **Step 3: Commit**
  Run:
  ```bash
  git add README.md
  git commit -m "docs(readme): sync CLI help page and add parser library usage example"
  ```
