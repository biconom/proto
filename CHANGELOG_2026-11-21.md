# 2026-11-21 — proto 0.3.40 (core 1.10.62)

## Стейкинг: признак промо «токен за депозит»

`biconom.types.Staking.Settings` — новое поле:

| Поле | Что это |
|---|---|
| `bool staking_promo_winzu_enabled = 5` | Промо-акция стейкинга «токен за депозит» (WINZU за пополнение тела по `promo_winzu_rate`) сейчас включена |

Приходит везде, где есть `Settings`: клиентские `StakingService.GetState`
(`settings`) и `GetConfig` (`settings`), админские `GetConfig` и `UpdateSettings`.

Фронту: блок «токен за депозит» показывать только при `true`. Курс
`promo_winzu_rate` остаётся заполненным и после окончания акции — по нему одному
решать нельзя. Отсутствие поля (старый core) читается как `false`.

Поле только для чтения: акция включается и выключается битом `staking_promo_winzu`
в `biconom.admin.marketing.MarketingService.SetMarketingFlags` (ROOT).
С 21.11.2026 акция на проде выключена.
