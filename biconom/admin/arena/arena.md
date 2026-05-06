# Arena Admin API — Документация

> **Пакет:** `biconom.admin.arena`  
> **Сервис:** `ArenaAdminService`  
> **Авторизация:** Требуются права администратора (`Permission::ROOT`). Все методы требуют валидную сессию с правами суперпользователя. При отсутствии прав — `UNAUTHENTICATED`.

---

## Концепция: Арены и Циклы

**Арена** — постоянный соревновательный контейнер. В системе 4 арены:

| ID | Имя | Источник баллов |
|----|-----|-----------------|
| 1 | `wincoin` | Покупка WIN токенов (=сумма в WIN) |
| 2 | `win_lite` | Первая активация ваучера в дереве 1 |
| 3 | `win_pro` | Первая активация ваучера в дереве 2 |
| 4 | `win_ultra` | Первая активация ваучера в дереве 3 |

**Цикл (раунд)** — ограниченный по времени соревновательный период внутри арены.  
Каждый цикл имеет собственный лидерборд (Board), конфигурацию и историю победителей.

### Жизненный цикл цикла

```
                          ┌──────────────────────────────────┐
           StartCycle     │                                  │
[нет цикла] ──────────► [ACTIVE] ◄─── ResumeCycle ───── [STOPPED]
                            │                                 │
                            │ StopCycle                       │
                            └──────────────────────────────►──┘
                            │
                            │ FinishCycle (вручную)
                            │ ИЛИ TTL-воркер (авто по ends_at)
                            ▼
                         [FINISHED]
                            │
              auto_renew=true + цикл завершился без StopCycle
                            │
                            ▼
                         [ACTIVE] (новый цикл, seq+1)
```

### Поведение add_score по статусу

| Статус цикла | Начисляются ли баллы? |
|---|---|
| `ACTIVE` + `now ∈ [started_at, ends_at)` | ✅ Да |
| `ACTIVE` + `now < started_at` | ❌ Нет (цикл ещё не начался) |
| `ACTIVE` + `now ≥ ends_at` | ❌ Нет (истёк, ждёт TTL-воркер) |
| `STOPPED` | ❌ Нет |
| `FINISHED` | ❌ Нет |

---

## Идентификаторы

### `Arena.Id` — идентификатор арены

```protobuf
oneof identifier {
    uint32 id   = 1;  // числовой: 1, 2, 3, 4
    string name = 2;  // строковый: "wincoin", "win_lite", "win_pro", "win_ultra"
}
```

### `Arena.Cycle.Id` — идентификатор цикла

```protobuf
Arena.Id arena_id = 1;
uint32   seq      = 2;  // 1-based порядковый номер. seq последнего == arena.total_cycles
```

> **Как получить последний цикл:**  
> Узнать `arena.total_cycles` → передать `seq = arena.total_cycles`.  
> Либо использовать `GetArena` — он возвращает последний известный цикл.

---

## Методы: Арена

---

### `GetArena` — детальные данные арены

```
rpc GetArena(Arena.Id) returns (ArenaAdminCard)
```

**Назначение:** Получить шаблон арены + последний известный цикл (независимо от статуса).

**Ответ `ArenaAdminCard`:**

| Поле | Содержимое |
|------|-----------|
| `arena` | Метаданные арены (id, name, total_cycles) |
| `template` | Конфигурация шаблона: duration, auto_renew, prizes, display |
| `cycle` | Последний цикл с лидербордом. `null` если ни одного цикла не запускалось |

**Особенности:**
- `cycle.leaderboard` — заполнен для администратора (не ограничен `display_count`).
- `cycle` — это **последний цикл** (не обязательно активный). Может быть `FINISHED`.
- Чтобы понять текущее состояние — смотри `cycle.status`.

**Ошибки:**

| Код | Причина |
|-----|---------|
| `NOT_FOUND` | Арена с таким ID/name не найдена |
| `UNAUTHENTICATED` | Нет прав ROOT |

---

### `ListArenas` — все арены

```
rpc ListArenas(Empty) returns (ArenaAdminCard.List)
```

