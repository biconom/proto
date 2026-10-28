# Сервис: DividendPoolAdminService

## 1. Описание

**`DividendPoolAdminService`** — административный сервис для управления глобальным состоянием дивидендного пула. Позволяет переключать режим работы модуля и просматривать агрегированную статистику.

**Статусы сервиса:**

| Статус | Генерация бонуса | Клейм бонуса | Описание |
|--------|:----------------:|:------------:|----------|
| **ACTIVE** | ✅ | ✅ | Полноценная работа |
| **PAUSED** | ❌ | ✅ | Пользователи забирают pending, новые не создаются |
| **STOPPED** | ❌ | ❌ | Всё заморожено |

---

## 2. Описание методов (RPC)

### `rpc GetStatus(Empty) returns (GetStatusResponse)`

- **Назначение**: Получить текущий статус и сводную статистику дивидендного пула.
- **Использование**: Панель управления администратора.
- **Блокировка**: Read lock.
- **Ответ `GetStatusResponse`**:
  - `status`: Текущий глобальный статус (`ACTIVE`, `PAUSED`, `STOPPED`).
  - `total_active_users`: Количество пользователей с активным pending-бонусом.
  - `latest_daily_rate`: Последний дневной процент в строковом формате (например `"0.2667"`). Пустая строка если ещё не генерировался.
  - `latest_daily_rate_date`: Дата в формате `YYYYMMDD` (например `20260226`). `0` если ещё не генерировался.
  - `total_invested`: Суммарный объём USDT, вложенный всеми пользователями (формат mantissa).
- **Авторизация**: `ADMIN_FINANCE`.

### `rpc SetStatus(SetStatusRequest) returns (Empty)`

- **Назначение**: Изменить глобальный статус сервиса дивидендного пула.
- **Использование**: Переключение режима работы. Изменение вступает в силу немедленно.
- **Блокировка**: Write lock.
- **Побочные эффекты**:
  - Статус персистится в RocksDB (CF `meta`).
  - При переключении в `PAUSED` — фоновая задача перестаёт генерировать новые дневные проценты.
  - При переключении в `STOPPED` — все клиентские операции блокируются.
- **Параметры `SetStatusRequest`**:
  - `status`: Новый статус. `UNSPECIFIED` вернёт ошибку.
- **Авторизация**: `ADMIN_FINANCE`.

### `rpc ActivateAll(Empty) returns (ActivateAllResponse)`

- **Назначение**: Массовая первая активация — сгенерировать pending-бонус для всех дистрибьюторов у которых `invested > 0`, но ещё нет `user_state`.
- **Использование**: Запуск после первичного наполнения инвестиционных данных, или после смены статуса на ACTIVE для немедленной активации всех.
- **Блокировка**: Write lock.
- **Побочные эффекты**:
  - Создаётся `UserDividendState` для каждого активированного дистрибьютора.
  - Стейты персистятся в RocksDB через worker (fire-and-forget).
- **Ответ `ActivateAllResponse`**:
  - `activated_count`: Количество дистрибьюторов, для которых был сгенерирован первый бонус.
- **Предусловия**: Статус сервиса должен быть `ACTIVE`, иначе вернёт `DIVIDEND_POOL_SERVICE_NOT_ACTIVE`.
- **Авторизация**: `ADMIN_FINANCE`.

### `rpc EnrollAutoReinvestAll(EnrollAutoReinvestAllRequest) returns (EnrollAutoReinvestAllResponse)`

- **Назначение**: Принудительно включить авто-реинвест на `cycles` циклов всем, у кого есть вложение в пул и нет активного авто-реинвеста (никогда не выбирали или срок закончился). Процент — текущий тариф конфига на `cycles` циклов.
- **Не трогает**: активных (процент и счётчик не меняются); выключенных администратором — пока не передан `include_admin_disabled = true`.
- **`dry_run = true`**: те же счётчики без изменений. Порядок в админке: прогноз → подтверждение → боевой вызов.
- **Отката нет**: выключать обратно только поштучно (`SetDistributorAutoReinvestStatus`).
- **Авторизация**: `ROOT`.

