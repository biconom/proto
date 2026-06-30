# CHANGELOG: 30 Июня 2026

Сводка изменений API. Два блока:
**(1)** деталь транзакции **`SlotQuestRewardDetails`** — приз за выполнение слот-квеста
теперь несёт `slot_id` + `quest_id` в истории транзакций;
**(2)** **WinTime**: статистика по типам, фильтр транзакций по типам, группировка подряд
однотипных (новый RPC `ListTransactionGroups`).

---

## 1. 🏆 Деталь транзакции: приз за слот-квест

### `biconom/types/transaction.proto`

В `oneof details` карточки транзакции добавлен вариант для приза за выполнение
квеста слота:

```protobuf
oneof details {
    // ...
    ReinvestDiscountDetails reinvest_discount = 22;
    SlotQuestRewardDetails slot_quest_reward = 23;  // НОВОЕ
}

message SlotQuestRewardDetails {
    uint32 slot_id = 1;   // слот, выполнивший условие квеста
    uint32 quest_id = 2;  // идентификатор квеста (1..6)
}
```

> Раньше транзакция приза за слот-квест приходила в истории **без деталей**
> (видна только как изменение баланса). Теперь несёт `slot_id` + `quest_id`,
> что позволяет UI показать карточку «приз за квест №N». В одной механике
> приз может быть в нескольких валютах (USDT / WIN_COINS / USDT_GAMING) —
> каждая приходит **отдельной** проводкой с одинаковыми `slot_id`/`quest_id`.
> `slot_id` присутствует в BFF-справочнике `slots` ответа истории.

---

## 2. 🪙 WinTime: статистика по типам, фильтр и группировка

Три механизма поверх истории WinTime. Тип транзакции — стабильный дискриминант
(метаданные не влияют): `ADMIN_ADJUST`, `PASSIVE_BONUS`, `REFERRAL_BONUS`,
`SLOT_QUEST_REWARD`.

### `biconom/types/win_time.proto`

```protobuf
message WinTime {
    enum TxType {
        TX_TYPE_UNSPECIFIED = 0;
        TX_TYPE_ADMIN_ADJUST = 1;
        TX_TYPE_PASSIVE_BONUS = 2;
        TX_TYPE_REFERRAL_BONUS = 3;
        TX_TYPE_SLOT_QUEST_REWARD = 4;
    }

    // Суммарная статистика по типу за всё время (sum знаковый: + начисления, − списания).
    message TypeStat { TxType type = 1; uint64 count = 2; int64 sum = 3; }

    // Группа подряд идущих однотипных транзакций.
    message TransactionGroup {
        TxType type = 1;
        uint64 count = 2;
        int64 sum = 3;
        uint32 seq_from = 4;
        uint32 seq_to = 5;
        google.protobuf.Timestamp created_from = 6;
        google.protobuf.Timestamp created_to = 7;
        uint32 group_seq = 8; // идентификатор группы = курсор пагинации
    }

    message Balance {
        int64 amount = 1;
        uint32 seq = 2;       // последний seqno транзакции (курсор для ListTransactions)
        uint32 group_seq = 3; // НОВОЕ: последний group_seq владельца (курсор для ListTransactionGroups)
    }
}
```

> `Balance.group_seq` (поле 3) — «верхушка» материализованных групп: номер последней
> группы владельца, рядом с `seq`. Удобный стартовый курсор для `ListTransactionGroups`
> (backward). `0`, если групп ещё нет. Отдаётся в любом ответе с балансом.

### `biconom/client/win_time/win_time.proto`

**(1) Статистика** — `repeated TypeStat stats` добавлено в `BalanceResponse` (поле 6).
Отдаётся в `GetBalance` и `ListTransactions`; агрегат за всю историю, не зависит от
фильтра/пагинации. Возвращаются только типы с `count > 0`.

**(2) Фильтр по типам** — в `ListTransactionsRequest` добавлено `repeated TxType types`
(поле 3). Пусто → все типы. Несколько типов → объединение (merge по времени).

**(3) Группировка** — новый RPC:

