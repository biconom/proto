# Changelog: 16 февраля 2026

## biconom/types/license.proto

### License.Plan
- **Добавлено** поле `bool enabled = 3` — доступен ли план к покупке в данный момент.
- Перенумерованы: `duration` → `=4`, `price` → `=5`.

---

## biconom/types/marketing_slot.proto

### MarketingSlot.DistributorState (новое)
Агрегированная статистика по дистрибьютору в разрезе деревьев:
- `distributor_id` (`=1`) — идентификатор дистрибьютора.
- `repeated TreeState tree_states` (`=2`) — состояния по каждому дереву:
  - `tree_id` (`=1`) — идентификатор дерева.
  - `slots_for_placement_count` (`=2`) — количество слотов, ожидающих ручной расстановки.
  - `slots_for_compression_count` (`=3`) — количество слотов, которые могут подняться за счёт компрессии.

### MarketingSlot.State
- **Удалено** поле `pending_manual_placement_count` — логика вынесена в `DistributorState.TreeState`.
- Перенумерованы: `tree_id` → `=5`, `placement_required` → `=6`, `placement_deadline_at` → `=7`, `placement_executed_at` → `=8`.

### MarketingSlot (корневой уровень)
- **Добавлено** поле `repeated DistributorState distributor_states = 10` — состояния дистрибьюторов с разбивкой по деревьям.

---

## biconom/client/marketing/marketing.proto

### MarketingService — новые методы

| Метод | Описание |
|-------|----------|
| `GetLicensePlan` | Получить тарифный план по идентификатору (`License.Plan.Id` → `tree_id`). Возвращает `License.Plan`. |
| `ListPotentialCompressionSlots` | Список потенциальных слотов, которые могут попасть к дистрибьютору благодаря компрессии неактивных уровней снизу по иерархии. |

### PurchaseLicensePlan — изменение входного аргумента
- **Было**: `License.Plan.Id` (только `tree_id`).
- **Стало**: `PurchaseLicensePlanRequest`:
  - `tree_id` (`=1`) — идентификатор дерева, в котором приобретается план.
  - `quantity` (`=2`) — количество ваучеров для покупки за один запрос.

### ListPendingManualPlacementSlotsRequest — новый фильтр
- **Добавлено** поле `optional uint32 tree_id = 2` — фильтр по дереву. Если не указан — возвращаются слоты по всем деревьям.
- Перенумерованы: `cursor` → `=3`, `sort` → `=4`.

### Унификация ответа: ListUnplacedSlotsResponse (новое)
Единый тип ответа для `ListPendingManualPlacementSlots` и `ListPotentialCompressionSlots`.
- **Заменяет**: `ListPendingManualPlacementSlotsResponse` (удалён).
- Содержит:
  - `total_count` (`=1`) — общее количество нерасставленных слотов.
  - `repeated UnplacedSlot items` (`=2`) — список нерасставленных слотов:
    - `slot_id` (`=1`) — ID слота.
    - `auto_placement_at` (`=2`, optional) — время автоматической расстановки (если применимо).
  - Связанные данные: `slots`, `slot_states`, `distributors`, `accounts`.

### ListPotentialCompressionSlotsRequest (новое)
- `optional uint32 tree_id = 1` — фильтр по дереву.
- `optional Slot.Id cursor = 2` — курсор для пагинации.
- `optional Sort sort = 3` — параметры сортировки.

---

## biconom/client/dictionary/dictionary.proto

### PublicResponse
- **Добавлено** поле `repeated biconom.types.License.Plan license_plans = 14` — список тарифных планов лицензий в публичном справочнике.
- **Добавлен** импорт `biconom/types/license.proto`.
