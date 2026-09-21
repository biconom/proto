# CHANGELOG: 19 Сентября 2026 (proto `v0.3.27` → `v0.3.28`)

Матчинг-бонус может выплачиваться **подарочными WINZU вместо USDT**. Режим
включается флагом маркетинга; пока флаг выключен, всё работает как раньше.

> ✅ **Не breaking.** Только новые поля в конец существующих сообщений. Старые
> клиенты их игнорируют.

## Что нужно сделать фронту — коротко

| Действие | Где подробно |
|---|---|
| Перегенерировать клиент из `.proto` | — |
| Админка: переключатель `matching_bonus_winzu` в флагах маркетинга | §1 |
| История кошелька: карточка `splitMatchingBonus` в WINZU | §2 |

## 1. 🚩 Флаг `matching_bonus_winzu`

### `biconom/admin/marketing/marketing.proto`

```protobuf
message GetMarketingFlagsResponse {
    // ...поля 1–6 без изменений...
    bool matching_bonus_winzu = 7;
}

message SetMarketingFlagsRequest {
    // ...поля 1–6 без изменений...
    optional bool matching_bonus_winzu = 7;
}
```

Включён — доля матчинг-бонуса считается по прежней формуле в USDT и
конвертируется по фиксированной расчётной цене **2,50 $ за 1 WINZU** (округление
вниз). USDT на кошелёк не зачисляется, приходит WINZU как подарочный токен: в
тело дивидендного пула (`invested_usdt`) он не попадает, в `earned_win` — попадает.

## 2. 🧾 Карточка `SplitMatchingBonusDetails`

### `biconom/types/transaction.proto`

```protobuf
message SplitMatchingBonusDetails {
    // ...поля 1–7 без изменений, суммы в них по-прежнему в USDT...
    optional string payout_usdt = 8;
    optional string winzu_rate = 9;
}
```

| Поля 8–9 | Что это за проводка | Как показывать |
|---|---|---|
| отсутствуют | Матчинг в USDT (как раньше) | Без изменений |
| заполнены | Матчинг в WINZU | `amount` проводки — WINZU; подпись «≈ `payout_usdt` USDT по `winzu_rate`» |

У токена упущенной выгоды WINZU-группы `payout_usdt` = `"0"`, смысл карточки
прежний — `missed_amount`.

`GetMatchingBonusHistory` не меняется: `payout` и остальные суммы там остаются в
USDT — это расчётная история бонуса.