```protobuf
rpc ListTransactionGroups(ListTransactionGroupsRequest) returns (TransactionGroupsResponse);

message ListTransactionGroupsRequest {
    optional uint32 cursor = 1;            // group_seq последней группы (exclusive)
    optional biconom.types.Sort sort = 2;  // direction + limit ГРУПП (≤ 1000)
}

message TransactionGroupsResponse {
    biconom.types.WinTime.Balance balance = 1;
    repeated biconom.types.WinTime.TransactionGroup groups = 2;
    optional uint32 next_cursor = 3;       // group_seq следующей страницы
    repeated biconom.types.WinTime.TypeStat stats = 4;
}
```

> Группы **материализованы** в БД (CF `wt_groups`): строятся при записи каждой транзакции —
> если тип совпал с последней группой дистрибьютора, она дополняется (count/sum/границы),
> иначе открывается новая. API читает их как готовый индекс, **без агрегации на лету**.
> Курсор и `limit` — по **группам** (`group_seq`), не по транзакциям. Фильтра по типам нет:
> группы идут по всей истории подряд (фильтр ломал бы «подряд»-семантику).
> Существующая история бэкфиллится один раз пост-миграцией.

---

## 💻 Как использовать на Frontend

### Карточка приза за слот-квест в истории транзакций

```javascript
const history = await TransactionServiceClient.History({ /* cursor, sort */ });
for (const group of history.items) {
    for (const entry of group.entries) {
        if (entry.details?.slotQuestReward) {
            const { slotId, questId } = entry.details.slotQuestReward;
            // slotId есть в history.slots (BFF-справочник)
            renderQuestReward({
                slotId,
                questId,
                amount: entry.amount,        // сумма приза (в валюте проводки)
                currencyId: entry.currencyId,
            });
        }
    }
}
```

> ⚠️ Один выполненный квест может породить **несколько** проводок (по одной на
> каждую валюту приза). Группируйте по `(slotId, questId)`, если хотите показать
> «приз за квест» единой карточкой с перечнем валют.

### WinTime: статистика, фильтр, группы

```javascript
// Статистика по типам (в любом ответе баланса/списка).
const bal = await WinTimeServiceClient.GetBalance({});
for (const s of bal.stats) {
    renderTypeStat(s.type, s.count, s.sum); // sum знаковый
}

// Фильтр: только реферальные + квестовые транзакции.
const filtered = await WinTimeServiceClient.ListTransactions({
    sort: { direction: 'BACKWARD', limit: 1000 },
    types: ['TX_TYPE_REFERRAL_BONUS', 'TX_TYPE_SLOT_QUEST_REWARD'],
});

// Группы подряд однотипных — материализованы в БД, пагинация по группам (group_seq).
// balance.groupSeq — «верхушка» групп, удобно как стартовый курсор (backward).
const grouped = await WinTimeServiceClient.ListTransactionGroups({
    sort: { direction: 'BACKWARD', limit: 1000 }, // limit = число ГРУПП
});
for (const g of grouped.groups) {
    renderGroup(g.groupSeq, g.type, g.count, g.sum, g.seqFrom, g.seqTo); // готовые агрегаты + id группы
}
if (grouped.nextCursor != null) loadNextPage(grouped.nextCursor); // курсор = group_seq
```

---

## 📋 Сводка изменений

| Что изменилось | Где |
|---|---|
| Деталь транзакции `SlotQuestRewardDetails` (slot_id + quest_id) | `transaction` (`slot_quest_reward = 23`) |
| `TxType` enum + `TypeStat` + `TransactionGroup` | `types/win_time.proto` |
| Статистика по типам `repeated TypeStat stats` | `BalanceResponse` (field 6) |
| Фильтр по типам `repeated TxType types` | `ListTransactionsRequest` (field 3) |
| Новый RPC `ListTransactionGroups` (+ Request/Response) | `client/win_time/win_time.proto` |
| `TransactionGroup.group_seq` — id группы / курсор пагинации | `types/win_time.proto` (field 8) |
| `Balance.group_seq` — номер последней группы владельца | `types/win_time.proto` (field 3) |
