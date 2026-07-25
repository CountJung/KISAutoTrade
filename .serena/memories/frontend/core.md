# Frontend core
- API boundary: src/api/types.ts, commands.ts, hooks.ts, queryKeys.ts; shared API surfaces under src/shared/api.
- Feature-Sliced layers: entities, features, widgets, pages; public index exports preserve layer boundaries.
- User-visible backend state/settings should have matching UI unless it is purely internal infrastructure.