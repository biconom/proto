# Biconom Types Directory Context

This directory (`biconom/types`) contains the core domain models of the Biconom platform.

## Style & Rules
All modifications and additions here **MUST** follow the rules defined in `.agent/rules/protobuf_style.md`.

### Quick Checklist for Contributors (AI & Human):
1. **Proto + MD Pair**: Never create a `.proto` without its `.md` documentation.
2. **Inner Types**: Always define `Id` and `List` inner messages for entities.
3. **Enums**:
   - Always start with `UNSPECIFIED = 0`.
   - Wrap complex state enums in a message (e.g. `message Status { enum Id { ... } }`).
   - Use `oneof` for ID fields where polymorphism might be needed (UUID vs Int vs String).

## Directory Structure
- Each file pairs with a markdown file.
- Do not group files into subfolders unless they represent a completely distinct subdomain that requires isolation (currently flat structure is preferred for core types).
