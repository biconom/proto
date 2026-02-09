# Protobuf Style Guide

This guide defines the comprehensive style and best practices for creating data models (`.proto`) and their documentation.

## 1. General Principles
- **Contract-First**: We define the API and data structures strictly in `.proto` files first.
- **Naming Protocol**: Use `snake_case` for field names and `CamelCase` for message and service names.
- **Package structure**: 
    - `[package].types`: Domain models.
    - `[package].client.[domain]`: Client-facing services.
    - `[package].admin.[domain]`: Admin-facing services.
- **Microservice Ready**: Usage of `oneof` for IDs and consistent `List` wrappers enables flexible and forward-compatible service interfaces.

## 2. Protocol Buffers (.proto) Structure

### 2.1. File Header
Always strict syntax and package definition.
```protobuf
syntax = "proto3";

package myproject.types;

import "google/protobuf/timestamp.proto";
import "google/protobuf/empty.proto"; // If used
```

### 2.3. Service Definitions
- **Empty Requests/Responses**: Do NOT define empty messages like `message MyRequest {}`. Use `google.protobuf.Empty`.
    - *Good*: `rpc Ping(google.protobuf.Empty) returns (Pong);`
    - *Bad*: `rpc Ping(PingRequest) returns (Pong); message PingRequest {}`

### 2.2. Message Structure
The main entity message should act as a namespace for its related types (Ids, Lists, Enums).

#### 2.2.1. Identification (Inner `Id` Message)
If the entity can be referenced by ID, define a nested `Id` message. Use `oneof` to support future alternative IDs (e.g., searching by name vs numeric ID).

```protobuf
message MyEntity {
    message Id {
        oneof identifier {
            string id = 1;      // Primary UUID or unique ID
            string slug = 2;    // Alternative text identifier (handle)
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
        string next_page_token = 2; // For pagination support
    }
}
```

#### 2.2.3. Enums and Wrapper Messages
- **UNSPECIFIED Rule**: The first element (0) MUST be `UNSPECIFIED`.
- **C++ Scoping Rule**: In Proto3, enum values are in the scope of the parent message. To avoid name collisions (e.g., two enums having `UNSPECIFIED`), always wrap enums in a sub-message.

*Example (Wrapped - MANDATORY for multiple enums in one file):*
```protobuf
message MyEntity {
    message Status {
        enum Id {
            UNSPECIFIED = 0;
            PENDING = 1;
            ACTIVE = 2;
        }
    }
    
    Status.Id status = 1;
}
```

## 3. Documentation 

Every model requires a Markdown file with the same name (e.g., `community.proto` -> `community.md`).

```markdown
# Model: [EntityName]

## 1. Description
[Short, high-level description.]

## 2. Business Purpose
[Why does this exist?]

## 3. Key Fields
[Explanation of important fields.]

## 4. Use Cases
- **[Case 1]**: ...
```
