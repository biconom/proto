# LotteryService — лотерея «5 из 36»

Суточная лотерея: один билет на дистрибьютора на тираж, цена — WinTime
(по умолчанию 2 000), выигрыш — 5 угаданных из 5. Призовой фонд (по умолчанию
1 000 USDT на кошелёк + 2 000 USDT стейкинг-депозитом + 3 000 WINZU) делится
поровну между победителями. Шанс билета — 1 : 376 992 (C(36,5)).

## Расписание (UTC)

| Момент | Что происходит |
|---|---|
| D−1 00:00 | Тираж D открыт: опубликован `server_seed_hash`, зафиксированы фонд и цена |
| D−1 23:55 | Продажи закрыты; до 00:00 купить нельзя |
| ≈ 23:56+ | Сервер берёт внешнюю случайность (блок BSC) и считает числа (скрыто) |
| D 00:00 | Раскрытие: числа, `server_seed`, `tickets_root`, блок; начисление призов |

Тираж D = UTC-дата раскрытия YYYYMMDD.

## Вход с поддомена

1. Основной фронт (Session-токен): `CreateLaunchCode` → `code` (60 с, одноразовый).
2. Открывает `https://<поддомен-лотереи>/#code=<code>` (фрагмент не уходит на сервер).
3. Фронт лотереи: `ExchangeLaunchCode{code}` без авторизации → `LotteryToken`.
4. Дальше `authorization: Bearer <token>`. Токен работает только в
   `LotteryService` и гаснет вместе с родительской сессией.

## Алгоритм (воспроизводимый)

Обозначения: `‖` — конкатенация байт, `BE4`/`BE8` — big-endian u32/u64,
SHA-256 и HMAC-SHA256 — стандартные.

1. **server_seed** — 32 случайных байта, создаётся при открытии тиража.
   `server_seed_hash = SHA256(server_seed)` публикуется сразу.
2. **Отпечаток билета** — при покупке сервер создаёт 16 случайных байт `salt`:
   `digest = SHA256("win5t" ‖ BE4(draw_id) ‖ BE8(ticket_id) ‖ numbers[5 байт по возрастанию] ‖ salt)`.
   Соль известна только серверу, поэтому покупатель не может подогнать результат
   своим билетом. Владельцу соль отдаётся после раскрытия.
3. **tickets_root** = `SHA256(digest_1 ‖ … ‖ digest_n)`, отпечатки отсортированы
   побайтно по возрастанию; нет билетов → `SHA256("")`.
4. **entropy** — для `ENTROPY_SOURCE_BSC_BLOCK` это 32 байта хэша блока BSC
   (первый принятый блок с `timestamp ≥ sales_close_at + 30`, после 15
   подтверждений); для `ENTROPY_SOURCE_INTERNAL` — пустая строка.
5. **Поток** — блоки `B_k = HMAC-SHA256(key = server_seed,
   msg = "win5draw" ‖ BE4(draw_id) ‖ tickets_root ‖ entropy ‖ BE4(k))`,
   k = 0, 1, 2, …; из блоков подряд читаются числа u32 big-endian (8 штук на блок).
6. **Выбор 5 из 36** без смещения (частичный Фишер–Йетс):
   ```
   pool = [1, 2, …, 36]
   for i in 0..5:
       n = 36 − i
       limit = floor(2^32 / n) · n
       x = следующий u32; пока x ≥ limit: x = следующий u32
       j = i + (x mod n)
       swap(pool[i], pool[j])
   numbers = sort(pool[0..5])
   ```

Проверка игроком: `SHA256(server_seed) == server_seed_hash`; свой `digest`
пересчитывается из своего билета и `salt` и есть в `GetDrawProof.ticket_digests`;
`SHA256(отсортированные digests) == tickets_root`; хэш блока сверяется в любом
обозревателе BSC; шаги 5–6 дают те же `numbers`.

## Ошибки

