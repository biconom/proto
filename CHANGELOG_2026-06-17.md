# CHANGELOG: 17 Июня 2026

Добавлен **авто-реинвест в дивидендном пуле** — пользователь выбирает тариф
(количество **циклов клейма** + **процент дисконта**), после чего клейм происходит
**автоматически**: бэк сам забирает дивиденд, покупает на него WIN и доначисляет
бонус-токены по проценту дисконта. Пока авто-реинвест активен — **ручной Claim
запрещён** (его надо скрыть/задизейблить в UI).

Изменения **полностью additive** (новые сообщения, новые RPC, новые поля с новыми
тегами) — старые экраны не ломаются.

> 🧮 **Цикл** = период блокировки бонуса (`lock_duration`, на проде ~24ч). Раньше в
> черновиках это называлось «дни» — теперь везде **`cycles`**.

---

## 🚀 Изменения в API

### `types/dividend_pool.proto` — 2 новых сообщения внутри `DividendPool`

```protobuf
message DividendPool {
    // ...существующие ServiceStatus / PendingBonus / Record...

    // Тариф авто-реинвеста из конфигурации (только для отрисовки выбора).
    message AutoReinvestTier {
        uint32 cycles = 1;            // число циклов клейма
        string discount_percent = 2; // напр. "10.0000"
    }

    // Текущее состояние авто-реинвеста пользователя.
    // Активен, пока active == true И cycles_remaining > 0.
    message AutoReinvestState {
        bool   active = 1;
        uint32 cycles = 2;             // выбрано циклов
        string discount_percent = 3;   // снэпшот на момент выбора, напр. "20.0000"
        uint32 cycles_spent = 4;       // уже отработано
        uint32 cycles_remaining = 5;   // осталось (cycles - cycles_spent)
        google.protobuf.Timestamp selected_at = 6;
    }
}
```

### `client/dividend_pool.proto` — 2 новых поля в `GetDividendPoolResponse`

```protobuf
message GetDividendPoolResponse {
    // ...поля 1–7 без изменений (purchased_win, earned_win, invested_usdt,
    //    received_usdt, bonus, status, current_daily_rate)...

    // Активен ли авто-реинвест прямо сейчас. Если true — ручной Claim запрещён.
    bool auto_reinvest_active = 8;

    // Текущее состояние авто-реинвеста (если когда-либо выбирался).
    optional biconom.types.DividendPool.AutoReinvestState auto_reinvest = 9;
}
```

> 📍 Для основного экрана пула этого достаточно: `auto_reinvest_active` + `auto_reinvest`
> приходят прямо в `GetDividendPool`, отдельный запрос делать не нужно.

### `client/dividend_pool.proto` — 2 новых RPC

```protobuf
service DividendPoolService {
    // ...GetDividendPool / ClaimDividendPool / GetDividendPoolHistory /
    //    GetMatchingBonusHistory — без изменений сигнатур...

    // Выбрать авто-реинвест по числу циклов из конфигурации.
    rpc SetAutoReinvest(SetAutoReinvestRequest) returns (SetAutoReinvestResponse);

    // Текущее состояние + доступные тарифы из конфигурации (для экрана выбора).
    rpc GetAutoReinvest(google.protobuf.Empty) returns (GetAutoReinvestResponse);
}

message SetAutoReinvestRequest {
    uint32 cycles = 1; // должно совпадать с одним из tiers
}
message SetAutoReinvestResponse {
    biconom.types.DividendPool.AutoReinvestState state = 1;
}
message GetAutoReinvestResponse {
    optional biconom.types.DividendPool.AutoReinvestState state = 1; // текущее (может отсутствовать)
    repeated biconom.types.DividendPool.AutoReinvestTier tiers = 2;  // что можно выбрать
}
```

### `admin/dividend_pool.proto` — 1 новый RPC (только админка, `ADMIN_FINANCE`)

```protobuf
rpc SetDistributorAutoReinvestStatus(SetDistributorAutoReinvestStatusRequest)
    returns (SetDistributorAutoReinvestStatusResponse);

message SetDistributorAutoReinvestStatusRequest {
    uint32 distributor_id = 1;
    bool   active = 2;
}
message SetDistributorAutoReinvestStatusResponse {
    biconom.types.DividendPool.AutoReinvestState state = 1;
}
```