### `rpc ExtendAutoReinvestUntil(ExtendAutoReinvestUntilRequest) returns (ExtendAutoReinvestUntilResponse)`

- **Назначение**: Продлить авто-реинвест до даты. Всем, у кого последний цикл наступает **раньше** дня `until`, добавляется ровно столько циклов, чтобы последний пришёлся на этот день. У кого программа и так заканчивается в этот день или позже — не трогаются.
- **Дата**: `until` — день по UTC, строго `DD-MM-YYYY` (например `01-11-2026`). Сегодняшний день допустим.
- **Как считаются циклы**: по календарю от ближайшего цикла партнёра (разблокировки его pending; просроченный — от текущего момента), цикл — раз в `lock_duration` (на проде сутки). Прошлые выплаты на расчёт не влияют. Пример: 28.10, `until = 01-11-2026` — у партнёра должно остаться 5 циклов (28, 29, 30, 31.10 и 01.11); если осталось 2, добавятся 3.
- **Что меняется**: растёт только `cycles` — процент дисконта и отработанные циклы прежние. Закончившиеся (циклы исчерпаны) включаются снова со своим прежним процентом (`reactivated`). Партнёры без состояния авто-реинвеста не затрагиваются (`without_auto_reinvest`) — их включает `EnrollAutoReinvestAll`. Выключенные администратором пропускаются, пока не передан `include_admin_disabled = true` (тогда выключение снимается).
- **Повтор**: вызов с той же датой — no-op. Продлить ещё — вызвать с более поздней датой.
- **`dry_run = true`**: те же счётчики без изменений.
- **Ответ**: `extended`, `reactivated`, `already_until`, `admin_disabled_skipped`, `admin_disabled_extended`, `without_auto_reinvest`, `cycles_added` (сумма добавленных циклов), эхо `until` и `dry_run`.
- **Авторизация**: `ROOT`.

---

## 3. Ошибки

| Код | Константа | Описание |
|-----|-----------|----------|
| `INVALID_ARGUMENT` | `DIVIDEND_POOL_INVALID_STATUS` | Передан невалидный статус (UNSPECIFIED или неизвестное значение) |
| `INVALID_ARGUMENT` | `DIVIDEND_POOL_AUTO_REINVEST_INVALID_CYCLES` | `EnrollAutoReinvestAll`: такого срока нет среди тарифов |
| `INVALID_ARGUMENT` | `DIVIDEND_POOL_AUTO_REINVEST_INVALID_DATE` | `ExtendAutoReinvestUntil`: дата не в формате `DD-MM-YYYY` или такого дня нет |
| `INVALID_ARGUMENT` | `DIVIDEND_POOL_AUTO_REINVEST_DATE_IN_PAST` | `ExtendAutoReinvestUntil`: день уже прошёл по UTC |
| `FAILED_PRECONDITION` | `DIVIDEND_POOL_SERVICE_NOT_ACTIVE` | Пул на паузе или остановлен |
| `FAILED_PRECONDITION` | `DIVIDEND_POOL_NO_DAILY_RATE` | `EnrollAutoReinvestAll`: ещё нет дневного процента |
| `FAILED_PRECONDITION` | `DIVIDEND_POOL_AUTO_REINVEST_NOT_CONFIGURED` | Не настроена пара WIN/USDT |

---

## 4. Права доступа

`GetStatus`, `SetStatus`, `ActivateAll`, `SetDistributorAutoReinvestStatus` — permission `ADMIN_FINANCE`. Массовые изменения авто-реинвеста `EnrollAutoReinvestAll` и `ExtendAutoReinvestUntil` — только `ROOT`. Доступ только для авторизованных сессий.
