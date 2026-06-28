# CHANGELOG: 27 Июня 2026

Сводка изменений API за последние 3 дня (24–27 июня). Пять крупных блоков:
**(1)** система блокировочных флагов дистрибьютора + новые маркетинговые флаги;
**(2)** подсистема игрового токена **WinTime** (клиентский сервис баланса и истории);
**(3)** облегчённый просмотр слота **`Marketing.GetSlotV2`** с поуровневыми агрегатами и доходом;
**(4)** модель **квестов слота** в `GetSlotV2` (квест №1: заполнение первой и второй линии);
**(5)** **дивидендный пул**: WIN от дисконт-бонуса авто-реинвеста (`earned_discount_win`).

---

## 1. 🚫 Блокировочные флаги дистрибьютора + маркетинговые флаги

### `biconom/admin/marketing/marketing.proto`

Добавлены RPC управления флагами (возвращают новое состояние):

```protobuf
rpc SetMarketingFlags(SetMarketingFlagsRequest) returns (GetMarketingFlagsResponse);
rpc SetDistributorFlags(SetDistributorFlagsRequest) returns (GetDistributorFlagsResponse);
```

**Новые маркетинговые флаги** (глобальные, на всех пользователей):

```protobuf
message GetMarketingFlagsResponse {
    bool v2 = 1;
    bool allow_capacity_upgrade = 2;          // покупка расширения ширины слота
    // ...
    bool allow_buy_voucher_for_others = 4;    // покупка ваучера в подарок
    bool allow_slot_placement = 5;            // досрочное ручное расставление слотов
}
```

**Блокировочные флаги конкретного дистрибьютора** (`distributor_id` обязателен):

```protobuf
message GetDistributorFlagsResponse {
    // ...
    bool block_transfer = 2;               // запрет перевода денег
    bool block_withdrawal = 3;             // запрет выплат
    bool block_slot_placement = 4;         // запрет расстановки слотов
    bool block_token_purchase = 5;         // запрет покупки токена (USDT → WINCOINS)
    bool block_buy_voucher_for_self = 6;   // запрет покупки ваучеров себе
    bool block_buy_voucher_for_others = 7; // запрет покупки ваучеров другим
}

message SetDistributorFlagsRequest {
    uint32 distributor_id = 1;
    optional bool block_transfer = 3;
    optional bool block_withdrawal = 4;
    optional bool block_slot_placement = 5;
    optional bool block_token_purchase = 6;
    optional bool block_buy_voucher_for_self = 7;
    optional bool block_buy_voucher_for_others = 8;
}
```

> ⚠️ Каждое поле в `Set*FlagsRequest` — **независимый бит**. `null`/`undefined` → бит не меняется; `true` → устанавливается; `false` → сбрасывается. `Set*` возвращает полное новое состояние флагов.

---

## 2. 🪙 WinTime — клиентский сервис игрового токена

Новая подсистема: игровой токен **WinTime** (пассивное поминутное начисление + реферальный бонус спонсору при регистрации личника). Баланс хранится отдельно от валют кошелька.

### `biconom/client/win_time/win_time.proto`

```protobuf
service WinTimeService {
    // Баланс + последние 10 транзакций (от новых к старым).
    rpc GetBalance(google.protobuf.Empty) returns (BalanceResponse);
    // История транзакций (cursor-based, до 1000 за запрос).
    rpc ListTransactions(ListTransactionsRequest) returns (BalanceResponse);
}
```

`BalanceResponse` несёт баланс + список транзакций + BFF-справочники (`slots` / `distributors` / `accounts`) для сущностей, упомянутых в деталях транзакций.

### `biconom/types/win_time.proto`

```protobuf
message WinTime {
    message Balance {
        int64 amount = 1;  // целые токены (знаковый)
        uint32 seq = 2;    // порядковый номер последней транзакции
    }
    message Transaction {
        uint32 seq = 1;
        int64 amount = 2;
        google.protobuf.Timestamp created_at = 3;
        oneof details {
            AdminAdjustDetails admin_adjust = 4;
            PassiveBonusDetails passive_bonus = 5;
            ReferralBonusDetails referral_bonus = 6;
            QuestRewardDetails quest_reward = 8;   // приз квеста слота: { slot_id, quest_id }
        }
    }
}
```

### Баланс WinTime в `WalletCurrency.List`

В ответ `WalletCurrencyService/List` добавлен баланс WinTime рядом с `items`:

```protobuf
message WalletCurrency.List {
    repeated WalletCurrency items = 1;
    optional WinTime.Balance win_time = 2;  // баланс WinTime владельца
}
```

---

## 3. 🔎 `Marketing.GetSlotV2` — облегчённый просмотр слота

### `biconom/client/marketing/marketing.proto`

```protobuf
rpc GetSlotV2(biconom.types.Slot.Id) returns (biconom.types.MarketingSlotV2);
```

В отличие от `GetSlot`: без `breadcrumbs` и `view_chains`; в `slots`/`distributors`/`accounts`/`states` — только авторизованный (executor) и просматриваемый (view) слоты. Агрегаты структуры отдаются через `levels_state`.

### `biconom/types/marketing_slot.proto` → `MarketingSlotV2.LevelState`

```protobuf
message LevelState {
    uint32 level = 1;                       // уровень глубины (1..MARKETING_LEVEL_LIMIT)
    uint32 structure_quantity = 2;          // слотов на уровне
    uint64 structure_capacity_default = 3;  // эталонная ёмкость D^level
    uint64 structure_capacity_potential = 4;// потенциальная ёмкость
    uint64 structure_capacity_reserved = 5; // зарезервированная (фактическая) ёмкость
    biconom.types.Price structure_income = 6;// доход (USDT) с этого уровня структуры
}
```