> ℹ️ Пользователь **сам отменить** авто-реинвест не может — выключить статус может
> только админ через этот метод. На клиенте кнопки «отменить авто-реинвест» нет.

### `types/transaction.proto` — 1 новая карточка в `Transaction.Group.Entry.details`

Новый вариант `oneof details` для отрисовки **дисконт-бонуса авто-реинвеста** в истории:

```protobuf
message Entry {
    oneof details {
        // ...существующие 1–21 без изменений...
        ReinvestDiscountDetails reinvest_discount = 22; // НОВЫЙ
    }

    message ReinvestDiscountDetails {
        string discount_percent = 1; // снэпшот тарифа, напр. "20.0000"
        string source_win = 2;       // купленный WIN, от которого считался дисконт
    }
}
```

> Изменение **additive** — новый тег `22` в oneof. Старые клиенты, не знающие
> про него, увидят `details == None` (как и для любого незнакомого варианта) и
> отрисуют запись как обычное начисление WIN.

---

## 🔒 Главное для UI: блокировка ручного Claim

Когда `auto_reinvest_active == true`:

- **Скрыть/задизейблить** кнопку «Claim» — клейм идёт автоматически на бэке.
- Если всё-таки вызвать `ClaimDividendPool` — вернётся
  `FAILED_PRECONDITION` с кодом **`DIVIDEND_POOL_AUTO_REINVEST_ACTIVE`**.
- Когда циклы закончатся (`cycles_remaining == 0`) или админ выключит статус —
  `auto_reinvest_active` станет `false`, и ручной Claim снова доступен.

---

## 💻 Примеры для Frontend

### Экран пула (из `GetDividendPool`)

```javascript
const r = await DividendPoolServiceClient.GetDividendPool({});

if (r.autoReinvestActive) {
  hideClaimButton();                 // клейм автоматический
  const s = r.autoReinvest;          // AutoReinvestState
  renderAutoReinvestBadge({
    discount: `${parseFloat(s.discountPercent)}%`,
    progress: `${s.cyclesSpent}/${s.cycles}`,   // напр. 12/90
    remaining: s.cyclesRemaining,
  });
} else {
  showClaimButton(r.bonus);          // обычный ручной клейм
}
```

### Экран выбора тарифа (из `GetAutoReinvest`)

```javascript
const a = await DividendPoolServiceClient.GetAutoReinvest({});

renderTiers(a.tiers.map(t => ({
  cycles: t.cycles,
  discount: `${parseFloat(t.discountPercent)}%`,
})));

if (a.state) renderCurrent(a.state);   // текущий выбор

// Выбор тарифа:
async function pick(cycles) {
  try {
    const { state } = await DividendPoolServiceClient.SetAutoReinvest({ cycles });
    onPicked(state);
  } catch (e) {
    // e.code === FAILED_PRECONDITION / INVALID_ARGUMENT — см. таблицу ниже
  }
}
```

> 💡 `discount_percent` приходит **строкой** (`"20.0000"`) — парсить через
> `parseFloat`. Это процент **дополнительных WIN-токенов**, которые начисляются
> сверх купленных на каждом авто-цикле.

---

## ⚠️ Коды ошибок (gRPC Status `message`)

| Метод | gRPC code | `message` | Когда |
|---|---|---|---|
| `ClaimDividendPool` | FAILED_PRECONDITION | `DIVIDEND_POOL_AUTO_REINVEST_ACTIVE` | авто-реинвест активен → ручной клейм запрещён |
| `SetAutoReinvest` | INVALID_ARGUMENT | `DIVIDEND_POOL_AUTO_REINVEST_INVALID_CYCLES` | `cycles` нет среди `tiers` |
| `SetAutoReinvest` | FAILED_PRECONDITION | `DIVIDEND_POOL_AUTO_REINVEST_NOT_CONFIGURED` | авто-реинвест выключен в конфиге (пара WIN/USDT не задана) |
| `SetAutoReinvest` | FAILED_PRECONDITION | `DIVIDEND_POOL_NO_INVESTMENT` / `DIVIDEND_POOL_NO_DAILY_RATE` / `DIVIDEND_POOL_SERVICE_NOT_ACTIVE` | нет инвестиций / нет дневной ставки / сервис не Active |
| `SetDistributorAutoReinvestStatus` (admin) | NOT_FOUND | `DIVIDEND_POOL_AUTO_REINVEST_NOT_FOUND` | у дистрибьютора нет состояния авто-реинвеста |

