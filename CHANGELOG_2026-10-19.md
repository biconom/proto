# CHANGELOG: 19 Октября 2026 (proto `v0.3.30` → `v0.3.31`)

Одно изменение дивидендного пула: админский RPC **массового включения авто-реинвеста**.
Пара к ядру `1.10.36`.

> ✅ **Не breaking.** Новый RPC и два новых сообщения. Ничего не удалено и не
> перенумеровано.

> ℹ️ **Клиентскому приложению делать ничего не нужно.** Включённые партнёры видят
> авто-реинвест как обычно — в `GetAutoReinvest` / `GetDividendPool` / профиле.

## Что нужно сделать админке — коротко

| Действие | Где подробно |
|---|---|
| Перегенерировать клиент из `.proto` | — |
| Кнопка «Включить авто-реинвест всем» с обязательным предпросмотром (`dry_run`) | §1 |

## 1. `DividendPoolAdminService.EnrollAutoReinvestAll`

### `biconom/admin/dividend_pool/dividend_pool.proto`

```protobuf
rpc EnrollAutoReinvestAll(EnrollAutoReinvestAllRequest) returns (EnrollAutoReinvestAllResponse);

message EnrollAutoReinvestAllRequest {
    uint32 cycles = 1;                 // срок в циклах (на проде цикл — сутки), из тарифов конфига
    bool include_admin_disabled = 2;   // включить и выключенных администратором
    bool dry_run = 3;                  // только посчитать, без изменений
}

message EnrollAutoReinvestAllResponse {
    string discount_percent = 1;       // процент, с которым включены ("40.0000")
    uint32 enrolled = 2;               // включено (не было или срок закончился)
    uint32 already_active = 3;         // пропущено: уже активен
    uint32 admin_disabled_skipped = 4; // пропущено: выключен администратором
    uint32 admin_disabled_enrolled = 5;// включено поверх выключения (include_admin_disabled)
    bool dry_run = 6;                  // эхо: true — это прогноз
}
```

**Права:** `ROOT` (остальные методы сервиса — `ADMIN_FINANCE`).

**Кого включает.** Всех, у кого есть вложение в пул и нет активного авто-реинвеста:
кто никогда не выбирал и у кого срок закончился. Процент — текущий тариф конфига на
`cycles` циклов, как если бы партнёр выбрал его сам.

**Кого не трогает.** Активных — ни процент, ни счётчик циклов не меняются. Выключенных
администратором — пока не передан `include_admin_disabled = true`.

**Повтор безопасен.** Второй вызов никого заново не включает: все уже в `already_active`.

**Отката нет.** Выключить обратно — только поштучно `SetDistributorAutoReinvestStatus`.
Поэтому в админке: сначала вызов с `dry_run = true`, показать цифры, затем подтверждение
и боевой вызов с теми же параметрами.

**Ошибки** (ничего не изменено):

| Код | Когда |
|---|---|
| `DIVIDEND_POOL_SERVICE_NOT_ACTIVE` | пул на паузе или остановлен |
| `DIVIDEND_POOL_NO_DAILY_RATE` | ещё не сгенерирован ни один дневной процент |
| `DIVIDEND_POOL_AUTO_REINVEST_INVALID_CYCLES` | такого срока нет среди тарифов |
| `DIVIDEND_POOL_AUTO_REINVEST_NOT_CONFIGURED` | не настроена пара WIN/USDT |
