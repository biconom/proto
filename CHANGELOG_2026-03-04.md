# Changelog: 4 марта 2026

## biconom/types/license.proto

### License.Plan
- **Добавлено** поле `biconom.types.Price manual_placement_price = 6` — базовая цена за ручную расстановку слота в данном дереве.

> **Для верстальщика**: Поле доступно через `ListLicensePlans` / `GetLicensePlan`. Позволяет показать стоимость ручной расстановки без дополнительных запросов. Значения: $5 для Tree 1, $20 для Tree 2 и Tree 3.

---

## biconom/client/marketing/marketing.proto

### CalculateManualPlacementPriceResponse
- **Добавлено** поле `biconom.types.Price default_price = 15` — базовая цена за ручную расстановку для дерева расставляемого слота.

> **Для верстальщика**: Поле `price` (=7) — рассчитанная стоимость конкретной расстановки (0 при совпадении с автоматической). Поле `default_price` (=15) — справочная базовая цена для данного дерева. Можно использовать для подсказки «Стоимость ручной расстановки: $5» рядом с результатом калькуляции.

---

## biconom/client/distributor/distributor.proto

### Response
- **Добавлено** поле `optional biconom.types.DividendPool.PendingBonus dividend_pool_bonus = 5` — ожидающий бонус дивидендного пула.

> **Для верстальщика**: Поле заполняется **только** когда авторизованный пользователь просматривает свой собственный профиль (executor == view) **и** статус пула не STOPPED. При просмотре чужого профиля поле отсутствует. Содержит `amount`, `base_amount`, `daily_rate`, `unlock_at` — всё что нужно для отображения «Ваш бонус: X USDT, доступен через Y».

- **Добавлено** поле `biconom.types.DividendPool.ServiceStatus.Id dividend_pool_status = 6` — глобальный статус дивидендного пула.

> **Для верстальщика**: Возвращается всегда. Значения: ACTIVE (1), PAUSED (2), STOPPED (3). При STOPPED бонус скрыт.

---

## biconom/client/dividend_pool/dividend_pool.proto

### GetDividendPoolResponse
- **Добавлено** поле `biconom.types.DividendPool.ServiceStatus.Id status = 6` — глобальный статус дивидендного пула.
- **Добавлено** поле `string current_daily_rate = 7` — текущий дневной процент.

> **Для верстальщика**: `status` показывает текущее состояние пула (ACTIVE/PAUSED/STOPPED). `current_daily_rate` — дневной процент (например `"0.2667"`), пустая строка если ещё не генерировался.

---

## biconom/admin/dividend_pool/dividend_pool.proto

### DividendPoolAdminService
- **Добавлен** метод `rpc ActivateAll(Empty) returns (ActivateAllResponse)` — массовая первая активация всех дистрибьюторов с invested > 0 без user_state.

### ActivateAllResponse
- **Добавлено** сообщение с полем `uint32 activated_count = 1`.

> **Для бэкенда**: Вызывать после смены статуса на ACTIVE для немедленной активации всех. Требует `ADMIN_FINANCE`.
