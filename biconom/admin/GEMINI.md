# Admin API Context

This directory (`biconom/admin`) contains the **API definitions (RPC Services)** for internal administration, back-office dashboards, and system management.

## Purpose vs Types
- **Unlike `biconom/types`** (which defines *data structures*), this directory defines **Management Services**.
- The focus is on **CRUD operations**, complex filtering, analytics, and privileged state changes.

## Style & Rules
1. **Administrative Scope**: Services here provide high-level control (e.g., `LedgerService`, `SystemSettings`).
2. **Proto + MD Pair**: Every `.proto` file **MUST** be paired with a `.md` file.
   - **Documentation Focus**: Clearly describe the side effects of actions and required permissions/roles.
3. **Imports**:
   - Reuse `biconom/types` entities for the data payload.
   - Admin APIs often return robust `List` wrappers defined in `types`.
4. **Naming**: Packages should follow the pattern `biconom.admin.[domain]`.
