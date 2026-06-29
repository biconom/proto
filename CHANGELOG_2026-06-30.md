# CHANGELOG: 30 Июня 2026

Сводка изменений API. Один блок:
**(1)** деталь транзакции **`SlotQuestRewardDetails`** — приз за выполнение слот-квеста
теперь несёт `slot_id` + `quest_id` в истории транзакций.

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

---

## 📋 Сводка изменений

| Что изменилось | Где |
|---|---|
| Деталь транзакции `SlotQuestRewardDetails` (slot_id + quest_id) | `transaction` (`slot_quest_reward = 23`) |