**Назначение:** Получить все 4 арены одним запросом. Каждая карточка содержит шаблон + последний цикл.

**Особенности:**
- Аналогично `GetArena` × 4.
- `cycle.leaderboard` заполнен для каждой арены.

---

### `UpdateTemplate` — обновить шаблон арены

```
rpc UpdateTemplate(UpdateTemplateRequest) returns (Arena.Template)
```

**Назначение:** Изменить шаблон арены, который применяется при старте **следующих** циклов. Текущий активный цикл не затрагивается.

**Запрос `UpdateTemplateRequest`:**

```protobuf
Arena.Id          arena_id = 1;
Arena.Template    template = 2;
```

**Поля шаблона:**

| Поле | Тип | Описание |
|------|-----|---------|
| `duration` | `uint32` (секунды) | Длительность цикла. **Обязательно > 0** |
| `auto_renew` | `bool` | Автозапуск нового цикла после завершения. `true` — только если цикл завершился в статусе `ACTIVE` (не `STOPPED`) |
| `display` | `DisplayConfig` | Настройки отображения лидерборда |
| `prizes_enabled` | `bool` | Управляет копированием призов в новый цикл. `false` — при `StartCycle` цикл создаётся без призов, но сам шаблон сохраняет `prizes_usdt` |
| `prizes_usdt` | `repeated string` | Призы по местам в USDT. Индекс 0 = 1-е место. Формат: `"1000.00"` (precision USDT из БД). Сохраняется в шаблоне **всегда**, независимо от `prizes_enabled` |

**`DisplayConfig`:**

| Поле | Описание |
|------|---------|
| `display_count` | Сколько топ-позиций отдаёт API клиенту |
| `show_total_participants` | Добавлять ли поле `total_participants` в ответ |
| `show_prizes` | Отдавать ли клиенту поле `prizes_usdt` в цикле |

**Поведение по статусу текущего цикла:**

| Статус цикла | Что происходит |
|---|---|
| `ACTIVE` | ✅ Шаблон обновлён. Текущий цикл продолжается без изменений |
| `STOPPED` | ✅ Шаблон обновлён. Текущий цикл продолжается без изменений |
| `FINISHED` | ✅ Шаблон обновлён |
| Нет цикла | ✅ Шаблон обновлён |

> ⚠️ Изменение `auto_renew` в шаблоне влияет только на **следующий** TTL-воркер цикл.  
> ⚠️ Изменение `prizes_usdt` в шаблоне не меняет призы текущего цикла — используй `UpdateCycleConfig`.
> 
> **Семантика `prizes_enabled`:** это не флаг «очистить призы», а флаг «копировать призы при старте».  
> Можно выставить `prizes_enabled=false` + сохранить `prizes_usdt` — шаблон запомнит конфигурацию призов,  
> но новые циклы будут запускаться без выплат. При возврате `prizes_enabled=true` — призы восстановятся автоматически.

**Ошибки:**

| Код | Причина |
|-----|---------|
| `INVALID_ARGUMENT` | `duration == 0` или не передан `template` |
| `NOT_FOUND` | Арена не найдена |

---

## Методы: Управление циклами

---

### `StartCycle` — запустить новый цикл

```
rpc StartCycle(StartCycleRequest) returns (Arena.Card)
```

**Назначение:** Запустить новый цикл арены. Автоматически создаётся новый Board (лидерборд).

**Запрос:**

```protobuf
Arena.Id                     arena_id   = 1;
optional Timestamp           started_at = 2;  // null = сейчас
optional Timestamp           ends_at    = 3;  // null = started_at + template.duration
optional Arena.Cycle.Config  config     = 4;  // null = из шаблона
```

**Режимы запуска:**

| Параметр `config` | Поведение |
|---|---|
| `null` | Конфигурация (display + prizes) копируется из шаблона арены |
| Передан явно | Используется переданный config целиком. Шаблон игнорируется для этого цикла |

**Параметры времени:**

| Параметр | Если `null` | Если передан |
|---|---|---|
| `started_at` | `now` (серверное время) | Может быть в будущем — цикл создаётся, но баллы не начисляются до `started_at` |
| `ends_at` | `started_at + template.duration` | Произвольная дата. Должна быть > `started_at` |

