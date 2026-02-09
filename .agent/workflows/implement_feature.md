---
description: End-to-End workflow from abstract Idea to Production-Ready Code (Spec -> Proto)
---

# Feature Implementation Workflow

This workflow orchestrates the entire lifecycle of a feature, from vague idea to compiling code, using the specialized skills `spec-architect` and `grpc-designer`.

## 1. Design Phase (Spec Architect)
**Trigger**: User has an idea or requirement.
**Action**:
1. Run `/spec-architect [idea]` or manually invoke the skill.
2. Follow the 4-step process:
    - **Discovery**: Brainstorm and ask questions.
    - **Safety**: Apply "God Mode" constraints.
    - **Structuring**: Define where in `specs/domains/` this belongs.
    - **Writing**: Create/Update `specs/domains/[domain]/[file].md`.

## 2. Review Phase (Human Check)
**Action**:
1. Present the generated spec to the User.
2. Ask: "Does this match your vision? Are there any business risks missed?"
3. If changes needed -> Go back to Step 1.

## 3. Implementation Phase (gRPC Designer)
**Trigger**: Spec is approved.
**Action**:
1. Invoke `grpc-designer` skill.
2. **Map**: Translate the Specification Models to Protobuf Messages.
    - Use `specs/` as the blueprint.
    - Apply `style-guide.md` (Strict rules).
3. **Execute**:
    - Create/Update `.proto` files in `biconom/types` or `biconom/client`.
    - Create/Update `.md` documentation next to `.proto`.
4. **Compile**:
    - Run `cargo build` (or check `build.rs`) to ensure valid Proto syntax.

## 4. Final Verification
**Action**:
1. Check `build.rs` output.
2. Verify file structure matches strict package rules.
