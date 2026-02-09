---
description: Create a new Data Model Entity (Proto + MD)
---

# Create New Data Model Entity

This workflow streamlines the creation of new Mentor data models, ensuring they adhere to the strict project standards defined in the `grpc-designer` skill.

## Steps

1. **Input Collection**:
   - Ask the user for the **Name** of the new entity (e.g., "Invoice", "Ticket").
   - (Optional) Ask for a brief description to pre-fill the documentation.

2. **File Generation**:
   - Create `mentor/types/[snake_case_name].proto`
   - Create `mentor/types/[snake_case_name].md`

3. **Proto Template Application**:
   - Apply the standard template with:
     - `message [Name]`
     - `message Id` (nested)
     - `message List` (nested)
     - `UNSPECIFIED` enum zero value (nested in a wrapper like `Status` if applicable, or general structure).
     - Standard `created_at` / `updated_at`.

4. **Markdown Template Application**:
   - Apply the standard 5-section structure:
     - 1. Описание
     - 2. Назначение и решаемые задачи
     - 3. Ключевые поля и концепции
     - 4. Сценарии использования (кейсы)
     - 5. Связи с другими моделями

5. **Verification**:
   - Run `cat` or `view_file` on created files to confirm content.