**Ответ:** `Arena.Card` с новым циклом. `cycle.leaderboard = null` (лидерборд пустой).

**Поведение по статусу:**

| Текущий статус | Результат |
|---|---|
| Нет цикла | ✅ Новый цикл запущен |
| `FINISHED` | ✅ Новый цикл запущен (seq+1) |
| `ACTIVE` | ❌ `FAILED_PRECONDITION: ARENA_CYCLE_ALREADY_ACTIVE` |
| `STOPPED` | ❌ `FAILED_PRECONDITION: ARENA_CYCLE_ALREADY_ACTIVE` |

> ⚠️ Нельзя запустить цикл пока есть незавершённый (Active/Stopped). Сначала `FinishCycle`.

**Ошибки:**

| Код | Причина |
|-----|---------|
| `FAILED_PRECONDITION: ARENA_CYCLE_ALREADY_ACTIVE` | Уже есть Active или Stopped цикл |
| `INVALID_ARGUMENT: ARENA_CYCLE_ENDS_AT_IN_PAST` | `ends_at` < `started_at` |
| `NOT_FOUND` | Арена не найдена или не настроена (нет шаблона) |

---

### `StopCycle` — приостановить цикл

```
rpc StopCycle(Arena.Id) returns (Arena.Card)
```

**Назначение:** Приостановить активный цикл. Баллы перестают начисляться. Лидерборд заморожен.

**Что происходит:**
- Статус меняется `ACTIVE → STOPPED`.
- Поле `stopped_at` заполняется текущим временем.
- Начисление баллов через `add_score` отклоняется.
- TTL-воркер при достижении `ends_at` **не запускает авто-продление** (если `auto_renew=true`).
- Призы **не выплачиваются**. Цикл просто приостановлен.

**Поведение по статусу:**

| Текущий статус | Результат |
|---|---|
| `ACTIVE` | ✅ Цикл остановлен |
| `STOPPED` | ❌ `FAILED_PRECONDITION: ARENA_CYCLE_NOT_ACTIVE` |
| `FINISHED` | ❌ `FAILED_PRECONDITION: ARENA_NO_ACTIVE_CYCLE` |
| Нет цикла | ❌ `FAILED_PRECONDITION: ARENA_NO_ACTIVE_CYCLE` |

**Ответ:** `Arena.Card` с обновлённым циклом (status=STOPPED). Leaderboard не заполнен.

---

### `ResumeCycle` — возобновить цикл

```
rpc ResumeCycle(Arena.Id) returns (Arena.Card)
```

**Назначение:** Возобновить приостановленный цикл. Баллы снова начинают начисляться.

**Что происходит:**
- Статус меняется `STOPPED → ACTIVE`.
- `stopped_at` сохраняется в истории.
- TTL-воркер снова отслеживает `ends_at` и запустит авто-продление если `auto_renew=true`.

> ⚠️ Если `ends_at` уже прошёл в момент возобновления — цикл немедленно истечёт при следующем тике TTL-воркера.

**Поведение по статусу:**

| Текущий статус | Результат |
|---|---|
| `STOPPED` | ✅ Цикл возобновлён |
| `ACTIVE` | ❌ `FAILED_PRECONDITION: ARENA_NO_ACTIVE_CYCLE` |
| `FINISHED` | ❌ `FAILED_PRECONDITION: ARENA_NO_ACTIVE_CYCLE` |
| Нет цикла | ❌ `FAILED_PRECONDITION: ARENA_NO_ACTIVE_CYCLE` |

**Ответ:** `Arena.Card` с обновлённым циклом (status=ACTIVE). Leaderboard не заполнен.

---

### `FinishCycle` — завершить цикл вручную

```
rpc FinishCycle(Arena.Id) returns (Arena.Card)
```

**Назначение:** Принудительно завершить текущий цикл (Active или Stopped), выплатить призы победителям и заархивировать лидерборд.

**Последовательность действий (атомарно):**

