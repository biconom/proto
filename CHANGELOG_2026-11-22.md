# 2026-11-22 — proto 0.3.41 (core 1.10.64)

## Стейкинг: бонус за ранг в нескольких валютах (USDT + WINZU)

За достижение ранга теперь может платиться несколько валют. Источник сумм, валют и
режима выплаты (сразу / вручную) — секция `staking_rank_bonus` конфига ядра. Бонус в
WINZU — это тот же бонус за ранг: та же карточка истории `staking_rank_bonus`, те же
обязательства и события журнала, отличается только валюта.

Все изменения — добавление полей. Старый фронт продолжает работать; без доработки он
покажет WINZU-бонус с суммой, но может подписать его валютой стейкинга (USDT), если
берёт валюту не из проводки / `currency_id`.

### Новое

| Где | Поле | Что это |
|---|---|---|
| `Staking` | `message RankReward { currency_id = 1, amount = 2, auto_release = 3 }` | Одна выплата за ранг в одной валюте |
| `Staking.Rank` | `repeated RankReward achievement_rewards = 7` | Все выплаты за ранг. Пусто — за ранг не платится ничего |
| `Staking.RankBonusSetting` | `repeated RankReward rewards = 4` | То же для админского `ListRankBonusSettings` |
| `Staking.Event.RankBonusReleased` | `uint32 currency_id = 4` | Валюта выплаченного бонуса |

### Изменился смысл (без смены типов)

- `Staking.Rank.achievement_bonus`, `RankBonusSetting.achievement_bonus` /
  `auto_release` — только USDT-часть бонуса ранга.
- `Staking.Obligation` — за один ранг может быть несколько обязательств, по одному
  на валюту (`currency_id`). `amount` — в валюте `currency_id`.
- `Staking.State.total_achievement_bonus_received` и
  `Summary.total_achievement_bonus_paid` — только USDT. WINZU в `total_earned`
  не входит.
- `Transaction…StakingRankBonusDetails` — валюта карточки = валюта проводки.
  Выплаты «сразу» всех валют одного ранга приходят одной группой проводок.
- `biconom.admin.staking.StakingAdminService.SetRankBonusAutoRelease` — устарел,
  всегда `FAILED_PRECONDITION` + `STAKING_RANK_BONUS_CONFIG_MANAGED`.
  `ListRankBonusSettings` — read-only отражение конфига.

### Фронту

- Карточка истории `staking_rank_bonus`: валюту брать из проводки (как у прочих
  карточек), не подставлять USDT.
- Журнал стейкинга, `RankBonusReleased`: валюта — `currency_id`.
- Таблица рангов: показывать `achievement_rewards` (например «1 000 USDT + 500 WINZU»)
  вместо одного `achievement_bonus`.
