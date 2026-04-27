# Changelog: 27 апреля 2026

## biconom/client/marketing/marketing.proto

### CalculateManualPlacementPriceResponse — Система квот расстановок

#### Удалено (breaking change с обратной совместимостью)
- **Удалено** поле `biconom.types.Price price = 7` — отдельное поле стоимости упразднено.

> Тег `7` **сохранён** — поле переименовано в `money_placement` внутри `oneof payment_kind`. Старые клиенты, читавшие тег 7 как `Price`, продолжат получать корректное значение при платной расстановке.

#### Добавлено

- **Добавлено** `oneof payment_kind` — тип оплаты расстановки, заменяет бывшее поле `price`:

  | Вариант | Тег | Описание |
  |---------|-----|----------|
  | `bool free_placement` | 16 | Бесплатно: авто-расстановка или пользователь выбрал то же место, что и алгоритм |
  | `bool quota_placement` | 17 | Списана 1 единица квоты (ручная расстановка при `quota_available > 0`) |
  | `biconom.types.Price money_placement` | 7 | Списаны деньги (ручная расстановка при `quota_available == 0`) |

- **Добавлено** поле `uint32 quota_available = 18` — текущий остаток квоты бесплатных расстановок на `executor_slot_id` после расчёта.

> **Для верстальщика**:
> - Показывайте `quota_available` рядом с формой выбора позиции расстановки: «Доступно бесплатных расстановок: N».
> - Если `payment_kind = free_placement` — показывайте «Бесплатно».
> - Если `payment_kind = quota_placement` — показывайте «Бесплатно (квота)», счётчик уменьшится на 1.
> - Если `payment_kind = money_placement` — показывайте сумму из поля и кнопку оплаты.

---

### Новые RPC: история и баланс расстановок

- **Добавлен** `rpc GetPlacementBalance(GetPlacementBalanceRequest) returns (MarketingSlotPlacement.BalanceWithHistory)` —
  баланс квоты + последние N записей журнала одним запросом.

- **Добавлен** `rpc ListPlacementLog(MarketingSlotPlacement.ListLogRequest) returns (MarketingSlotPlacement.ListLogResponse)` —
  пагинированная история расстановок слота. Поддерживает cursor-пагинацию, сортировку (`Sort.direction`) и фильтрацию по типу (`filter_kinds`).

> **Для верстальщика**:
> - `GetPlacementBalance` — для виджета «Баланс квот» с краткой историей (до 20 строк).
> - `ListPlacementLog` — для полного журнала «Моя история расстановок» с пагинацией.
> - Используйте `Sort { direction: BACKWARD }` (по умолчанию) для вывода от новых к старым.

---

## biconom/types/marketing_slot.proto

### MarketingSlot.State — новое поле

- **Добавлено** `uint32 placement_quota_available = 10` — доступный остаток квоты бесплатных ручных расстановок.
  Возвращается во всех методах, использующих `MarketingSlot.State`, включая:
  - `ListOwnSlotStates` *(контекст: distributor_id)*
  - `PurchaseLicensePlan`
  - `DeactivateAutoRenewal` / `RestoreAutoRenewal`

> **Для верстальщика**: показывайте рядом с каждым слотом на дашборде: «Расстановок осталось: N».
> При `> 0` — следующая ручная расстановка в отличную от авто-позицию бесплатная (квота).
> При `== 0` — расстановка платная (см. `CalculateManualPlacementPrice`).

---

## biconom/types/license.proto

### License.Plan — новое поле

- **Добавлено** `uint32 free_placements_initial = 7` — начальная квота бесплатных ручных расстановок,
  выдаваемая каждому новому слоту при регистрации в данном дереве.
  - `0` = бесплатная квота не предоставляется.
  - Текущее значение по умолчанию: **5** для всех деревьев.

> **Для верстальщика**: используйте для отображения «При покупке вы получите N бесплатных расстановок»
> на странице выбора тарифного плана.

---

## biconom/types/marketing_slot_placement.proto *(новый файл)*

Добавлен новый тип `MarketingSlotPlacement` для системы квот и журнала расстановок.

### MarketingSlotPlacement.Kind — варианты записей

