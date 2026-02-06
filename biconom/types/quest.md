# Quest Data Model

Модель `Quest` представляет собой контейнер для различных типов общественных заданий с поддержкой сложных условий и наград.

## Структура

Все вспомогательные модели (`Status`, `QuestStage`, `CommunityVolumeCondition` и др.) вложены внутрь сообщения `Quest`.

### Типы Квестов (kind)

#### `ExchangeTradeQuest`
Торговый квест, привязанный к бирже (`exchange_id`) и валютной паре (`currency_pair_id`).
Состоит из этапов (`Quest.ExchangeTradeQuest.Stage`).

### Этапы (Quest.ExchangeTradeQuest.Stage)
Определяют прогресс выполнения квеста.
- **Статус**: `PENDING`, `ACTIVE`, `PAUSED`, `COMPLETED`.
- **Временные метки**: `created_at`, `activated_at`, `completed_at` для аудита.
- **Условия (Condition)**: `condition_base_currency` и `condition_quote_currency` (опциональные).
- **Награда (Reward)**: `price_change` (изменение цены).

### Условия (Quest.Condition.CurrencyVolume)
Цель по накоплению общего объема торгов в конкретной валюте (`currency_id`).
- `target_amount`: Целевое значение.
- `accumulated_amount`: Текущее значение.

### Награды (Quest.ExchangeTradeQuest.Stage.Reward.PriceChange)
Награда в виде изменения курса обмена.
- `buy_price`: Новая цена покупки.
- `sell_price`: Новая цена продажи.

## Пример использования
Клиент получает объект `Quest`, проверяет `kind` (например, `exchange_trade`).
Затем итерируется по `exchange_trade.stages` для построения интерфейса прогресса. Активный этап определяет текущую цель и отображается в виде прогресс-бара.
