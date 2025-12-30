# Orbit ↔ Mole Integration Schemas (Draft)

These schemas are the shared contract for interchange between Orbit (Rust) and Mole (Bash/Go).

## Project Metadata (`.mole/projects.json`, `.orbit/index.json` compatible)

```jsonc
{
  "version": "0.1",
  "root": "/Users/alice/Workspace",
  "generated_at": "2025-01-05T12:34:56Z",
  "projects": [
    {
      "path": "apps/api",
      "kind": "active_standalone", // enum
      "pinned": true,
      "latest_mtime": "2025-01-04T18:22:10Z",
      "size_bytes": 123456789,
      "artifact_count": 3,
      "has_git": true,
      "has_rust": true,
      "has_node": false,
      "has_python": false,
      "fingerprint": "b3:abcd1234..." // optional
    }
  ]
}
```

### Kind enum
- `active_standalone`
- `standalone`
- `experimental`
- `backup_duplicate`
- `vendor_third_party`
- `unknown`

## System Metrics Snapshot (`.orbit/metrics.json`)

```jsonc
{
  "version": "0.1",
  "captured_at": "2025-01-05T12:34:56Z",
  "cpu": { "usage_pct": 42.1, "load1": 0.82, "load5": 1.05, "load15": 1.23, "cores": 8 },
  "memory": { "used_gb": 14.2, "total_gb": 24.0, "swap_used_gb": 0.3, "swap_total_gb": 4.0 },
  "disk": { "used_gb": 303.4, "total_gb": 512.0, "read_mb_s": 2.1, "write_mb_s": 18.3 },
  "network": { "rx_mb_s": 3.2, "tx_mb_s": 0.8 },
  "battery": { "level_pct": 100, "health": "normal", "cycles": 423, "temperature_c": 40.0 }
}
```

## Safety / Whitelist (`.orbit/whitelist.json`, `.mole/whitelist.json`)

```jsonc
{
  "version": "0.1",
  "paths": [
    "/Users/alice/Workspace/critical-app",
    "/Users/alice/Library/Application Support"
  ]
}
```

## Notes
- All timestamps ISO-8601 (UTC).
- Numbers are in base units; renderers can format (GB/MB).
- Files are append-safe via atomic write/rename.
- Backwards compatibility: tolerate unknown fields for forward schema evolution.

## Pins / Focus (`~/.orbit/focus.json`)

```jsonc
{
  "version": 1,
  "pinned": [
    "/Users/alice/Workspace/critical-app",
    "/Users/alice/Workspace/service-b"
  ]
}
```

## Session (`~/.orbit/session.json`)

```jsonc
{
  "version": 1,
  "root": "/Users/alice/Workspace",
  "lens": "projects", // or "space"
  "search": "api",
  "selections": [
    "/Users/alice/Workspace/apps/api",
    "/Users/alice/Workspace/apps/web"
  ],
  "palette": { "open": false, "cursor": 0 },
  "high_contrast": true
}
```

## Shared Index (`~/.orbit/index.json`)

```jsonc
{
  "version": 1,
  "scanned_at": "2025-01-05T12:34:56Z",
  "root": "/Users/alice/Workspace",
  "projects": [
    {
      "path": "/Users/alice/Workspace/apps/api",
      "name": "api",
      "kind": "active_standalone", // enum from Project Metadata section
      "fingerprint": "b3:abcd1234",
      "pinned": true,
      "dupe_group": "grp-1", // nullable
      "size_bytes": 123456789,
      "mtime": "2025-01-04T18:22:10Z",
      "stale_artifacts": ["node_modules", "target"]
    }
  ],
  "space": {
    "entries": [
      {
        "path": "/Users/alice/Workspace/apps",
        "size_bytes": 456789123,
        "is_dir": true,
        "children": [
          "/Users/alice/Workspace/apps/api",
          "/Users/alice/Workspace/apps/web"
        ],
        "mtime": "2025-01-04T18:22:10Z",
        "atime": "2025-01-04T18:22:10Z"
      }
    ],
    "large_files": [
      {
        "path": "/Users/alice/Workspace/apps/api/logs/huge.log",
        "size_bytes": 234567890,
        "mtime": "2025-01-03T10:00:00Z",
        "atime": "2025-01-03T10:00:00Z"
      }
    ]
  }
}
```
