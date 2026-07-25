# Conventions
- Rust IPC returns CmdResult<T>; shared read-heavy state uses Arc<RwLock<T>>; serde views use camelCase and must mirror TypeScript.
- Register every new IPC command in Tauri generate_handler and mirror it through TS type, command wrapper, hook, and IPC docs.
- Frontend follows FSD boundaries; shared helpers belong in shared/lib or shared/ui. TanStack hooks use central query keys.
- Use semantic MUI palette tokens and direct-path MUI icon imports.
- Before symbol edits use Serena find_symbol plus find_referencing_symbols; use text search for non-code or semantic misses.
- Treat Graphify as a candidate/impact graph, not a clone detector. Verify helper equivalence and references with Serena before moving code to a shared module.
- Repeated trading and loss-prevention changes require risk-guard review and living-document updates.