> `structure_income` показывает, сколько слот **лично заработал** с покупок слотов на этой глубине ниже (линейный уровневой маркетинг-бонус), а не суммарный доход уровня.

---

## 4. 🏆 Квесты слота в `GetSlotV2`

В `MarketingSlotV2` добавлен список состояний всех квестов просматриваемого слота:

```protobuf
message MarketingSlotV2 {
    // ...
    repeated Quest quests = 10;  // состояния всех квестов слота
}

message Quest {
    enum Status {
        STATUS_UNSPECIFIED = 0;
        IN_PROGRESS = 1; // доступен: окно открыто, условие не достигнуто
        COMPLETED = 2;   // пройден: условие выполнено
        EXPIRED = 3;     // недоступен: окно истекло
    }
    message Requirement { uint32 level = 1; uint32 filled = 2; uint32 required = 3; }

    uint32 quest_id = 1;                                  // идентификатор квеста (1..6)
    Status status = 2;                                    // пройден / доступен / недоступен
    optional google.protobuf.Timestamp deadline_at = 3;  // дедлайн (нет окна → поля нет)
    repeated Requirement requirements = 4;               // прогресс по уровням
    repeated biconom.types.Price amount_rewards = 5;     // валютные вознаграждения (currency_id + сумма)
}
```

**Квест №1** (`quest_id = 1`): заполнить первую линию слота (5 слотов) и вторую линию (25 слотов)
**за 72 часа** с момента создания слота. Приз зависит от дерева (`tree_id`): USDT / WIN_COINS /
USDT_GAMING (через леджер) + WIN_TIME. Квесты 2..6 (закрытие уровней 3..7) — позже.

---

## 5. 💎 Дивидендный пул: WIN от дисконт-бонуса авто-реинвеста

### `biconom/client/dividend_pool/dividend_pool.proto`

В `GetDividendPoolResponse` добавлено отдельное поле — сколько WIN-токенов заработано
за счёт **дисконт-бонуса** при включённом авто-реинвесте:

```protobuf
message GetDividendPoolResponse {
    // ...
    optional biconom.types.DividendPool.AutoReinvestState auto_reinvest = 9;

    // Суммарный WIN, полученный как дисконт-бонус за установку авто-реинвеста (формат mantissa).
    // Эмиссия WIN сверх купленного при каждом авто-claim; в earned_win НЕ входит.
    string earned_discount_win = 10;
}
```

> ⚠️ `earned_discount_win` — это **отдельная** метрика, не пересекается с `earned_win`
> (поле 2, структурные награды). Считает только WIN, эмитированный как дисконт авто-реинвеста
> (`BonusReason::ReinvestDiscount`). При выключенном авто-реинвесте поле всегда `"0"`.

---

## 💻 Как использовать на Frontend

### Баланс WinTime в списке балансов

```javascript
const list = await WalletCurrencyServiceClient.List({});
renderCurrencyBalances(list.items);
if (list.winTime) {
    renderWinTimeBalance(list.winTime.amount); // int64, целые токены
}
```

### Отдельный экран WinTime (баланс + история)

```javascript
// Виджет баланса + превью 10 последних
const head = await WinTimeServiceClient.GetBalance({});
renderBalance(head.balance, head.items);

// Пагинация истории (до 1000 за запрос; курсор — seq последней)
const page = await WinTimeServiceClient.ListTransactions({
    cursor: lastSeq,
    sort: { direction: 'BACKWARD', limit: 1000 },
});
```

### Квесты слота в карточке слота

```javascript
const slot = await MarketingServiceClient.GetSlotV2({ id: slotId });
for (const q of slot.quests) {
    const reqs = q.requirements.map(r => `L${r.level}: ${r.filled}/${r.required}`).join(', ');
    const rewards = q.amountRewards.map(p => `${p.amount} (cur ${p.currencyId})`).join(', ');
    renderQuest({
        id: q.questId,
        status: q.status,                 // IN_PROGRESS / COMPLETED / EXPIRED
        deadline: q.deadlineAt ?? null,   // optional — может отсутствовать
        progress: reqs,
        rewards,
    });
}
```

### Блокировочные флаги дистрибьютора (админ-панель)

```javascript
// Установить только нужные биты (остальные не трогаем)
const flags = await AdminMarketingServiceClient.SetDistributorFlags({
    distributorId: 123,
    blockWithdrawal: true,
    blockTransfer: true,
});
console.log(flags); // полное новое состояние
```

---

## 📋 Сводка изменений

| Что изменилось | Где |
|---|---|
| `SetMarketingFlags` / `SetDistributorFlags` возвращают новое состояние | `admin/marketing/marketing.proto` |
| Маркетинговые флаги `allow_buy_voucher_for_others`, `allow_slot_placement` | `GetMarketingFlagsResponse` (4, 5) |
| Блок-флаги дистрибьютора `block_transfer/withdrawal/slot_placement/token_purchase/buy_voucher_*` | `Get/SetDistributorFlags*` |
| Новый сервис `WinTimeService` (`GetBalance`, `ListTransactions`) | `client/win_time/win_time.proto` |
| Типы `WinTime.Balance` / `WinTime.Transaction` (oneof details) | `types/win_time.proto` |
| Баланс WinTime в ответе списка балансов | `WalletCurrency.List.win_time` (field 2) |
| Новый RPC `Marketing.GetSlotV2` | `client/marketing/marketing.proto` |
| `MarketingSlotV2` + `LevelState` (агрегаты структуры + доход по уровням) | `types/marketing_slot.proto` |
| Модель квестов слота `repeated Quest quests` (+ Requirement, amount_rewards) | `MarketingSlotV2` (field 10) |
| WIN от дисконт-бонуса авто-реинвеста `earned_discount_win` | `GetDividendPoolResponse` (field 10) |
