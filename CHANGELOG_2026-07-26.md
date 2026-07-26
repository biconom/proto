# CHANGELOG: 26 Июля 2026

Сводка изменений API после релиза от 25 июля. Один блок:
**(1)** **custom_flags** — пользовательская битовая маска флагов интерфейса
(два новых RPC в `UserService` + новое поле в `AccountService.GetCurrent`).

---

## 1. 🚩 custom_flags — пользовательская битовая маска флагов

Заказчику нужно хранить на пользователе флаги состояния фронтенда: «интро
пройдено», «обучение завершено» и т.д. Механика аналогична
`UserService.GetPermissions`, но маска **пользовательская**: семантику битов
определяет исключительно фронтенд, бэкенд хранит значение непрозрачно и не
интерпретирует. Тип — `uint32` (32 флага).

### `biconom/client/user/user.proto`

Два новых метода в `UserService`:

```protobuf
service UserService {
    // Получает пользовательскую битовую маску флагов (custom_flags) текущего пользователя.
    rpc GetCustomFlags(google.protobuf.Empty) returns (GetCustomFlagsResponse);

    // Атомарно обновляет пользовательскую битовую маску флагов:
    // new_mask = (old_mask | set_bit_mask) & ~clear_bit_mask.
    rpc UpdateCustomFlags(UpdateCustomFlagsRequest) returns (GetCustomFlagsResponse);
}

message GetCustomFlagsResponse {
    uint32 custom_flags_bit_mask = 1;
}

message UpdateCustomFlagsRequest {
    uint32 set_bit_mask = 1;   // биты, которые нужно установить (OR)
    uint32 clear_bit_mask = 2; // биты, которые нужно снять (AND NOT, применяется после set)
}
```

Запись сделана атомарной (set/clear вместо полной перезаписи), чтобы две
вкладки/устройства не затирали изменения друг друга. Оба метода требуют
авторизацию по сессии; пользователь определяется из токена.

### `biconom/client/account/account.proto`

Маска возвращается и в сводке текущего аккаунта — фронтенду не нужен
отдельный запрос при загрузке приложения:

```protobuf
message UserProfile {
    // ...существующие поля 1-7...
    uint32 custom_flags_bit_mask = 8; // ← НОВОЕ: см. UserService.GetCustomFlags
}
```

### Серверная сторона (core v1.9.90)

- Новое поле `custom_flags: u32` на записи `user`; начальное значение для
  всех существующих пользователей — `0`.
- Pre-init миграция `v01_09_90` перезаписывает таблицу `user` в новом формате
  (postcard позиционен) — выполняется автоматически при первом старте v1.9.90.
- Версия proto: **0.3.5**.
