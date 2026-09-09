# CHANGELOG: 9 Сентября 2026 (proto `v0.3.24` → `v0.3.25`)

Одно аддитивное изменение: `SystemControlService.UserFindByEmail` теперь
отдаёт и список дистрибьюторов найденного пользователя.

> ✅ **Не breaking.** Добавлено поле №5 в существующий ответ. Старые клиенты
> его игнорируют.

## 1. 🔎 `UserFindByEmailResponse.distributor_ids`

### `biconom/admin/system/system.proto`

```protobuf
message UserFindByEmailResponse {
    bool   found   = 1;
    uint32 user_id = 2;
    string email   = 3;
    bool   banned  = 4;
    repeated uint32 distributor_ids = 5;   // ← новое, по возрастанию id
}
```

Право, резолв почты и остальные поля без изменений (`SYSTEM_CONTROL`,
канонический резолв как при авторизации). `distributor_ids` пуст, если
пользователь не найден (`found = false`) или дистрибьютора ещё не создал.
Карточки дистрибьюторов при необходимости запрашиваются отдельно по id.

## Совместимость

| | |
|---|---|
| Номера и типы существующих полей | не изменены |
| Новые поля | `UserFindByEmailResponse.distributor_ids` (№5) |
| Старый клиент против нового бэкенда | работает, поле игнорируется |
| Новый клиент против старого бэкенда | работает, поле всегда пустое |