1. Определяет `seq` текущего ongoing-цикла.
2. Собирает победителей (`collect_winners`): топ-N из лидерборда × prizes_usdt по местам.
3. Завершает цикл: статус → `FINISHED`, `finished_at` = now, Board архивируется.
4. Выплачивает призы атомарно через `write_call`:
   - Переводит сумму из `ORG_POOL_ARENA_PRIZES` → spot-кошелёк победителя (USDT).
   - Если кошелёк не существует — создаётся автоматически.
   - Если перевод не прошёл — место пропускается (warn в логах), остальные продолжаются.
5. Сохраняет `awards` в цикл (транзакция ID + paid_at).

**Важно о призах:**
- Приз выплачивается только если `prizes_usdt[place-1] > 0`.
- Если участников меньше призовых мест — невостребованные призы не выплачиваются.
- Если `prizes_enabled=false` в шаблоне → цикл был создан без призов (`prizes_usdt` пуст в `CycleConfig`) → никаких выплат.
- Новый цикл **не создаётся** автоматически при ручном завершении. Авто-продление (`auto_renew`) работает только через TTL-воркер.

**Поведение по статусу:**

| Текущий статус | Результат |
|---|---|
| `ACTIVE` | ✅ Цикл завершён, призы выплачены |
| `STOPPED` | ✅ Цикл завершён, призы выплачены |
| `FINISHED` | ❌ `FAILED_PRECONDITION: ARENA_NO_ACTIVE_CYCLE` |
| Нет цикла | ❌ `FAILED_PRECONDITION: ARENA_NO_ACTIVE_CYCLE` |

**Ответ:** `Arena.Card` с завершённым циклом (`status=FINISHED`, поле `winners` заполнено).

---

## Методы: Чтение циклов

---

### `GetCycle` — метаданные конкретного цикла

```
rpc GetCycle(Arena.Cycle.Id) returns (Arena.Cycle)
```

**Назначение:** Получить полные данные конкретного цикла по `(arena_id, seq)`, включая лидерборд.

**Ответ `Arena.Cycle`:**

| Поле | Доступность |
|------|------------|
| `arena_id`, `seq`, `status` | Всегда |
| `started_at`, `ends_at` | Всегда |
| `finished_at` | Только если `FINISHED` |
| `stopped_at` | Только если был Stop |
| `winners` | Только если `FINISHED` и были призы |
| `prizes_usdt` | Только если `Config.display.show_prizes=true` |
| `total_participants` | Только если `Config.display.show_total_participants=true` |
| `leaderboard` | Всегда (Admin получает без ограничений display_count) |

> ℹ️ Для `FINISHED` цикла лидерборд возвращается из архива (данные неизменны).

**Ошибки:**

| Код | Причина |
|-----|---------|
| `NOT_FOUND` | Арена или цикл с таким seq не найден |

---

### `ListCycles` — список циклов с пагинацией

```
rpc ListCycles(ListCyclesAdminRequest) returns (Arena.Cycle.List)
```

**Назначение:** Получить историю всех циклов арены постранично.

**Запрос:**

```protobuf
Arena.Id         arena_id = 1;
optional uint32  cursor   = 2;  // seq последнего элемента (exclusive). null = с самого нового
optional Sort    sort     = 3;  // по умолчанию: BACKWARD, 20 записей
```

**Пагинация (cursor-based):**

| Направление | Поведение |
|---|---|
| `BACKWARD` (default) | Новые → старые. `cursor` = seq последнего полученного элемента |
| `FORWARD` | Старые → новые. `cursor` = seq последнего полученного элемента |

**Конец списка:** последняя страница содержит цикл с `seq == 1`.

**Особенности:**
- `leaderboard` **не заполняется** в списке (только метаданные цикла).
- `prizes_usdt` и `total_participants` — по настройкам `Config.display`.
- Лимит по умолчанию: 20 записей. Максимум ограничен настройками сервера.

---

### `GetCycleConfig` — конфигурация цикла

```
rpc GetCycleConfig(Arena.Cycle.Id) returns (Arena.Cycle.Config)
```

