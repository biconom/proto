# Сервис: MarketingService

## 1. Описание

**`MarketingService`** — это основной сервис для работы с аффилейт-маркетингом. Он предназначен для управления маркетинговыми слотами, отслеживания структуры (цепочек), мониторинга доходов и управления жизненным циклом лицензий и подписок.

## 2. Методы сервиса (RPC)

### Контекст авторизации: `distributor_id`

Методы, требующие авторизации от имени дистрибьютора.

- **`ListOwnSlotStates`**: Возвращает список состояний всех слотов текущего дистрибьютора. Каждый элемент (`MarketingSlot.State`) теперь включает поле `placement_quota_available` — текущий остаток бесплатных расстановок на каждом слоте. Используется для дашборда.
- **`GetLicensePlan`**: Возвращает конкретный тарифный план по идентификатору (`License.Plan.Id` → `tree_id`). Включает поле `free_placements_initial` — количество бесплатных расстановок, выдаваемых каждому новому слоту в этом дереве.
- **`ListLicensePlans`**: Возвращает список доступных тарифных планов лицензии (`License.Plan.List`).
- **`PurchaseLicensePlan`**: Приобретает тарифный план лицензии. Принимает `tree_id` (дерево) и `quantity` (количество ваучеров). Возвращает обновлённое состояние слота (`MarketingSlot.State`).
- **`ListPendingManualPlacementSlots`**: Возвращает список слотов, ожидающих ручной расстановки. Опционально принимает `distributor_id` и `tree_id` для фильтрации.
- **`ListPotentialCompressionSlots`**: Возвращает список потенциальных слотов, которые могут попасть к дистрибьютору благодаря компрессии неактивных уровней снизу. Опционально принимает `tree_id` для фильтрации.

Оба метода (`ListPendingManualPlacementSlots` и `ListPotentialCompressionSlots`) возвращают единый тип `ListUnplacedSlotsResponse`, содержащий список `UnplacedSlot` (с `slot_id` и `auto_placement_at`) и связанные данные для обогащения интерфейса.

### Контекст авторизации: `slot_id`

Методы, требующие авторизации от имени конкретного слота.

- **`GetSlot`**: Возвращает полную информацию о выбранном слоте по его идентификатору. Ответ включает в себя состояние, структуру цепочек и связанные данные для отображения.
- **`DeactivateAutoRenewal` / `RestoreAutoRenewal`**: Управление флагом автоматического продления подписки для текущего слота. Возвращают актуальное состояние биллинга и лицензии.
- **`SearchSlots`**: Осуществляет поиск слотов в иерархии по префиксу логина дистрибьютора-владельца. Поддерживает гибкую фильтрацию:
  - **`filter_distributor_relationship_state_kinds`**: Фильтр по реферальной связи.
  - **`filter_slot_relationship_state_kinds`**: Фильтр по маркетинговой связи.
  - **`filter_distributor_depth_limit_up` / `filter_distributor_depth_limit_down`**: Ограничение глубины поиска по иерархии дистрибьюторов.
  - **`filter_slot_depth_limit_up` / `filter_slot_depth_limit_down`**: Ограничение глубины поиска по иерархии слотов.
  - **`tree_ids`**: Фильтр по идентификаторам деревьев.
- **`CalculateCapacityUpgradePrice`**: Рассчитывает стоимость увеличения количества мест в первой линии.
- **`PurchaseCapacityUpgrade`**: Осуществляет покупку дополнительных мест в первой линии. Возвращает `biconom.types.Slot`.
- **`CalculateManualPlacementPrice`**: Предпросмотр и расчёт стоимости расстановки. Возвращает `CalculateManualPlacementPriceResponse`.
  - Без `target_location`: авто-прогноз (всегда бесплатно).
  - С `target_location`: расчёт с учётом системы квот.
  Ответ содержит `payment_kind` (oneof) и `quota_available`.
- **`PurchaseManualPlacement`**: Фактическая расстановка слота. Использует те же аргументы. При необходимости списывает квоту или деньги. Возвращает `biconom.types.MarketingSlot`.
- **`GetPlacementBalance`**: Возвращает `MarketingSlotPlacement.BalanceWithHistory` — баланс квоты расстановок + последние `recent_limit` записей журнала. Если `slot_id` не указан — используется авторизованный.
- **`ListPlacementLog`**: Пагинированная история расстановок слота (`MarketingSlotPlacement.ListLogResponse`). Поддерживает cursor-пагинацию, сортировку через `Sort` (FORWARD / BACKWARD) и фильтрацию по `filter_kinds`.

## 3. Основные модели данных

### 3.1. MarketingSlot
Центральная модель (агрегат) для отрисовки интерфейса слота:
- **`tree_id`**: Идентификатор дерева.
- **`executor_slot_id`**: Авторизованный слот.
- **`view_slot_id`**: Просматриваемый слот.
- **`breadcrumbs`**: Путь между исполнителем и просматриваемым слотом.
- **`view_chains`**: Дочерние цепочки `view_slot_id`.
- **`slot_states`, `slots`, `distributors`, `accounts`**: Связанные данные для гидратации UI.

### 3.2. MarketingSlot.State
- **`distributor_id`**: Дистрибьютор-владелец.
- **`tree_id`**: Дерево слота.
- **`license_state`**: Техническое право (лицензия).
- **`subscription_state`**: Коммерческое состояние (подписка).
- **`placement_required`**: Флаг, что слот нуждается в расстановке.
- **`placement_deadline_at`**: Дедлайн, после которого авто-расстановка.
- **`placement_executed_at`**: Время фактической расстановки.
- **`placement_quota_available`** *(новое)*: Доступный остаток квоты бесплатных ручных расстановок. `> 0` — следующая ручная расстановка в отличную от авто позицию бесплатна (спишется 1 квота). `== 0` — платная.

