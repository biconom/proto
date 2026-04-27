# ArenaService — Клиентский сервис арен

## Описание

Сервис предоставляет доступ к турнирным аренам (лидерборд-соревнованиям).
Дистрибьюторы зарабатывают баллы (WinCoins или активации лицензий) и соревнуются
за позиции в рейтинге в рамках циклов с фиксированной длительностью.

## Контекст авторизации

Все методы требуют авторизацию от имени дистрибьютора (`distributor_id`).

## Методы

### ListArenas

Получить список всех арен с текущим циклом.

- **Request**: `Empty`
- **Response**: `ListArenasResponse` — список арен + текущие циклы

### GetArena

Детальные данные по арене: текущий цикл, топ-10, позиция текущего пользователя.

- **Request**: `Arena.Id` (по id или по имени)
- **Response**: `GetArenaResponse` — арена, цикл, топ-записи, my_entry + UI-данные

### ListCycles

Полный список циклов арены (история раундов).

- **Request**: `ListCyclesRequest` — arena_id
- **Response**: `Arena.Cycle.List` — все циклы

### GetCycle

Метаданные конкретного цикла.

- **Request**: `Arena.Cycle.Id` — arena_id + seq
- **Response**: `Arena.Cycle`

### GetLeaderboard

Рейтинговая таблица для конкретного цикла арены (краткая выжимка для интерфейса топа).

- **Request**: `GetLeaderboardRequest`
  - `arena_id` — какая арена
  - `cycle_seq` — опционально, номер цикла (по умолчанию — последний)
  - `top_limit` — опционально, количество записей (по умолчанию 10)
- **Response**: `GetLeaderboardResponse` — цикл, board, топ-записи (Leaderboard.Entry), my_entry, UI-данные

### ListLeaderboardEntries

Получить полный список участников лидерборда с пагинацией. Позволяет отобразить список тех, кто находится на определенном количестве баллов и прокручивать всю доску.

- **Request**: `ListLeaderboardEntriesRequest`
  - `arena_id`
  - `cycle_seq` (опционально)
  - `limit` и `offset` для пагинации
- **Response**: `ListLeaderboardEntriesResponse` — список Leaderboard.Entry, общее количество, UI-данные

## UI-обогащение

Все ответы с рейтинговыми данными (GetArena, GetLeaderboard, ListLeaderboardEntries) включают:
- `distributors` — данные дистрибьюторов (username, invite_code)
- `accounts` — аккаунты (для отображения имени)
- `images` — аватары участников

В данных цикла (`Arena.Cycle`) теперь содержатся:
- `prize_fund` — общий призовой фонд (mantissa)
- `winners_count` — количество призовых мест
- `winner_user_ids` — список дистрибьюторов, победивших в цикле (заполняется по его завершению)
