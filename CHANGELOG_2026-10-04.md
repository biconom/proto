# CHANGELOG: 4 Октября 2026 (proto `v0.3.28` → `v0.3.29`)

Стейкинг: **экстра-прибыль** — акция, по которой депозит, созданный в заданном диапазоне
дат, на каждой выплате цикла получает дополнительный процент от тела. Пара к ядру
`1.10.32`. Пока в системе нет ни одной акции, всё работает как раньше.

> ✅ **Не breaking.** Только новые сообщения, новые поля в конец существующих сообщений,
> новый `oneof`-вариант и новые RPC. Ничего не удалено и не перенумеровано. Старые
> клиенты новые поля игнорируют.

## Что нужно сделать фронту — коротко

| Действие | Где подробно |
|---|---|
| Перегенерировать клиент из `.proto` | — |
| Диалог нового депозита: строка «Extra profit X %» из `State.extra_promo.rate` | §2 |
| Карточка депозита: строка из `Deposit.extra_rate`, начислено — `Deposit.total_extra_paid` | §2 |
| Журнал: обработать событие `ExtraIncomePaid` (`oneof` 22) | §3 |
| История кошелька: карточка `staking_extra_income` | §4 |
| Сводка: «заработано» = `total_income_paid` + `total_extra_paid` (два отдельных поля) | §2 |
| Админка: экран акций (5 новых RPC) | §5 |

Правило, которое надо донести до экрана: экстра начисляется на **каждой** выплате цикла
депозита, **созданного** в диапазоне действующей акции (смотрится дата создания депозита,
не дата выплаты); сумма — `тело × rate`, **отдельной проводкой** в той же группе, что и
доход цикла. При включённом реинвесте прибыли доход и экстра уходят в тело вместе. Акцию
можно завести за прошлый диапазон — она затронет только **следующие** выплаты, доплаты за
прошедшие циклы нет. Новая версия акции меняет будущие выплаты, но не прошлые.

## 1. Новая модель `Staking.ExtraPromo`

### `biconom/types/staking.proto`

```protobuf
message ExtraPromo {
    message Id   { oneof identifier { uint32 id = 1; } }
    message List { repeated ExtraPromo items = 1; }

    enum StatusBit {
        STATUS_BIT_UNSPECIFIED = 0;
        STATUS_BIT_ARCHIVED = 1;     // заменена новой версией
        STATUS_BIT_DEACTIVATED = 2;  // выключена админом
    }

    uint32 id = 1;                                    // идентификатор ВЕРСИИ
    google.protobuf.Timestamp deposit_created_at_ge = 2;  // включается
    google.protobuf.Timestamp deposit_created_at_lt = 3;  // не включается
    string rate = 4;                                  // коэффициент: 5% -> "0.05"
    uint32 status = 5;                                // битовая маска, 0 = действующая
    uint32 prev_id = 6;                               // цепочка действующих
    uint32 next_id = 7;
    uint32 version_prev_id = 8;                       // цепочка версий
    uint32 version_next_id = 9;
    uint32 created_by_user_id = 10;                   // только админский API, клиенту 0
    google.protobuf.Timestamp created_at = 11;
    google.protobuf.Timestamp updated_at = 12;
}
```

Запись неизменяема: любая правка — новая версия, прежняя получает `ARCHIVED`. Подробно —
[`biconom/types/staking.md`, раздел 4.5](biconom/types/staking.md).

## 2. Новые поля в существующих сообщениях

### `biconom/types/staking.proto`

```protobuf
message Deposit {
    // ...поля 1–24 без изменений...
    string extra_rate = 25;        // ставка ближайшей выплаты, "0.00" = акции нет
    uint32 extra_promo_id = 26;    // версия акции, 0 = нет
    string total_extra_paid = 27;  // начислено экстры за всё время
}

message DepositsSummary {
    // ...поля 1–11 без изменений...
    string total_extra_paid = 12;
}

message State {
    // ...поля 1–9 без изменений...
    optional ExtraPromo extra_promo = 10;  // акция, действующая прямо сейчас
}

message Summary {
    // ...поля 1–12 без изменений...
    string total_extra_paid = 13;  // для админа
}
```

| Поле | Как показывать |
|---|---|
| `State.extra_promo.rate` | Строка «Extra profit X %» в диалоге **нового** депозита. Поля нет — строку не показывать |
| `Deposit.extra_rate` | Строка у **открытого** депозита. `"0.00"` — акции нет либо депозит созрел или закрыт: скрыть |
| `Deposit.body × Deposit.extra_rate` | Ожидаемая экстра ближайшей выплаты. Считает **клиент**: прогнозных полей сервер не отдаёт |
| `Deposit.total_extra_paid` | Сколько экстры уже начислено по депозиту |

- `Deposit.total_income_paid` экстру **не** включает: «всего заработано» — сумма двух полей.
  То же в сводке: `DepositsSummary.total_income_paid` — только доход по ставке тира,
  `DepositsSummary.total_extra_paid` — экстра.
- `DepositsSummary.total_reinvested` теперь включает экстру, ушедшую в тело, поэтому может
  быть больше `total_income_paid`. Сравнивать с суммой `total_income_paid + total_extra_paid`.
- `Deposit.extra_rate` не зафиксирована в депозите: новая версия акции меняет её для
  следующей выплаты. Выплаченное не пересчитывается.

## 3. События журнала

### `biconom/types/staking.proto`

```protobuf
message Event {
    message ExtraIncomePaid {
        uint32 payout_seq = 1;
        string amount = 2;
        string rate = 3;            // применённая ставка, "0.05"
        string body_at_payout = 4;  // база: тело на момент выплаты
        uint32 extra_promo_id = 5;
    }
    // oneof data: ...без изменений...
    //     ExtraIncomePaid extra_income_paid = 22;
}
```

