# Сервис: AuthService

## 1. Описание

**`AuthService`** — это публичный клиентский сервис, который является единой точкой входа для **регистрации**, **авторизации** и **восстановления доступа**.

Ключевой особенностью этого сервиса является то, что он **инициирует** процессы, возвращая `ConfirmationResponse`. Этот ответ может содержать `Confirmation` (сессию подтверждения), которую необходимо завершить через `ConfirmationService`, или `authorization_bearer` для немедленного доступа в случаях, когда дополнительное подтверждение не требуется (например, при входе с доверенного устройства).

## 2. Описание методов (RPC)

### `rpc CheckContact(Contact) returns (CheckResponse)`
- **Назначение**: Проверить, существует ли уже пользователь с таким контактом (email).
- **Параметры `Contact`**:
  - `email`: Email для проверки.
- **Ответ `CheckResponse`**:
  - `is_busy`: `true`, если email уже используется.

### `rpc CheckDistributorUsername(CheckDistributorUsernameRequest) returns (CheckResponse)`
- **Назначение**: Проверить, занято ли указанное имя пользователя для дистрибьютора.
- **Параметры `CheckDistributorUsernameRequest`**:
  - `username`: Имя пользователя для проверки.
- **Ответ `CheckResponse`**:
  - `is_busy`: `true`, если имя пользователя уже занято.

### `rpc Authorize(Contact) returns (ConfirmationResponse)`
- **Назначение**: Начать процесс входа для **существующего** пользователя.
- **Параметры `Contact`**:
  - `email`: Email пользователя для входа.

### `rpc Register(RegisterRequest) returns (ConfirmationResponse)`
- **Назначение**: Начать процесс регистрации для **нового** пользователя.
- **Параметры `RegisterRequest`**:
  - `contact`: Контактные данные (email) для нового аккаунта.
  - `locale`: Локаль, выбранная пользователем.
  - `optional distributor_request`: Необязательные данные для одновременной регистрации аккаунта дистрибьютора.
    - `invite_code`: Код приглашения.
    - `distributor_username`: Желаемое имя пользователя для дистрибьютора.

### `rpc RecoverPassword(Contact) returns (ConfirmationResponse)`
- **Назначение**: Начать процесс восстановления доступа.
- **Параметры `Contact`**:
  - `email`: Email аккаунта, для которого нужно восстановить доступ.
