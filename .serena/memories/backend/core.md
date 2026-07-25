# Backend core
- Primary orchestration: src-tauri/src/lib.rs; IPC commands under src-tauri/src/commands; API clients under src-tauri/src/api; trading domain under src-tauri/src/trading.
- KIS/Toss authentication, TR IDs, provider fields, and rate limits must be verified from official sources, never guessed.
- Internal structs must not leak snake_case through axum; use camelCase view structs or explicit JSON keys.