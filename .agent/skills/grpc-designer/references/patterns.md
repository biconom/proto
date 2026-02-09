# Protobuf Patterns

## 1. Wrapped Enums (Zero-Collision)
Always wrap enums in a message to avoid name collisions between multiple enums in the same package/file.

```protobuf
message Status {
    enum Id {
        UNSPECIFIED = 0;
        ACTIVE = 1;
        DELETED = 2;
    }
}
// Usage:
Status.Id status = 1;
```

## 2. Nested Identifiers (Flexibility)
Use `oneof` inside a nested `Id` message to allow future alternative lookups.

```protobuf
message User {
    message Id {
        oneof identifier {
            string id = 1;      // UUID
            string username = 2; // Handle
        }
    }
}
```

## 3. Pagination Pattern
Always include `List` message with pagination support.

```protobuf
message User {
    message List {
        repeated User items = 1;
        string next_page_token = 2;
    }
}
```

## 4. Money and Currency
Reuse common types (create `common.proto` if not exists).

```protobuf
import "types/common.proto";

message Pricing {
    types.Money price = 1;
}
```

## 5. Service Methods
Follow the `MethodNameRequest`/`MethodNameResponse` pattern.

```protobuf
service OrderService {
    rpc CreateOrder(CreateOrderRequest) returns (CreateOrderResponse);
}

message CreateOrderRequest {
    string item_id = 1;
}

message CreateOrderResponse {
    string order_id = 1;
}
```

## 6. Clean Requests (Empty)
If a method takes no arguments (or returns nothing substantial), use `google.protobuf.Empty`. Do not create `MyRequest {}`.

```protobuf
import "google/protobuf/empty.proto";

service MyService {
    rpc Ping(google.protobuf.Empty) returns (PingResponse);
}
```

## 7. Single External Argument (Direct Use)
If an RPC method accepts **exactly one** argument that is an external type (e.g. `User.Id`, `Order` from another package), use it directly instead of wrapping it in a Request message.

**Correct**:
```protobuf
rpc GetUser(types.User.Id) returns (types.User);
```

**Incorrect** (Unnecessary wrapper):
```protobuf
rpc GetUser(GetUserRequest) returns (GetUserResponse);
message GetUserRequest { types.User.Id id = 1; }
```