**Назначение:** Получить конфигурацию (display + prizes) конкретного цикла.

**Ответ `Arena.Cycle.Config`:**

```protobuf
DisplayConfig    display     = 1;
repeated string  prizes_usdt = 2;  // Всегда заполнен для Admin, независимо от show_prizes
```

> ℹ️ Admin всегда видит `prizes_usdt` из конфига, даже если `show_prizes=false` (флаг влияет только на клиентский API).

**Работает для всех статусов:** `ACTIVE`, `STOPPED`, `FINISHED`.

---

### `UpdateCycleConfig` — обновить конфигурацию цикла

```
rpc UpdateCycleConfig(UpdateCycleConfigRequest) returns (Arena.Cycle.Config)
```

**Назначение:** Изменить display-настройки и/или призы конкретного цикла.

**Запрос:**

```protobuf
Arena.Cycle.Id     cycle_id = 1;  // (arena_id, seq)
Arena.Cycle.Config config   = 2;  // Полная замена конфига (не merge)
```

> ⚠️ **Полная замена:** передаются все поля config целиком, включая те что не меняются.

**Дифференцированное поведение по статусу:**

| Статус | Display (show_prizes, show_total_participants, display_count) | Prizes (prizes_usdt) |
|--------|--------------------------------------------------------------|----------------------|
| `ACTIVE` | ✅ Обновляется | ✅ Обновляется |
| `STOPPED` | ✅ Обновляется | ✅ Обновляется |
| `FINISHED` | ✅ Обновляется (для ретроспективного отображения) | ❌ **Игнорируется** (призы уже выплачены, нельзя изменить) |

> ⚠️ Для `FINISHED` цикла призы изменить **невозможно** — они уже выплачены и зафиксированы в `winners`. Display-настройки можно менять для управления отображением архивных данных.

**Ошибки:**

| Код | Причина |
|-----|---------|
| `NOT_FOUND` | Цикл не найден |
| `INVALID_ARGUMENT` | Не передан `cycle_id` или `config` |

---

## Методы: Лидерборд

---

### `GetLeaderboard` — топ лидерборда

```
rpc GetLeaderboard(Arena.Cycle.Id) returns (Leaderboard)
```

**Назначение:** Получить топ рейтинговой таблицы цикла. Admin получает полный топ **без ограничений `display_count`** (в отличие от клиентского API).

**Работает для всех статусов:** `ACTIVE`, `STOPPED`, `FINISHED` (архивный).

**Ответ `Leaderboard`:**
- `items[]` — отсортированы по убыванию значения (rank ASC).
- `my_rank` — позиция Admin-пользователя в лидерборде (по distributor_id из сессии).
- `value_precision` — точность значений (precision арены, не USDT).

> ℹ️ Для арены 1 (`wincoin`) значение — это количество WIN токенов (precision = precision валюты).  
> Для арен 2-4 значение — счётчик первых активаций (целое число).

---

### `ListLeaderboardEntries` — полный список с пагинацией

```
rpc ListLeaderboardEntries(LeaderboardAdminRequest) returns (Leaderboard)
```

**Назначение:** Постраничный доступ ко **всем** участникам лидерборда. Используется для экспорта и анализа.

**Запрос:**

```protobuf
Arena.Cycle.Id   cycle_id = 1;
optional uint32  cursor   = 2;  // rank последнего элемента (exclusive). null = с начала
optional Sort    sort     = 3;  // по умолчанию: FORWARD, 100 записей
```

**Пагинация:**
- `FORWARD` (rank 1 → N): `cursor` = rank последнего полученного элемента.
- `BACKWARD` (rank N → 1): `cursor` = rank последнего полученного элемента.
- Лимит по умолчанию: 100 записей.

**Отличие от `GetLeaderboard`:**
- `GetLeaderboard` — возвращает топ одним запросом (без пагинации).
- `ListLeaderboardEntries` — поддерживает курсорную пагинацию по всему рейтингу.

**Работает для всех статусов:** `ACTIVE`, `STOPPED`, `FINISHED`.

---

## Матрица доступности методов по статусу цикла

