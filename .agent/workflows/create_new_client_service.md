---
description: Create a new Client API Service (RPC + MD)
---

# Create New Client Service

This workflow standardizes the creation of new Client API Services, ensuring they follow the project's architecture managed by the `grpc-designer` skill.

## Steps

1. **Input Collection**:
   - Ask for the **Domain/Service Name** (e.g., "Wallet", "Trade").
   - (Optional) ask for specific methods to include initially.

2. **Directory Setup**:
   - Create a new subdirectory: `biconom/client/[snake_case_name]`.

3. **File Generation**:
   - Create `biconom/client/[snake_case_name]/[snake_case_name].proto`
   - Create `biconom/client/[snake_case_name]/[snake_case_name].md`

4. **Proto Template Application**:
   - Create the file with the following standard structure:
     ```protobuf
     syntax = "proto3";

     package biconom.client.[snake_case_name];

     // Import common types (adjust as needed)
     import "biconom/types/common.proto"; 

     // [CamelCaseName]Service provides ...
     service [CamelCaseName]Service {
       // Example method:
       // rpc Get(GetRequest) returns (GetResponse);
     }

     // Define Request/Response messages below
     message GetRequest {
       // ...
     }

     message GetResponse {
       // ...
     }
     ```

5. **Markdown Template Application**:
   - Create the documentation with the following sections:
     ```markdown
     # Service: [CamelCaseName]Service

     ## 1. Описание
     [Description of what this service provides to the client/frontend.]

     ## 2. Методы API
     
     ### `MethodName`
     - **Назначение**: ...
     - **Входные параметры**: ...
     - **Возвращаемое значение**: ...

     ## 3. Права доступа и безопасность
     [Does it require a Bearer token? specific scopes?]

     ## 4. Сценарии использования
     - **[UseCase 1]**: ...
     ```

6. **Verification**:
   - Run `view_file` to confirm the package name matches the directory structure.
