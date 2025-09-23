# Сервис: PaymentDestinationService

## 1. Описание

**`PaymentDestinationService`** — это клиентский сервис для управления "адресной книгой" пользователя (списком назначений платежей). Он предоставляет полный набор методов для создания, просмотра, изменения и архивации записей (`PaymentDestination`).

## 2. Описание методов (RPC)

### `rpc List(ListRequest) returns (ListResponse)`
- **Назначение**: Получить список сохраненных назначений платежей с возможностью фильтрации.
- **Параметры `ListRequest`**:
    - `optional biconom.types.PaymentDestination.Status.Id status`: Позволяет отфильтровать записи по их статусу (`ACTIVE` или `ARCHIVED`).
- **Логика фильтрации**:
    - Если **поле `status` пусто**, сервис вернет все записи со статусом `ACTIVE` (поведение по умолчанию).
    - Если **указан `status`**, будут возвращены записи только с этим статусом.
- **Ответ `ListResponse`**: Содержит отфильтрованный список `biconom.types.PaymentDestination`.

### `rpc Create(CreateRequest) returns (biconom.types.PaymentDestination)`
- **Назначение**: Создать новое назначение платежа в адресной книге пользователя. Новой записи автоматически присваивается статус `ACTIVE`.
- **Параметры `CreateRequest`**:
    - `name`: Пользовательское имя для записи (например, "Мой кошелек для BNB").
    - `oneof instrument_details`: Конкретные данные реквизитов (например, `blockchain`).

### `rpc Update(UpdateRequest) returns (biconom.types.PaymentDestination)`
- **Назначение**: Изменить пользовательское имя (`name`) существующей записи.
- **Параметры `UpdateRequest`**:
    - `destination_id`: ID назначения, которое нужно обновить.
    - `name`: Новое имя.

### `rpc Archive(ArchiveRequest) returns (biconom.types.PaymentDestination)`
- **Назначение**: Архивировать существующее назначение. Этот метод выполняет "мягкое удаление", меняя статус записи на `ARCHIVED`. После этого она перестанет отображаться в списке по умолчанию.
- **Параметры `ArchiveRequest`**:
    - `destination_id`: ID назначения, которое нужно заархивировать.
