# Arena Client API — Документация

> **Пакет:** `biconom.client.arena`  
> **Сервис:** `ArenaService`  
> **Авторизация:** Требуется активная сессия дистрибьютора. Все методы возвращают `UNAUTHENTICATED` для гостевых запросов.

---

## Концепция: Арены и Циклы

**Арена** — постоянный соревновательный контейнер. В системе 4 арены:

| ID | Имя | Источник баллов |
|----|-----|-----------------|
| 1 | `wincoin` | Покупка WIN токенов (значение = сумма в WIN) |
| 2 | `win_lite` | Первая активация ваучера дистрибьютора в дереве 1 |
| 3 | `win_pro` | Первая активация ваучера дистрибьютора в дереве 2 |
| 4 | `win_ultra` | Первая активация ваучера дистрибьютора в дереве 3 |

**Цикл (раунд)** — ограниченный по времени соревновательный период.  
Каждый цикл имеет собственный **лидерборд (Board)**: таблица участников с ранжированием по набранным баллам.

### Жизненный цикл цикла (видимость клиенту)

```
[нет цикла] ──Admin──► [ACTIVE] ──Admin──► [STOPPED] ──Admin──► [FINISHED]
                          │                                        │
                          │◄────────── Admin: ResumeCycle ─────────│
                          │
                          └──Admin: FinishCycle / TTL-воркер──► [FINISHED]
                                                                    │
                              auto_renew=true (не Stopped) ─────────┘
                                         ▼
                                      [ACTIVE] (новый цикл)
```

**Клиент не управляет жизненным циклом.** Клиент только читает данные.

### Начисление баллов

Баллы начисляются **только** при выполнении условий:

| Условие | Результат |
|---------|-----------|
| Статус `ACTIVE` + `now ∈ [started_at, ends_at)` | ✅ Баллы начисляются |
| Статус `ACTIVE`, но `now < started_at` | ❌ Цикл ещё не начался |
| Статус `ACTIVE`, но `now ≥ ends_at` | ❌ Цикл истёк, ждёт завершения |
| Статус `STOPPED` | ❌ Баллы не начисляются |
| Статус `FINISHED` | ❌ Цикл завершён |

### Логика начисления баллов по аренам

**Арена 1 (`wincoin`):**
- Баллы начисляются дистрибьютору и его спонсору при покупке WIN токенов.
- `delta = количество_купленных_WIN` (в минимальных единицах валюты).