| Код | gRPC | Когда |
|---|---|---|
| `LOTTERY_DISABLED` | FailedPrecondition | продажи выключены админом |
| `LOTTERY_SALES_CLOSED` | FailedPrecondition | `draw_id` не открыт сейчас (в т.ч. окно 23:55–00:00) |
| `LOTTERY_NOT_ELIGIBLE` | FailedPrecondition | не выполнена маска допуска |
| `LOTTERY_INVALID_NUMBERS` | InvalidArgument | не 5 разных чисел 1..36 |
| `LOTTERY_DRAW_NOT_FOUND` | NotFound | нет такого тиража |
| `LOTTERY_DRAW_NOT_REVEALED` | FailedPrecondition | `GetDrawProof` до раскрытия |
| `LOTTERY_LAUNCH_CODE_INVALID` | Unauthenticated | код неверный, использован или истёк |
| `LEDGER_INSUFFICIENT_FUNDS` | FailedPrecondition | не хватает WinTime |
| `ACCESS_SCOPE_DENIED` | PermissionDenied | `x-act-as`, или Lottery-токеном вызван `CreateLaunchCode` |

## Тестовый вектор

Посчитан двумя независимыми реализациями (сервер на Rust — тест
`lottery_vector_is_byte_exact` в `core/src/infra/lottery/draw_tests.rs`, и
эталон на Python `hashlib`/`hmac`). Все значения — hex, числа — по возрастанию.

Вход:

| Поле | Значение |
|---|---|
| `server_seed` | `000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f` |
| `draw_id` | `20261118` (`BE4` = `013528fe`) |
| билет 1 | `ticket_id = 1`, числа `3 7 12 25 36`, `salt = 11111111111111111111111111111111` |
| билет 2 | `ticket_id = 2`, числа `1 2 3 4 5`, `salt = 22222222222222222222222222222222` |
| билет 3 | `ticket_id = 3`, числа `5 11 18 29 33`, `salt = 000102030405060708090a0b0c0d0e0f` |

Результат:

| Величина | Значение |
|---|---|
| `server_seed_hash` | `630dcd2966c4336691125448bbb25b4ff412a49c732db2c8abc1b8581bd710dd` |
| `digest` билета 1 | `752664a8839a9305e6e3f4159e61cc0afdd91bf1c37463c6c1a9750b9045ffd7` |
| `digest` билета 2 | `769a015b9da13a2cb7f14efad13ec3859fdd2593812fde4e428c1a4e1d9ebf38` |
| `digest` билета 3 | `8f3ca6f116be36460d7334338d6f0c8cc7644dcaa5f8cc239f961f0f57c51f80` |
| `tickets_root` | `de9f86051c458c37d17a4485593a3a5b8a30cf23f464db1edc494f0be0aa1c77` |
| `numbers`, `ENTROPY_SOURCE_INTERNAL` (entropy пустая) | `15 16 17 19 36` |
| `numbers`, `ENTROPY_SOURCE_BSC_BLOCK`, entropy = `62a0fcefa976cb1c7345338a5e493cb70e154629ec15c0b7ba2bcf42c95816e0` | `5 10 13 28 30` |
| `tickets_root` без билетов (`SHA256("")`) | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Пример входа отпечатка билета 1 (`5 + 4 + 8 + 5 + 16 = 38` байт):
`77696e3574` ‖ `013528fe` ‖ `0000000000000001` ‖ `03070c1924` ‖ `11…11`.

Формат в API: `server_seed`, `server_seed_hash`, `tickets_root`, `digest`, `salt`,
`bsc_block_hash` — hex в нижнем регистре без префикса `0x` (хэш блока в
обозревателе BSC показывается с `0x` — сравнивать без него).

Отбраковка (шаг 6): для `n = 36` граница `limit = 4294967292`; значения
`4294967292…4294967295` пропускаются и берётся следующее u32 потока.