---

## 🧾 Как один авто-цикл выглядит в истории транзакций

Каждый автоматический клейм порождает **до 4 НЕЗАВИСИМЫХ транзакций**, и в
`GetTransactions` они приходят **отдельными группами** (`Transaction.Group`),
каждая со своим `group_id` и своим `created_at`. Они **НЕ** объединены в одну
карточку — это намеренно, чтобы дивиденд, покупка и дисконт были видны по
отдельности. Порядок по времени: дивиденд → (матчинг) → покупка → дисконт.

| # | Группа (карточка) | Валюта | `details` вариант | Знак | Как опознать |
|---|---|---|---|:---:|---|
| 1 | Выплата дивиденда | USDT | `dividend_pool_claim` (17) | `+` | подзаголовок «Dividend Pool» |
| 2 | Матчинг-бонус спонсорам* | USDT | `split_matching_bonus` (21) | `+` | приходит **спонсорам**, не самому юзеру |
| 3 | Покупка WIN | WIN/USDT | `exchange` (10) | `+`/`−` | `trading_pair_id` + `exchange_id` |
| 4 | **Дисконт-бонус** | WIN | `reinvest_discount` (22) | `+` | **новая карточка**, `discount_percent` |

\* Группа 2 появляется только при включённом матчинге (V2) и видна **получателям
бонуса** (вышестоящим спонсорам), а не инициатору авто-реинвеста.

### Группа покупки WIN (как у обычной покупки)

Покупка WIN на полученный дивиденд проходит как **обычная торговая сделка** —
та же карточка `exchange`, те же структурные награды вышестоящим
(`trade_referral_bonus`, отдельными группами у получателей). Для пользователя
визуально это неотличимо от ручной покупки WIN. Никакого спец-флага на этой
карточке нет.

### Группа дисконт-бонуса (новое, отдельная группа)

```jsonc
// Transaction.Group
{
  "group_id": 987654,            // СВОЙ group_id, не равен группе покупки
  "created_at": "2026-06-18T...",
  "status": "SUCCESS",
  "entries": [
    {
      "currency_id": <WIN>,
      "amount": "30.00",         // фактически начисленный дисконт в WIN
      "reinvest_discount": {     // <-- идентификатор карточки
        "discount_percent": "20.0000",
        "source_win": "150.00"   // куплено WIN, 20% от него = 30 WIN
      }
    }
  ]
}
```

**Как верстальщику отличить дисконт от прочих начислений WIN:**

- Карточка опознаётся **исключительно** по выставленному `details.reinvest_discount`.
  Не привязывайтесь к сумме/валюте — только к варианту oneof.
- `amount` проводки = сам дисконт (в WIN). `source_win` — справочно (объём
  покупки, от которого считали процент): `amount = source_win × percent / 100`,
  округление **вниз**.
- `discount_percent` — строка с 4 знаками (`parseFloat` → `20`). Это **снэпшот**
  тарифа на момент выбора авто-реинвеста, не пересчитывается между циклами.
- Это **отдельная группа** в ленте — рендерить как самостоятельную карточку
  «Бонус авто-реинвеста +N WIN», рядом (по времени) с карточкой покупки WIN, но
  **не** внутри неё.

### Что НЕ приходит

- Карточки «отмены»/«возврата» дисконта не существует — дисконт не отзывается.
- Внутри группы дисконта всегда **ровно одна** видимая проводка (начисление на
  кошелёк); системное списание из org-пула на стороне клиента отфильтровано.

---

## 🛡️ Версионирование backend

Бэк: следующий релиз `core` (после **`core@1.9.74`**). Изменение схемы **additive**
(новые сообщения/RPC + поля 8–9 в `GetDividendPoolResponse` + карточка
`reinvest_discount = 22` в `transaction.proto`) — старые клиенты их игнорируют,
можно деплоить бэк до фронта.

> ⚙️ Авто-реинвест включается на бэке per-environment (`config.yaml` →
> `dividend_pool.auto_reinvest.win_usdt_exchange_id/currency_pair_id`). Пока пара
> не задана — `SetAutoReinvest` отдаёт `…_NOT_CONFIGURED`, экран выбора показывать
> не нужно (или показывать «скоро»).
