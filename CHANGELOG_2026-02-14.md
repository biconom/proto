# Changelog: 14 февраля 2026

## biconom/types/license.proto

### License (корневой уровень)
- **Удалены** `License.Id`, `License.List` — License теперь чистый контейнер для вложенных типов.
- **Удалены** корневые поля `id`, `title`, `metadata`.

### License.Plan
- **Удалены** поля `license_entity_id` и `metadata` (с перенумерацией оставшихся).
- `Plan.Id` теперь содержит **только** `tree_id` (план однозначно привязан к дереву).
- Итоговые поля: `tree_id=1`, `title=2`, `duration=3`, `price=4`.

### License.Voucher
- **Заменено** поле `plan` (ссылка на Plan) на инлайн-снимок условий:
  - `slot_id=2` — кому принадлежит ваучер.
  - `duration=3` — длительность на момент покупки.
  - `price=4` — цена на момент покупки.
- Поля перенумерованы: `id=1`, `slot_id=2`, `duration=3`, `price=4`, `activated=5`, `activated_at=6`, `created_at=7`.

### License.State
- **Удалён** статус `GRACE_PERIOD`.
- **Удалено** поле `grace_period_expires_at`.

---

## biconom/types/subscription.proto

### Subscription.State
- **Удалено** поле `plan` (ссылка на License.Plan) — план теперь определяется деревом слота.
- `next_billing_at` перенумерован: `=2` (было `=3`).

### Subscription (корневой уровень)
- **Удалены** `Subscription.List` — не используется.
- **Удалены** корневые поля `license=1` и `plans=2` — данные теперь получаются через `ListLicensePlans`.
- **Удалён** импорт `license.proto` — больше не используется.
- Subscription теперь чистый контейнер для `State`.

---

## biconom/types/marketing_slot.proto

### MarketingSlot (корневой уровень)
- **Добавлено** поле `tree_id=9` — идентификатор дерева (executor и view принадлежат ему).

### MarketingSlot.State
- **Добавлены** новые поля:
  - `tree_id=6` — идентификатор дерева, к которому относится слот.
  - `placement_required=7` (bool) — нуждается ли слот в расстановке.
  - `placement_deadline_at=8` (Timestamp) — дедлайн ручной расстановки; после него сработает автоматическая.
  - `placement_executed_at=9` (optional Timestamp) — фактическое время расстановки.

---

## biconom/client/marketing/marketing.proto

### MarketingService — переименования методов
| Было | Стало | Тип ответа |
|------|-------|------------|
| `ListSubscriptionPlans` | **`ListLicensePlans`** | `License.Plan.List` (было `Subscription`) |
| `PurchaseSubscriptionPlan` | **`PurchaseLicensePlan`** | `MarketingSlot.State` (было `MarketingSlot`) |

### MarketingService — перегруппировка методов
Методы разделены на две секции по контексту авторизации:

**`distributor_id`:**
- `ListOwnSlotStates`
- `ListLicensePlans`
- `PurchaseLicensePlan`
- `ListPendingManualPlacementSlots` ← **перенесён** из секции `slot_id`

**`slot_id`:**
- `GetSlot`
- `DeactivateAutoRenewal` / `RestoreAutoRenewal`
- `SearchSlots`
- `CalculateCapacityUpgradePrice` / `PurchaseCapacityUpgrade`
- `CalculateManualPlacementPrice` / `PurchaseManualPlacement`

### ListPendingManualPlacementSlotsRequest
- Поле `slot_id` **заменено** на `optional biconom.types.Distributor.Id distributor_id=1`.
- Если `distributor_id` не указан — используется авторизованный пользователь.
- Поля перенумерованы: `distributor_id=1`, `cursor=2`, `sort=3`.

### SearchSlotsRequest
- **Добавлено** поле `repeated uint32 tree_ids=9` — фильтр по деревьям.

### Удалён импорт
- `biconom/types/subscription.proto` — больше не используется в MarketingService.

---

## biconom/client/dictionary/dictionary.proto

### PublicResponse
- **Добавлено** поле `repeated biconom.types.Tree trees=13` — деревья теперь отдаются в публичном справочнике.

---

## biconom/types/transaction.proto

### Transaction.Status.Id
- **Добавлен** статус `REVERSAL=7` — обозначает транзакцию-возврат, которая отменяет ранее проведённую (`POSTED`) транзакцию. Отличается от `REVERSED` (который ставится на оригинальную транзакцию, которая была отменена).