**Арены 2–4 (`win_lite`, `win_pro`, `win_ultra`):**
- Баллы начисляются **только спонсору** при первой активации ваучера его подчинённым в конкретном дереве.
- `delta = 1` (одно очко за первую активацию).
- Повторные активации у того же дистрибьютора в том же дереве — баллов не дают.

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
uint32   seq      = 2;  // 1-based порядковый номер. Последний цикл: seq == arena.total_cycles
```

> **Как получить последний цикл:**  
> `GetArena(arena_id)` — возвращает последний цикл сразу.  
> Или: `arena.total_cycles` → `GetCycle(arena_id, seq = arena.total_cycles)`.

---

## Методы: Арена

---

### `GetArena` — детальные данные арены

```
rpc GetArena(Arena.Id) returns (Arena.Card)
```

**Назначение:** Получить данные арены с последним известным циклом и лидербордом.

**Ответ `Arena.Card`:**

| Поле | Содержимое |
|------|-----------|
| `arena` | Метаданные: id, name, total_cycles |
| `cycle` | Последний цикл арены (независимо от статуса). `null` если ни одного цикла не запускалось |
| `cycle.leaderboard` | Заполнен топ-N + запись текущего дистрибьютора |
| `cycle.prizes_usdt` | Заполняется только если `Config.show_prizes=true` |
| `cycle.total_participants` | Заполняется только если `Config.show_total_participants=true` |

**Особенности:**
- Лидерборд (`leaderboard.items`) содержит **топ-N позиций** (N определяется Admin через `display_count`) плюс запись самого дистрибьютора, даже если он вне топа.
- Поле `leaderboard.my_rank` — ранг текущего дистрибьютора. `null` если он не участвует.
- `cycle` — это **последний цикл**, он может быть `FINISHED`. Смотри `cycle.status` чтобы определить текущее состояние.

**Ошибки:**

| Код | Причина |
|-----|---------|
| `NOT_FOUND` | Арена с таким ID/name не найдена |
| `UNAUTHENTICATED` | Нет активной сессии |

---

### `ListArenas` — список всех арен

```
rpc ListArenas(Empty) returns (Arena.Card.List)
```

**Назначение:** Получить все 4 арены одним запросом для отображения обзорного экрана.

**Ответ:** `Arena.Card.List` — список из 4 карточек арен.

**Отличия от `GetArena`:**

| Поведение | `GetArena` | `ListArenas` |
|---|---|---|
| `cycle` | Последний цикл | Последний цикл |
| `cycle.leaderboard` | ✅ Заполнен (топ + my_rank) | ❌ `null` (только метаданные) |

> ℹ️ `ListArenas` — лёгкий запрос для отображения карточек арен без лидербордов. Используй `GetArena` для полных данных по одной арене.

**Ошибки:**

| Код | Причина |
|-----|---------|
| `UNAUTHENTICATED` | Нет активной сессии |

---

## Методы: Циклы

---

### `GetCycle` — метаданные конкретного цикла

```
rpc GetCycle(Arena.Cycle.Id) returns (Arena.Cycle)
```

**Назначение:** Получить полные данные по конкретному циклу, включая лидерборд.

**Запрос:** `Arena.Cycle.Id` = `{ arena_id, seq }`.

**Ответ `Arena.Cycle`:**

| Поле | Доступность |
|------|------------|
| `arena_id`, `seq`, `status` | Всегда |
| `started_at`, `ends_at` | Всегда |
| `finished_at` | Только если `FINISHED` |
| `stopped_at` | Только если был Stop |
| `winners` | Только если `FINISHED` и были выплаты призов |
| `prizes_usdt` | Только если `Config.show_prizes=true` |
| `total_participants` | Только если `Config.show_total_participants=true` |
| `leaderboard` | ✅ Заполнен (топ + my_rank текущего дистрибьютора) |

**Работает для всех статусов:** `ACTIVE`, `STOPPED`, `FINISHED`.  
Для `FINISHED` цикла лидерборд возвращается из архива (данные неизменны).

**Ошибки:**

| Код | Причина |
|-----|---------|
| `NOT_FOUND` | Арена или цикл с таким seq не найден |

---

### `ListCycles` — история циклов арены

```
rpc ListCycles(ListCyclesRequest) returns (Arena.Cycle.List)
```

**Назначение:** Получить историю всех циклов арены постранично. Используется для отображения истории раундов.

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

**Конец списка:** последняя страница — та, в которой есть элемент с `seq == 1`.

**Важно:**
- `leaderboard` **не заполняется** (только метаданные цикла). Для лидерборда используй `GetCycle` или `GetLeaderboard`.
- `prizes_usdt` и `total_participants` — по настройкам `Config.display` каждого цикла.

**Ошибки:**

| Код | Причина |
|-----|---------|
| `INVALID_ARGUMENT` | Не передан `arena_id` |
| `NOT_FOUND` | Арена не найдена |

---

## Методы: Лидерборд

---

### `GetLeaderboard` — рейтинговая таблица цикла

```
rpc GetLeaderboard(Arena.Cycle.Id) returns (Leaderboard)
```

**Назначение:** Получить топ рейтинговой таблицы конкретного цикла. Основной метод для отображения лидерборда.

**Запрос:** `Arena.Cycle.Id` = `{ arena_id, seq }`.

**Ответ `Leaderboard`:**

| Поле | Содержимое |
|------|-----------|
| `items[]` | Топ-N записей, отсортированных по значению (rank ASC) |
| `my_rank` | Позиция текущего дистрибьютора. `null` если он не участвует |
| `value_precision` | Точность значений (зависит от арены) |

**Состав `items[]`:**
- Топ-N участников (N = `display_count`, настраивается Admin).
- Запись самого дистрибьютора **добавляется отдельно** если он вне топа — чтобы пользователь всегда видел свою позицию.

**`value_precision` по аренам:**

| Арена | Значение | Precision |
|-------|---------|-----------|
| `wincoin` (1) | Количество WIN токенов | precision валюты WIN |
| `win_lite` (2), `win_pro` (3), `win_ultra` (4) | Счётчик первых активаций | 0 (целое число) |

**Работает для всех статусов:** `ACTIVE`, `STOPPED`, `FINISHED` (архивный).

**Ошибки:**

| Код | Причина |
|-----|---------|
| `NOT_FOUND` | Арена или цикл не найден |

---

### `ListLeaderboardEntries` — полный список с пагинацией

```
rpc ListLeaderboardEntries(ListLeaderboardEntriesRequest) returns (Leaderboard)
```

**Назначение:** Постраничный просмотр **всех** участников рейтинга. Используется для экранов "весь список участников" и прокрутки рейтинговой таблицы.

**Запрос:**

```protobuf
Arena.Cycle.Id   cycle_id = 1;  // (arena_id, seq)
optional uint32  cursor   = 2;  // rank последнего элемента (exclusive). null = с начала
optional Sort    sort     = 3;  // по умолчанию: FORWARD, 50 записей
```

**Пагинация (cursor-based по rank):**

| Направление | Поведение |
|---|---|
| `FORWARD` (default) | rank 1 → N. `cursor` = rank последнего полученного элемента |
| `BACKWARD` | rank N → 1. `cursor` = rank последнего полученного элемента |

**Конец списка:** получено записей меньше, чем `sort.limit`.

**Ответ `Leaderboard`:**
- `items[]` — текущая страница записей.
- `my_rank` — ранг текущего дистрибьютора (всегда включается, независимо от страницы).

**Отличие от `GetLeaderboard`:**

| | `GetLeaderboard` | `ListLeaderboardEntries` |
|---|---|---|
| Пагинация | ❌ Один запрос — топ | ✅ Курсорная пагинация |
| Лимит записей | `display_count` | `sort.limit` (до 50 по умолчанию) |
| Назначение | Главный экран арены | Экран "все участники" |

**Работает для всех статусов:** `ACTIVE`, `STOPPED`, `FINISHED`.

**Ошибки:**

| Код | Причина |
|-----|---------|
| `INVALID_ARGUMENT` | Не передан `cycle_id` |
| `NOT_FOUND` | Арена или цикл не найден |

---

## Матрица методов: что заполняется

| Метод | `leaderboard` | `my_rank` | `prizes_usdt` | `winners` |
|-------|:---:|:---:|:---:|:---:|
| `GetArena` | ✅ Топ + me | ✅ | Если `show_prizes` | Если `FINISHED` |
| `ListArenas` | ❌ null | ❌ | Если `show_prizes` | Если `FINISHED` |
| `GetCycle` | ✅ Топ + me | ✅ | Если `show_prizes` | Если `FINISHED` |
| `ListCycles` | ❌ null | ❌ | Если `show_prizes` | Если `FINISHED` |
| `GetLeaderboard` | ✅ Топ + me | ✅ | — | — |
| `ListLeaderboardEntries` | ✅ Страница + me | ✅ | — | — |

---

## Форматы данных

### Суммы USDT (`prizes_usdt`, `winners[].amount_usdt`)

Передаются в виде строк. Precision определяется из БД для валюты USDT.

```
"1000"      → 1000 USDT
"500.50"    → 500.50 USDT
"0.000001"  → минимальная единица при precision=6
```

### Значения лидерборда (`value`)

- **Арена 1 (`wincoin`):** значение в WIN токенах, применяется `value_precision`.
- **Арены 2–4:** целое число (количество активаций), `value_precision = 0`.

```
// Арена 1: значение "1500000000" при precision=8 → 15 WIN
// Арена 2: значение "7" → 7 первых активаций
```

---

## Типичные сценарии использования

### Сценарий 1: Главный экран арены

```
1. GetArena({ name: "wincoin" })
   → arena.total_cycles    // текущий seq
   → cycle.status          // Active/Stopped/Finished
   → cycle.leaderboard.items    // топ-N для отображения
   → cycle.leaderboard.my_rank  // моя позиция
   → cycle.prizes_usdt     // призы (если show_prizes=true)
