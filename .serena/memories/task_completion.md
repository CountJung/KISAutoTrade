# Task completion
- Always run: cd src-tauri && cargo check; then npx tsc --noEmit, npm run check:graphify, and npm run check:project-map. Resolve warnings, not only errors.
- Run npm run check:fsd for frontend boundary changes and npm run check:project-map when files/modules change.
- Run focused Playwright or npm run test:e2e for visual or interaction-risk UI changes.
- Update living docs triggered by the change: docs/project-map.md, docs/ipc-commands.md, todo.md, and relevant .github/skills files.
- Review the final diff for secrets, duplicated helpers, files over 1000 lines, polling/cache/listener/daemon risk, and broker/account-scope drift.
- After helper moves or symbol deletions run npm run graphify:refresh and npm run check:graphify; use project_mapper when files or module boundaries changed.
