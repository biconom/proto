# Protocol Buffers Style Guide for Biconom

This guide defines the comprehensive style and best practices for creating data models (`.proto`) and their documentation (`.md`) in the **Biconom** project.

## 1. General Principles
- **Contract-First**: We define the API and data structures strictly in `.proto` files first.
- **Microservice Ready**: Usage of `oneof` for IDs and consistent `List` wrappers enables flexible and forward-compatible service interfaces.
- **Documentation Driven**: Every `.proto` file **MUST** have a corresponding `.md` file describing its purpose, lifecycle, and relations.

## 2. Protocol Buffers (.proto) Structure

### 2.1. File Header
Always strict syntax and package definition.
```protobuf
syntax = "proto3";

package biconom.types; // Adjust package path if necessary (e.g. biconom.client.account)

import "google/protobuf/timestamp.proto";
// other imports
```

### 2.2. Message Structure
The main entity message should act as a namespace for its related types (Ids, Lists, Enums).

#### 2.2.1. Identification (Inner `Id` Message)
If the entity can be referenced by ID, define a nested `Id` message. Use `oneof` to support future alternative IDs (e.g., searching by name vs numeric ID).

```protobuf
message MyEntity {
    message Id {
        oneof identifier {
            uint32 id = 1;      // Primary numeric ID
            string slug = 2;    // Alternative text identifier
        }
    }
    // ...
}
```

#### 2.2.2. Lists (Inner `List` Message)
Always define a `List` message to encapsulate arrays. This makes RPC responses extensible (e.g. adding pagination metadata later).

```protobuf
message MyEntity {
    // ...
    message List {
        repeated MyEntity items = 1;
    }
}
```

#### 2.2.3. Enums and Wrapper Messages
- **UNSPECIFIED Rule**: The first element (0) MUST be `UNSPECIFIED`.
- **Packaging**: Complex enums should be wrapped in a message to avoid namespace pollution and provide context. The enum itself is named `Id` inside the wrapper.

*Example (Wrapped - Preferred for standard states):*
```protobuf
message MyEntity {
    // Wrapper message for related logic/enums
    message Status {
        enum Id {
            UNSPECIFIED = 0;
            ACTIVE = 1;
            ARCHIVED = 2;
        }
    }

    // Usage
    Status.Id status = 3;
}
```

*Example (Direct - Allowed for simple local flags):*
```protobuf
message MyEntity {
    enum Type {
        UNSPECIFIED = 0;
        PRIMARY = 1;
        SECONDARY = 2;
    }
}
```

#### 2.2.4. Standard Fields
Include timestamps for lifecycle tracking.
```protobuf
message MyEntity {
    // ... fields ...
    google.protobuf.Timestamp created_at = 10;
    google.protobuf.Timestamp updated_at = 11;
}
```

## 3. Documentation (.md) Structure

Every model requires a Markdown file with the same name (e.g., `community.proto` -> `community.md`).
Follow this strict section structure:

```markdown
# Модель: [EntityName]

## 1. Описание
[Short, high-level description of what this entity is.]

## 2. Назначение и решаемые задачи
[Why does this exist? What business problem does it solve?]
### Ключевые задачи:
- **[Task 1]**: ...
- **[Task 2]**: ...

## 3. Ключевые поля и концепции
[Explanation of important fields, especially logic for enums or complex types.]
- `field_name`: Description...

## 4. Сценарии использования (кейсы)
- **[Case 1]**: Description of a flow...
- **[Case 2]**: ...

## 5. Связи с другими моделями
- **[OtherEntity]**: Relation description (e.g., ownership, hierarchy).
```

## 4. Example: Full Template

```protobuf
// biconom/types/example.proto
syntax = "proto3";
package biconom.types;
import "google/protobuf/timestamp.proto";

message Example {
    message Id {
        oneof identifier {
            uint32 id = 1;
        }
    }
    message List {
        repeated Example items = 1;
    }
    message Status {
        enum Id {
            UNSPECIFIED = 0;
            PENDING = 1;
            DONE = 2;
        }
    }

    uint32 id = 1;
    Status.Id status = 2;
    google.protobuf.Timestamp created_at = 3;
}
```