| Вариант | Описание |
|---------|----------|
| `QUOTA_GRANT = 1` | Начисление квоты (при создании слота или административно) |
| `QUOTA_CONSUME = 2` | Списание 1 единицы квоты за ручную расстановку |
| `QUOTA_PENALTY = 3` | Штрафное административное списание квоты |
| `MONEY_PLACEMENT = 4` | Платная расстановка (квота исчерпана, ручная позиция ≠ авто) |
| `FREE_MANUAL_PLACEMENT = 5` | Ручная расстановка пользователем на авто-позицию (бесплатно) |
| `AUTO_PLACEMENT = 6` | Авто-расстановка системой по истечении дедлайна |

### MarketingSlotPlacement.Balance

| Поле | Описание |
|------|----------|
| `slot_id` | Слот |
| `quota_granted` | Суммарно начислено квоты |
| `quota_used` | Суммарно потрачено квоты (Consume + Penalty) |
| `quota_available` | Текущий доступный остаток (`granted − used`) |
| `total_placements` | Всего расстановок (авто + ручные) |

### MarketingSlotPlacement.ListLogRequest

```protobuf
message ListLogRequest {
    optional uint32 slot_id = 1;      // если не указан — авторизованный slot_id
    optional uint64 cursor = 2;       // cursor-пагинация по log_id (исключительно)
    optional biconom.types.Sort sort = 3; // направление (BACKWARD по умолчанию) и лимит
    repeated Kind filter_kinds = 5;   // фильтр по типу операции
}
```

### MarketingSlotPlacement.ListLogResponse

```protobuf
message ListLogResponse {
    Balance balance = 1;
    repeated LogEntry items = 2;
    optional uint64 next_cursor = 3;  // курсор для следующей страницы
    bool has_more = 4;
}
```

---

## Логика системы квот (справка)

```
target_location не указан или совпадает с авто?
  ├── ДА  → payment_kind = free_placement  (квота не тратится)
  └── НЕТ (ручная позиция отличается):
        ├── quota_available > 0  → payment_kind = quota_placement  (−1 квота)
        └── quota_available == 0 → payment_kind = money_placement  (деньги)
```

Все расстановки логируются в истории с указанием инициатора:
- `FreeManualPlacement` — пользователь выбрал совпадающую позицию (бесплатно, инициатор: пользователь).
- `AutoPlacement` — система расставила по истечении дедлайна (инициатор: алгоритм).
- `QuotaConsume` — квота списана за ручную расстановку.
- `MoneyPlacement` — деньги списаны за ручную расстановку.

---

### Новые RPC: накопительный пул дерева

- **Добавлен** `rpc GetTreePoolBalance(biconom.types.Tree.Id) returns (biconom.types.Price)` —
  текущий баланс накопительного пула конкретного дерева в валюте плана (обычно USDT).
  - Пул пополняется при активации **платного ваучера с `policy_id = 1`**.
  - Если для дерева пул не настроен — возвращает `Price` с нулевым значением.

- **Добавлен** `rpc ListTreePoolBalances(google.protobuf.Empty) returns (ListTreePoolBalancesResponse)` —
  балансы накопительных пулов **всех деревьев** одним запросом.
  Оптимизирован для дашборда: вместо N вызовов `GetTreePoolBalance` — один запрос.
  Деревья без настроенного пула в список не включаются.

#### Новое сообщение `ListTreePoolBalancesResponse`

```protobuf
message ListTreePoolBalancesResponse {
    // Баланс накопительного пула одного дерева.
    message Balance {
        uint32 tree_id = 1;              // Идентификатор дерева
        biconom.types.Price balance = 2; // Баланс пула в валюте плана дерева
    }

    repeated Balance items = 1;
}
```

> **Для верстальщика**:
> - `GetTreePoolBalance` — для виджета баланса пула на странице конкретного дерева.
> - `ListTreePoolBalances` — для дашборда с обзором всех деревьев и их пулов.
> - `balance.amount` содержит сумму в строковом формате с учётом `precision` валюты (например `"1250.500000"` для USDT).
> - Если `items` пустой — ни у одного дерева пул ещё не пополнялся или не настроен.

#### Механика пула

```
При покупке ваучера (policy_id=1, price > 0):
  кошелёк покупателя → ORG_POOL_LICENSE_FEE      (доход компании)
                      → ORG_POOL_LICENSE_MARKETING (маркетинговый резервуар)

При активации ваучера (policy_id=1, price > 0):
  ORG_POOL_LICENSE_MARKETING → ORG_POOL_TREE_N   (накопительный пул дерева)
  ORG_POOL_LICENSE_MARKETING → выплаты по маркетингу (унилевел, бонусы)
```