```

### Сценарий 2: Карточки всех арен на главной странице

```
1. ListArenas()
   → items[0..3]           // 4 арены
   → items[i].cycle.status // статус каждой
   → items[i].cycle.seq    // номер текущего цикла
   // leaderboard не заполнен — используй GetArena для деталей
```

### Сценарий 3: Экран "все участники" с прокруткой

```
1. GetArena({ id: 1 })            // узнать seq текущего цикла
2. ListLeaderboardEntries({
       cycle_id: { arena_id: {id: 1}, seq: N },
       sort: { forward: true, limit: 50 }
   })
   → items[0..49]          // места 1..50
   → my_rank               // моя позиция (всегда)

3. ListLeaderboardEntries({
       cycle_id: ...,
       cursor: 50,          // rank последнего элемента
       sort: { forward: true, limit: 50 }
   })
   → items[0..N]           // следующие места
   // Конец когда items.length < 50
```

### Сценарий 4: История раундов арены

```
1. ListCycles({ arena_id: {name: "win_lite"} })
   → items[]               // последние 20 циклов (BACKWARD)
   → items[0].seq          // последний
   → items[last].seq       // самый ранний на странице

2. ListCycles({
       arena_id: ...,
       cursor: items[last].seq,   // следующая страница
   })
   // Конец когда items содержит цикл с seq == 1
```

### Сценарий 5: Данные завершённого цикла (призёры)

```
1. GetCycle({ arena_id: {id: 2}, seq: 5 })
   → cycle.status           // FINISHED
   → cycle.winners[]        // призёры по местам
   → cycle.winners[0].distributor_id   // 1-е место
   → cycle.winners[0].amount_usdt      // "1000.000000"
   → cycle.leaderboard.items           // финальный топ
```
