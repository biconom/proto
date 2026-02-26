# Сервис: DividendPoolService

## 1. Описание

**`DividendPoolService`** — клиентский сервис механики пассивного дохода. Пользователь получает ежедневные дивиденды пропорционально сумме USDT, вложенной в покупку WIN токенов.

**Жизненный цикл пользователя:**
1. **Активация** — пользователь нажимает «Активировать», первый бонус генерируется (без зачисления).
2. **Блокировка** — бонус заблокирован на 24 часа.
3. **Клейм** — после `unlock_at` пользователь забирает USDT → новый бонус генерируется автоматически.
4. **Пропущенные дни сгорают** — мотивация ежедневных визитов.

**Формула:** `bonus_amount = invested_usdt × daily_rate / 100`

---

## 2. Описание методов (RPC)

### `rpc GetDividendPool(Empty) returns (GetDividendPoolResponse)`

- **Назначение**: Получить текущее состояние дивидендного пула авторизованного пользователя.
- **Использование**: Вызывается при открытии страницы дивидендов. Возвращает агрегированные метрики и pending-бонус.
- **Блокировка**: Read lock.
- **Ответ `GetDividendPoolResponse`**:
  - `purchased_win`: Общий объём WIN, купленного через трейдинг (mantissa).
  - `earned_win`: Общий объём WIN, заработанного бонусами (зарезервировано, сейчас `"0"`).
  - `invested_usdt`: Общий объём USDT, потраченного на покупку WIN — «тело» дивидендов.
  - `received_usdt`: Общий объём USDT, полученного из дивидендного пула.
  - `bonus`: Текущий pending-бонус. `null` если дивиденды не активированы.
- **Авторизация**: Требуется (distributor).

### `rpc ClaimDividendPool(Empty) returns (ClaimDividendPoolResponse)`

- **Назначение**: Забрать начисленный бонус и/или активировать дивиденды.
- **Использование**: Кнопка «Забрать прибыль» или «Активировать».
- **Блокировка**: Write lock (создаёт финансовую транзакцию через MLM engine).
- **Сценарии**:
  
  | Состояние | Действие |
  |-----------|----------|
  | Не активирован | Генерирует первый бонус, `claimed_amount = "0"` |
  | Бонус заблокирован | Ошибка `DIVIDEND_POOL_BONUS_LOCKED` |
  | Бонус разлокирован, Active | Зачисляет USDT → генерирует новый бонус |
  | Бонус разлокирован, Paused | Зачисляет USDT, `bonus = null` |
  | Сервис Stopped | Ошибка `DIVIDEND_POOL_SERVICE_STOPPED` |

- **Финансовая транзакция**:
  - Credit: `ORG_POOL_DIVIDEND` (id=11)
  - Debit: Spot wallet пользователя
  - Reason: `BonusReason::DividendPoolBonus`
- **Ответ `ClaimDividendPoolResponse`**:
  - `bonus`: Новый pending-бонус (или `null`).
  - `claimed_amount`: Зачисленная сумма USDT (mantissa). `"0"` при первой активации.
- **Авторизация**: Требуется (distributor).

### `rpc GetDividendPoolHistory(Empty) returns (GetDividendPoolHistoryResponse)`

- **Назначение**: Получить полную историю выплат дивидендов.
- **Использование**: Страница истории / подробная статистика.
- **Блокировка**: Read lock.
- **Особенности**: Без пагинации (~365 записей/год). Читает напрямую из RocksDB.
- **Ответ `GetDividendPoolHistoryResponse`**:
  - `items`: Список записей `DividendPool.Record` в хронологическом порядке.
- **Авторизация**: Требуется (distributor).

---

## 3. Ошибки

| Код | Константа | Описание |
|-----|-----------|----------|
| `FAILED_PRECONDITION` | `DIVIDEND_POOL_SERVICE_STOPPED` | Сервис полностью остановлен |
| `FAILED_PRECONDITION` | `DIVIDEND_POOL_SERVICE_NOT_ACTIVE` | Нельзя генерировать новые бонусы |
| `FAILED_PRECONDITION` | `DIVIDEND_POOL_BONUS_LOCKED` | Бонус ещё заблокирован (unlock_at > now) |
| `FAILED_PRECONDITION` | `DIVIDEND_POOL_NO_DAILY_RATE` | Дневной процент не сгенерирован |

---

## 4. Права доступа

Все методы требуют авторизации и наличия `distributor_id > 0` у пользователя. Используется permission `TRADE_CREATE`.
