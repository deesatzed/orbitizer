# Orbit + Mole: The Unified Developer Toolkit for Local Workspaces

In the world of modern software engineering, our local disk is often a chaotic sprawl of project clones, build artifacts, and forgotten experiments. Today, we're excited to announce the full unification of **Orbit** and **Mole**—two powerful tools designed to bring order, speed, and deep insights to your local developer environment.

## One Workspace, Two Lenses

**Orbit** (Rust) and **Mole** (Go) are no longer just separate tools; they are now a synchronized ecosystem sharing a unified data layer.

- **Orbit** is your high-speed engine. Written in Rust, it specializes in massive disk walks, project census, and "Snapshotting" your pinned projects for safe-keeping.
- **Mole** is your interactive cockpit. A feature-rich TUI written in Go, it offers deep-cleaning, system monitoring, and an intuitive "Analyze" mode for surgical disk cleanup.

---

## 🚀 Key Features

### 1. Unified Project Management
Both tools now share a common configuration at `~/.orbit/`.
- **Shared Focus**: Pin a project in Orbit, and it's immediately "focused" in Mole.
- **Shared Session**: Navigate to a deep directory in Mole, search for a specific file, and toggle High Contrast mode—your state is persisted and ready for your next session, even if you switch tools.

### 2. Instant-On Performance
Waiting for a disk scan is a thing of the past. Mole now hydrates its "Overview" mode directly from Orbit's background-generated index. When you open Mole, your project sizes and metadata are already there.

### 3. Headless CI Power
With the new `orbit ci` command, you can integrate workspace health checks into your automated workflows. It runs a full census, exports reports (MD, JSON, CSV), and prepares the shared state for your local interactive sessions.

### 4. Safety-First Deletion
Mole's "Analyze" mode now features a multi-level safety net:
- **Trash Integration**: Deletions are moved to a timestamped trash folder instead of immediate `rm -rf`.
- **Infinite Undo**: Regret a delete? `Ctrl+Z` restores your files instantly.
- **Batch Actions**: Select multiple projects and perform mass exports, pins, or deletions with a single palette action (`x`).

---

## 🛠️ Use Cases

- **The "Node_Modules" Purge**: Use Mole's `mo purge` to instantly reclaim gigabytes from old JavaScript projects while keeping your "Focus" projects safe.
- **Workspace Census**: Run `orbit census` to get a high-level view of every project you've touched in the last 30 days.
- **The "High Contrast" Workspace**: For those working in bright environments or needing better accessibility, the `C` key toggles a high-contrast theme across the entire suite.
- **Project Snapshotting**: Before making risky changes, use `orbit snap` to create a point-in-time manifest of your pinned projects.

---

## 📦 Setting Up

### 1. Prerequisites
- **Rust** (for Orbit)
- **Go** (for Mole)

### 2. Installation
Clone the merged repository and build both targets:

```bash
# Build Orbit (Rust)
cargo build --release
cp target/release/orbit /usr/local/bin/

# Build Mole (Go)
cd Mole
go build -o analyze ./cmd/analyze
cp analyze /usr/local/bin/mo-analyze
```

---

## 🚦 Getting Started

### Step 1: Initialize your Workspace
Start by letting Orbit index your primary developer folder:
```bash
orbit ci --root ~/src --depth 4
```

### Step 2: Interactive Cleanup
Launch Mole's analyzer to find space-hogging artifacts:
```bash
mo-analyze --overview
```

### Step 3: Pin and Focus
In the Mole TUI, press `p` to pin a project. It will now appear at the top of your Orbit "Focus" list and be included in future snapshots.

### Step 4: System Health
Keep an eye on your hardware with Mole's live dashboard:
```bash
mo status
```

---

## 🔮 The Vision

We believe developer tools should be as fast as Rust and as interactive as Go. By unifying Orbit and Mole, we're building a toolkit that respects your disk space, your time, and your cognitive load.

**Happy Hacking!**
