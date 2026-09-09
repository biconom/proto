# CHANGELOG: 9 Сентября 2026 (proto `v0.3.24` → `v0.3.26`)

Два аддитивных изменения за день: `UserFindByEmail` отдаёт список
дистрибьюторов (`v0.3.25`), и стейкинг получил промо-акцию **«токен за
депозит»** — WINZU в подарок за каждое поступление в тело депозита (`v0.3.26`).

> ✅ **Не breaking.** Только новые поля в конец существующих сообщений, новое
> событие журнала и новая карточка истории транзакций. Старые клиенты новое
> игнорируют.

## Что нужно сделать фронту — коротко

| Действие | Где подробно |
|---|---|
| Перегенерировать клиент из `.proto` | — |
| Карточка истории транзакций `staking_promo_token` | §2 |
| Событие журнала стейкинга `promo_token_received` | §3 |
| Настройки модуля: показать/редактировать `promo_winzu_rate` | §4 |
| Админка маркетинга: тумблер `staking_promo_winzu` | §5 |

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

---

## 2. 🎁 Промо-акция «токен за депозит» — карточка в истории транзакций

### `biconom/types/transaction.proto`

```protobuf
oneof details {
    // ... 4–31 без изменений ...
    StakingPromoTokenDetails staking_promo_token = 32;   // ← новое
}

message StakingPromoTokenDetails {
    uint32 deposit_id  = 1;   // депозит, поступление в который дало бонус
    string base_amount = 2;   // сумма ЭТОГО поступления в USDT
    string rate        = 3;   // применённый курс, USDT за 1 WINZU ("2.50")
}
```

Механика. Пока акция включена, при **каждом** поступлении денег в тело
депозита — открытие депозита любым путём (покупка, реинвест тела созревшего,
подарок админа) и реинвест прибыли цикла — партнёру-владельцу дарится WINZU
из орг-пула компании. Сумма проводки на WINZU-кошельке `= base_amount / rate`,
округление вниз; результат `0` проводки не создаёт. Деньги партнёра при этом
**не конвертируются** — USDT целиком уходит в тело, WINZU приходит сверху.

| Что | Как |
|---|---|
| Валюта проводки | WINZU (`Transaction.currency_id`) |
| База (`base_amount`) | открытие — сумма открытия; реинвест прибыли — реинвестированная сумма цикла; реинвест тела — всё тело нового депозита |
| Реферальная лестница с бонуса | нет |
| Дивидендный пул | не участвует: база дивидендов — только покупки WIN за USDT |
| `marketing_blocked` депозита | на бонус НЕ влияет |
| Уведомление | не отправляется |

Ретро-начисление: при выкатке партнёры получают бонус по **всем** уже
существующим депозитам (активным, созревшим, закрытым) с базой «всё тело» — та
же карточка, `base_amount` равен телу депозита.

---

## 3. 📒 Событие журнала `Staking.Event.promo_token_received`

### `biconom/types/staking.proto`

```protobuf
message Event {
    message PromoTokenReceived {
        string base_amount = 1;   // база в USDT
        string amount      = 2;   // подаренный WINZU
        string rate        = 3;   // USDT за 1 WINZU ("2.50")
        uint32 accrual_seq = 4;   // 0 — за открытие / ретро, N — за реинвест прибыли цикла N
    }
    oneof data {
        // ... 10–20 без изменений ...
        PromoTokenReceived promo_token_received = 21;   // ← новое
    }
}
```

| Событие | Деньги | `ledger_group_id` | Когда |
|---|---|---|---|
| `promo_token_received` | да | группа **бонуса**: орг-пул → WINZU-кошелёк | рядом с `deposit_opened` / `income_reinvested` того же депозита, тот же `ts` |

Группа самого поступления в тело — у соседнего события, как у
`deposit_granted`. Бонус приходит в `ListTransactions` стейкинга своей группой.

---

## 4. ⚙️ `Staking.Settings.promo_winzu_rate` и `UpdateSettings`

```protobuf
message Staking.Settings {
    // ... 1–3 без изменений ...
    string promo_winzu_rate = 4;   // ← новое: USDT за 1 WINZU, "2.50"
}

message UpdateSettingsRequest {
    // ... 1–3 без изменений ...
    optional string promo_winzu_rate = 4;   // ← новое, > 0, шаг — цент
}
```

Курс хранится в центах USDT: `"2.50"` → 250. Значение с дробными центами
отвергается `InvalidArgument`. Курс сам по себе акцию не включает — только
флаг §5. Дефолт после выкатки — `2.50`.

---

## 5. 🚩 `MarketingService.Get/SetMarketingFlags.staking_promo_winzu`

### `biconom/admin/marketing/marketing.proto`

```protobuf
message GetMarketingFlagsResponse { /* 1–5 */ bool staking_promo_winzu = 6; }
message SetMarketingFlagsRequest  { /* 1–5 */ optional bool staking_promo_winzu = 6; }
```

Глобальный тумблер акции. Выключен — новые поступления бонуса не дают, уже
выданный WINZU остаётся. Миграция при выкатке **включает** флаг после
ретро-начисления.

---

## Совместимость

| | |
|---|---|
| Номера и типы существующих полей | не изменены |
| Новые поля | `UserFindByEmailResponse.distributor_ids` (№5), `Staking.Settings.promo_winzu_rate` (№4), `UpdateSettingsRequest.promo_winzu_rate` (№4), `Get/SetMarketingFlags*.staking_promo_winzu` (№6) |
| Новые сообщения | `StakingPromoTokenDetails` (details №32), `Staking.Event.PromoTokenReceived` (data №21) |
| Старый клиент против нового бэкенда | работает; новые поля игнорируются, неизвестная карточка приходит с пустым `details`, неизвестное событие — с пустым `data` |
| Новый клиент против старого бэкенда | работает; новые поля пустые/`false`, `UpdateSettings.promo_winzu_rate` игнорируется |
