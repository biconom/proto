# CHANGELOG: 28 Октября 2026 (proto `v0.3.32` → `v0.3.33`)

Одно изменение дивидендного пула: админский RPC **продления авто-реинвеста до даты**.
Пара к ядру `1.10.45`.

> ✅ **Не breaking.** Новый RPC и два новых сообщения. Ничего не удалено и не
> перенумеровано.

> ℹ️ **Клиентскому приложению делать ничего не нужно.** Продлённые партнёры видят
> больший `cycles` / `cycles_remaining` в `AutoReinvestState` как обычно. Число
> `cycles` может не совпадать ни с одним тарифом (например 33) — это продление,
> а не выбор тарифа: показывать как есть, не искать по нему тариф.

## Что нужно сделать админке — коротко

| Действие | Где подробно |
|---|---|
| Перегенерировать клиент из `.proto` | — |
| Форма «Продлить авто-реинвест до даты» с обязательным предпросмотром (`dry_run`) | §1 |

## 1. `DividendPoolAdminService.ExtendAutoReinvestUntil`

### `biconom/admin/dividend_pool/dividend_pool.proto`

```protobuf
rpc ExtendAutoReinvestUntil(ExtendAutoReinvestUntilRequest) returns (ExtendAutoReinvestUntilResponse);

message ExtendAutoReinvestUntilRequest {
    string until = 1;                  // день последнего цикла, UTC, строго DD-MM-YYYY
    bool include_admin_disabled = 2;   // продлить и выключенных администратором (снимает выключение)
    bool dry_run = 3;                  // только посчитать, без изменений
}

message ExtendAutoReinvestUntilResponse {
    string until = 1;                  // эхо даты
    uint32 extended = 2;               // продлено: активен, кончался раньше даты
    uint32 reactivated = 3;            // включено снова: циклы были исчерпаны
    uint32 already_until = 4;          // не тронуто: и так до этого дня или дольше
    uint32 admin_disabled_skipped = 5; // пропущено: выключен администратором
    uint32 admin_disabled_extended = 6;// продлено поверх выключения (include_admin_disabled)
    uint32 without_auto_reinvest = 7;  // пропущено: нет состояния авто-реинвеста
    uint64 cycles_added = 8;           // добавлено циклов всем вместе
    bool dry_run = 9;                  // эхо: true — это прогноз
}
```

**Права:** `ROOT`.

**Кого продлевает.** Всех, у кого последний цикл наступает раньше дня `until` (UTC):
добавляется ровно столько циклов, чтобы последний пришёлся на этот день. Счёт — по
календарю от ближайшего цикла партнёра; прошлые выплаты на расчёт не влияют.
Закончившиеся включаются снова со своим прежним процентом.

**Кого не трогает.** У кого последний цикл и так в этот день или позже. Без состояния
авто-реинвеста — сначала `EnrollAutoReinvestAll`. Выключенных администратором — пока не
передан `include_admin_disabled = true`.

**Повтор безопасен.** Вызов с той же датой — no-op; продлить ещё — более поздняя дата.

**Ошибки** (ничего не изменено):

| Код | Когда |
|---|---|
| `DIVIDEND_POOL_AUTO_REINVEST_INVALID_DATE` | дата не `DD-MM-YYYY` или такого дня нет |
| `DIVIDEND_POOL_AUTO_REINVEST_DATE_IN_PAST` | день уже прошёл по UTC |
| `DIVIDEND_POOL_SERVICE_NOT_ACTIVE` | пул на паузе или остановлен |
| `DIVIDEND_POOL_AUTO_REINVEST_NOT_CONFIGURED` | не настроена пара WIN/USDT |
