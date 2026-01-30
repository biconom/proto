# Client API Context

This directory (`biconom/client`) contains the **API definitions (RPC Services)** exposed to end-users (frontend web, mobile apps).

## Purpose vs Types
- **Unlike `biconom/types`** (which defines *data structures* and *state* models), this directory defines **Actions** and **Interfaces**.
- Files here define `service ServiceName { rpc Method(...) }` and their specific Request/Response messages.

## Style & Rules
1. **Service-Oriented**: Each file usually corresponds to a domain service (e.g., `AuthService`, `AccountService`).
2. **Proto + MD Pair**: Every `.proto` file **MUST** be paired with a `.md` file explaining the service capabilities.
   - **Documentation Focus**: Describe the **Methods**, **Permissions**, and **Flows** (how to use the API), rather than just listing fields.
3. **Imports**:
   - Heavily refer to reusable models from `biconom/types`.
   - **Avoid** redefining core entities (like `Currency`, `User`) inside the client protos; import them instead.
4. **Naming**: Packages should follow the pattern `biconom.client.[domain]`.
