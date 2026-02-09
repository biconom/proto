---
description: Create a new Admin API Service (RPC + MD)
---

# Create New Admin Service

This workflow standardizes the creation of new Admin API Services, ensuring they follow the project's architecture managed by the `grpc-designer` skill. These services are located in `biconom/admin` and are intended for internal administration, back-office dashboards, and system management.

## Steps

1. **Input Collection**:
   - Ask for the **Domain/Service Name** (e.g., "Ledger", "SystemSettings").
   - (Optional) ask for specific methods to include initially.

2. **Directory Setup**:
   - Create a new subdirectory: `biconom/admin/[snake_case_name]`.

3. **File Generation**:
   - Create `biconom/admin/[snake_case_name]/[snake_case_name].proto`
   - Create `biconom/admin/[snake_case_name]/[snake_case_name].md`

4. **Proto Template Application**:
   - Create the file with the following standard structure:
     ```protobuf
     syntax = "proto3";

     package biconom.admin.[snake_case_name];

     // Импорт общих типов (скорректируйте при необходимости)
     import "biconom/types/common.proto"; 

     // [CamelCaseName]Service предоставляет административные возможности для ...
     service [CamelCaseName]Service {
       // Пример метода:
       // rpc Get(GetRequest) returns (GetResponse);
     }

     // Определите сообщения Request/Response ниже
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
     [Описание возможностей сервиса для административной панели.]

     ## 2. Методы API
     
     ### `MethodName`
     - **Назначение**: ...
     - **Входные параметры**: ...
     - **Возвращаемое значение**: ...
     - **Побочные эффекты**: ... (Важно для админских действий)

     ## 3. Права доступа и безопасность
     - **Требуемые права**: [Укажите необходимые роли/права]
     
     ## 4. Сценарии использования
     - **[UseCase 1]**: ...
     ```

6. **Verification**:
   - Run `view_file` to confirm the package name matches the directory structure `biconom/admin/...`.