**Изменился смысл** (не тип и не номер) `Event.IncomeReinvested.amount`: теперь это сумма,
ушедшая в тело, — доход цикла **вместе** с экстрой (`IncomePaid.amount` +
`ExtraIncomePaid.amount`). Без акции значение прежнее.

## 4. Карточки транзакций

### `biconom/types/transaction.proto`

```protobuf
// oneof details: ...без изменений...
//     StakingExtraIncomeDetails staking_extra_income = 33;

message StakingExtraIncomeDetails {
    uint32 deposit_id = 1;
    uint32 payout_seq = 2;
    string rate = 3;            // коэффициент ("0.05")
    uint32 extra_promo_id = 4;
}
```

Приходит в **одной группе** с `staking_income` того же цикла, отдельной проводкой.
Подробно — [`biconom/types/transaction.md`](biconom/types/transaction.md), раздел 22.

**Изменился смысл** `StakingProfitReinvestDetails`: при экстре в тело уходит сумма дохода
и экстры **одной** проводкой. Группа выплаты с реинвестом и акцией — три строки:
`staking_income`, `staking_extra_income`, `staking_profit_reinvest`.

## 5. Админка: `StakingAdminService`

### `biconom/admin/staking/staking.proto`

```protobuf
rpc CreateExtraPromo(CreateExtraPromoRequest) returns (Staking.ExtraPromo);
rpc UpdateExtraPromo(UpdateExtraPromoRequest) returns (Staking.ExtraPromo);
rpc GetExtraPromo(Staking.ExtraPromo.Id) returns (Staking.ExtraPromo);
rpc ListExtraPromos(ListExtraPromosRequest) returns (Staking.ExtraPromo.List);
rpc ListExtraPromoVersions(Staking.ExtraPromo.Id) returns (Staking.ExtraPromo.List);

message CreateExtraPromoRequest {
    google.protobuf.Timestamp deposit_created_at_ge = 1;
    google.protobuf.Timestamp deposit_created_at_lt = 2;
    string rate = 3;
}

message UpdateExtraPromoRequest {
    uint32 id = 1;  // верхняя (не архивная) версия
    optional google.protobuf.Timestamp deposit_created_at_ge = 2;
    optional google.protobuf.Timestamp deposit_created_at_lt = 3;
    optional string rate = 4;
    optional bool deactivated = 5;
}

message ListExtraPromosRequest {
    bool include_deactivated = 1;
}
```

Все пять требуют `ADMIN_STAKING`. `UpdateExtraPromo` **не правит запись**, а выпускает
новую версию. Ошибки:

| Ситуация | Код |
|---|---|
| Диапазон пересекается с действующей акцией (создание, обновление) | `FailedPrecondition`, `STAKING_EXTRA_PROMO_OVERLAP` |
| Граница не задана, с ненулевым `nanos`, вне 0…4294967295 секунд либо `lt` не больше `ge` | `InvalidArgument`, `STAKING_EXTRA_PROMO_RANGE_INVALID` |
| Ставка не число, не больше нуля, больше `"1"` либо с более чем 6 знаками после точки | `InvalidArgument`, `STAKING_EXTRA_PROMO_RATE_INVALID` |
| `UpdateExtraPromo` ничего не меняет | `InvalidArgument`, `STAKING_EXTRA_PROMO_NO_CHANGE` |
| `UpdateExtraPromo` по архивной версии | `FailedPrecondition`, `STAKING_EXTRA_PROMO_ARCHIVED` |
| Версии с таким `id` нет (`UpdateExtraPromo`, `GetExtraPromo`, `ListExtraPromoVersions`) | `NotFound`, `STAKING_EXTRA_PROMO_NOT_FOUND` |

Правила ввода для формы акции:

- **Даты** — `google.protobuf.Timestamp` с точностью до секунды: `nanos` обязан быть `0`
  (из JS: `seconds = Math.floor(date.getTime() / 1000)`, `nanos = 0`). Нижняя граница
  включается, верхняя — нет: акция «весь октябрь» — это `ge = 1 октября 00:00:00`,
  `lt = 1 ноября 00:00:00`.
- **Ставка** — коэффициент строкой, а не процент: 5% → `"0.05"`. От `"0.000001"` до `"1"`,
  не более 6 знаков после точки — седьмой знак не обрезается, а даёт ошибку.
- **Список** `ListExtraPromos` отдаёт только верхние версии, от поздних диапазонов к ранним;
  действующая — `status == 0`, деактивированная — `status == 2` (приходит только с
  `include_deactivated = true`). История правок одной акции — `ListExtraPromoVersions`.

Подробно — [`biconom/admin/staking/staking.md`](biconom/admin/staking/staking.md), раздел 3.5.

## Миграция фронта

1. Перегенерировать клиент из `.proto`.
2. Клиент: ничего ломающего. Для показа акции достаточно `State.extra_promo` и
   `Deposit.extra_rate`.
3. Журнал: если в `switch` по `Event.data` есть ветка по умолчанию с ошибкой —
   добавить `extra_income_paid` (22).
4. История кошелька: если есть ветка по умолчанию для неизвестной карточки — добавить
   `staking_extra_income` (33).
5. Если где-то `IncomeReinvested.amount` сверялся с `IncomePaid.amount` — учесть экстру.
6. Админка: экран акций; `UpdateExtraPromo` вызывать с `id` **верхней** версии.

## Что не менялось

Ничего из существующего не удалено и не перенумеровано: поля `Deposit` 1–24,
`DepositsSummary` 1–11, `State` 1–9, `Summary` 1–12, варианты `Event.data` и
`HistoryEntry.details` до 32 — на прежних номерах.