| Метод | Нет цикла | ACTIVE | STOPPED | FINISHED |
|-------|:---------:|:------:|:-------:|:--------:|
| `GetArena` | ✅ | ✅ | ✅ | ✅ |
| `ListArenas` | ✅ | ✅ | ✅ | ✅ |
| `UpdateTemplate` | ✅ | ✅ | ✅ | ✅ |
| `StartCycle` | ✅ | ❌ | ❌ | ✅ |
| `StopCycle` | ❌ | ✅ | ❌ | ❌ |
| `ResumeCycle` | ❌ | ❌ | ✅ | ❌ |
| `FinishCycle` | ❌ | ✅ | ✅ | ❌ |
| `GetCycle` | — | ✅ | ✅ | ✅ |
| `ListCycles` | ✅ | ✅ | ✅ | ✅ |
| `GetCycleConfig` | — | ✅ | ✅ | ✅ |
| `UpdateCycleConfig` (display) | — | ✅ | ✅ | ✅ |
| `UpdateCycleConfig` (prizes) | — | ✅ | ✅ | ❌ |
| `GetLeaderboard` | — | ✅ | ✅ | ✅ |
| `ListLeaderboardEntries` | — | ✅ | ✅ | ✅ |

---

## Форматы данных

### Суммы USDT (`prizes_usdt`)

Суммы передаются и возвращаются в виде строк с десятичной точкой.  
Precision определяется из базы данных для валюты USDT (обычно 6).

```
"1000"      → 1000 USDT
"500.50"    → 500.50 USDT
"0.000001"  → минимальная единица при precision=6
```

> ⚠️ **Не используй `8` знаков после запятой** при передаче значений — precision берётся из конфигурации валюты в БД.

### Временны́е метки (`Timestamp`)

Все временны́е поля — `google.protobuf.Timestamp` (UTC).

```protobuf
// Пример: started_at в будущем
started_at { seconds: 1746000000 }
```

---

## Типичные сценарии использования

### Сценарий 1: Первый запуск арены

```
1. UpdateTemplate(arena_id, { duration: 604800, auto_renew: true, prizes_enabled: true, prizes_usdt: ["1000", "500", "250"] })
2. StartCycle(arena_id)                  // started_at=now, ends_at=now+duration
```

### Сценарий 2: Экстренная остановка и возобновление

```
1. StopCycle(arena_id)                   // ACTIVE → STOPPED
   // ... пауза ...
2. ResumeCycle(arena_id)                 // STOPPED → ACTIVE
```

### Сценарий 3: Досрочное ручное завершение

```
1. FinishCycle(arena_id)                 // ACTIVE/STOPPED → FINISHED + выплата призов
2. StartCycle(arena_id)                  // новый цикл если нужен вручную
```

### Сценарий 4: Изменение призов активного цикла

```
1. GetCycleConfig(arena_id, seq)         // проверить текущие настройки
2. UpdateCycleConfig({ cycle_id, config: { prizes_usdt: ["2000", "1000"] } })
   // ⚠️ Меняет призы следующего FinishCycle. Уже выплаченные призы не меняются.
```

### Сценарий 5: Просмотр всех участников для аудита

```
1. GetArena(arena_id)                    // узнать seq текущего цикла
2. ListLeaderboardEntries({ cycle_id: {arena_id, seq}, sort: {limit: 100} })
3. ListLeaderboardEntries({ cycle_id, cursor: last_rank, sort: {limit: 100} })
   // Повторять пока items.length < limit
```

### Сценарий 6: Временное отключение призов без потери конфигурации

```
// Сохраняем prizes_usdt в шаблоне, но отключаем выплату для новых циклов:
1. UpdateTemplate(arena_id, { prizes_enabled: false, prizes_usdt: ["1000", "500"], duration: 604800 })
   // Шаблон сохранит ["1000", "500"], новые циклы запустятся без выплат.

// Восстановить выплаты позже:
2. UpdateTemplate(arena_id, { prizes_enabled: true, prizes_usdt: ["1000", "500"], duration: 604800 })
   // prizes_usdt уже был в шаблоне, можно передать те же значения.
```