### 3.3. CalculateManualPlacementPriceResponse

| Поле | Тег | Описание |
|------|-----|----------|
| `executor_slot_id` | 1 | ID авторизованного слота |
| `target_slot_id` | 2 | ID расставляемого слота |
| `target_parent_id` | 3 | Целевой родитель |
| `target_parent_branch_number` | 4 | Ветка целевого родителя |
| `predicted_parent_id` | 5 | Авто-прогноз: родитель |
| `predicted_parent_branch_number` | 6 | Авто-прогноз: ветка |
| `payment_kind` | oneof | **Тип оплаты** (теги 7, 16, 17) |
| `quota_available` | 18 | Остаток квот после расчёта |
| `default_price` | 15 | Справочная цена за ручную расстановку |

#### `payment_kind` (oneof)

| Вариант | Тег | Условие |
|---------|-----|---------|
| `free_placement: bool` | 16 | Авто или совпадение позиций — бесплатно |
| `quota_placement: bool` | 17 | Ручная при `quota_available > 0` — квота |
| `money_placement: Price` | 7 | Ручная при `quota_available == 0` — деньги |

> **Обратная совместимость**: тег 7 (`money_placement`) совпадает с бывшим полем `price`. Старые клиенты увидят это поле как `Price` при платной расстановке.

> **Для верстальщика**: показывайте `quota_available` рядом с формой — «Доступно бесплатных расстановок: N».

### 3.4. MarketingSlotPlacement — журнал расстановок

*(тип: `biconom.types.MarketingSlotPlacement`)*

#### MarketingSlotPlacement.Balance
| Поле | Описание |
|------|---------|
| `slot_id` | Слот |
| `quota_granted` | Суммарно начислено квоты |
| `quota_used` | Суммарно потрачено квоты |
| `quota_available` | Текущий остаток (`granted − used`) |
| `total_placements` | Всего расстановок (авто + ручные) |

#### MarketingSlotPlacement.Kind (типы записей)
| Вариант | Описание |
|---------|----------|
| `QUOTA_GRANT` | Начисление квоты (при создании слота или административно) |
| `QUOTA_CONSUME` | Списание 1 единицы квоты за ручную расстановку |
| `QUOTA_PENALTY` | Штрафное административное списание |
| `MONEY_PLACEMENT` | Платная расстановка (квота исчерпана) |
| `FREE_MANUAL_PLACEMENT` | Ручная расстановка пользователем на авто-позицию (бесплатно) |
| `AUTO_PLACEMENT` | Авто-расстановка системой по истечении дедлайна |

#### GetPlacementBalanceRequest
```protobuf
message GetPlacementBalanceRequest {
    optional uint32 slot_id = 1;   // если не указан — авторизованный slot_id
    uint32 recent_limit = 2;        // 0 = только баланс; максимум 20
}
```

#### ListLogRequest
```protobuf
// Пример: последние 20 записей в обратном порядке
ListLogRequest {
    slot_id: 42,
    cursor: 150,            // с какого log_id (исключительно)
    sort: Sort { direction: BACKWARD, limit: 20 },
    filter_kinds: [MONEY_PLACEMENT, QUOTA_CONSUME]
}
```

## 4. Система квот бесплатных расстановок

При регистрации слот автоматически получает `N` бесплатных расстановок (`free_placements_initial` из `License.Plan`, по умолчанию = **5**).

**Приоритет оплаты:**

```
target_location не передан или совпадает с авто?
  ├── ДА  → FREE  (квота не тратится, логируется FreeManualPlacement / AutoPlacement)
  └── НЕТ (ручная в другое место):
        ├── quota_available > 0  → QUOTA  (−1 квота, логируется QuotaConsume)
        └── quota_available == 0 → MONEY  (деньги, логируется MoneyPlacement)
```

## 5. Жизненный цикл

### Покупка лицензии
При `PurchaseLicensePlan`: `License.State` и `Subscription.State` → `ACTIVE`. Повторная покупка продлевает срок через дополнительные ваучеры.

## 6. Рекомендации для Frontend

1. **Дерево**: используйте `slots`, `distributors`, `slot_states` для построения структуры.
2. **Суммы**: `Price.amount` — строка; используйте `precision` из справочника валют.
3. **Статус**: проверяйте `license_state.status`; если не `ACTIVE` — ограничивайте функционал.
4. **Квоты на дашборде**: в `ListOwnSlotStates` каждый `State` содержит `placement_quota_available`. Показывайте рядом со слотом: «Расстановок осталось: N».
5. **Квоты перед формой**: перед расстановкой показывайте `quota_available` из `CalculateManualPlacementPriceResponse`. `> 0` — бесплатно (квота), `== 0` — платно (`default_price`).
6. **История**: используйте `GetPlacementBalance` для краткого виджета + `ListPlacementLog` для полного журнала с пагинацией.

## 7. Связи с типами (biconom.types)

- **`License.State`**, **`License.Plan`** (включая `free_placements_initial`), **`Subscription.State`**, **`Price`**
- **`MarketingSlot`**, **`MarketingSlotPlacement`**, **`Slot`**, **`Distributor`**, **`Account`**

---
*Документация актуальна для версии proto-файлов от 27 апреля 2026 г.*