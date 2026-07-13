# CHANGELOG: 13 Июля 2026

Сводка изменений API. Три блока:
**(1)** новая подсистема **Geo** — карта геометок дистрибьюторов (`GeoService`);
**(2)** **WalletCurrencyV2** — упрощённое представление баланса (строкой вместо `Ledger`),
новые RPC `GetV2` / `ListV2`;
**(3)** **удаление `WinTimeService`** — история/баланс WinTime больше не отдаются отдельным
сервисом; баланс теперь идёт как обычная валюта в списках `WalletCurrencyService`.

---

## 1. 🗺️ Новая подсистема Geo (карта геометок)

Клиентский сервис `GeoService` для карты геометок дистрибьюторов: дистрибьютор ставит/снимает
свою метку (opt-in), получает лёгкий список чужих меток по зоне с фильтрами и отдельно
догружает полные профили. Разделение сделано ради масштаба (~1 млн точек).

### `biconom/types/geo.proto` 🆕

Переиспользуемые модели (пространство имён `Geo`):

```protobuf
message Geo {
    // Область видимости точек по иерархии (единообразно с analytics.ScopeOptions).
    message ScopeOptions {
        enum Id {
            UNSPECIFIED = 0;  // не задано → ошибка
            PERSONAL = 1;     // только сам смотрящий
            TEAM = 2;         // поддерево (с ограничением глубины)
            COMPANY_WIDE = 3; // вся компания
        }
        Id id = 1;
        optional uint32 team_depth_limit = 2; // глубина поддерева (для TEAM)
        bool include_personal = 3;            // включать себя в pins
    }

    // Координаты (градусы) — переиспользуемый тип.
    message Coordinates {
        double latitude = 1;  // −90..90
        double longitude = 2; // −180..180
    }

    // Геометка дистрибьютора.
    message Pin {
        uint32 distributor_id = 1;
        double latitude = 2;
        double longitude = 3;
        int64 win_time_balance = 4;                // баланс WinTime владельца
        google.protobuf.Timestamp updated_at = 5;
    }
}
```

> Координаты наружу — `double` (градусы). Внутри — fixed-point `i32` (`× 1e6`), деталь
> реализации не утекает. Метка без `updated_at` = «пустая» точка (метки нет).

### `biconom/client/geo/geo.proto` 🆕

```protobuf
service GeoService {
    rpc SetPin(biconom.types.Geo.Coordinates) returns (biconom.types.Geo.Pin);
    rpc RemovePin(google.protobuf.Empty) returns (google.protobuf.Empty);
    rpc ListPins(ListPinsRequest) returns (PinsResponse);
    rpc GetDistributorsInfo(DistributorsInfoRequest) returns (DistributorsInfoResponse);
}
```

- **`SetPin`** — поставить/обновить свою метку (принимает `Geo.Coordinates` напрямую).
  Невалидные координаты → `GEO_INVALID_COORDINATES`.
- **`RemovePin`** — снять метку (идемпотентно).
- **`ListPins`** — лёгкий список меток по прямоугольной зоне. Все четыре границы
  (`min/max_lat`, `min/max_lon`) **обязательны** (`optional` — чтобы отличить «не задано»
  от валидного `0.0`; отсутствие → ошибка). Границы клампятся к ±90/±180, перепутанные
  `min`/`max` нормализуются. Фильтры: баланс `≥ min_win_time` + иерархия `scope`. Только точки
  С меткой, без лимита. Ответ `PinsResponse { pins, my_pin }` — лёгкий (id + координаты +
  баланс), личная метка отдельным полем.
- **`GetDistributorsInfo`** — догрузка полных профилей по списку id (BFF): distributor +
  account (с user и аватаром) + метка + баланс. Дубликаты дедуплицируются, несуществующие
  id отбрасываются, число уникальных ≤ `LIST_LIMIT_MAX` (1000, сверх → ошибка). Порядок —
  по возрастанию `distributor_id`.

Авторизация: активная сессия + право `DISTRIBUTOR_MAP_VIEW`. Владелец — по сессии.

> Полное описание — `client/geo/geo.md` и `types/geo.md`.

---

## 2. 💰 WalletCurrencyV2 — упрощённый баланс (строкой)

Облегчённый аналог `WalletCurrency`: вместо полного объекта `Ledger` — единственный
итоговый баланс строкой. Для экранов, где нужен только конечный баланс.

### `biconom/types/wallet_currency.proto`

Добавлена модель `WalletCurrencyV2` (рядом с `WalletCurrency`):

```protobuf
message WalletCurrencyV2 {
    message Id { uint32 wallet_type_id = 1; uint32 currency_id = 2; } // ключ для GetV2
    message List { repeated WalletCurrencyV2 items = 1; }

    uint32 wallet_type_id = 1;
    uint32 currency_id = 2;
    uint32 currency_precision = 3;         // scale, с которым отформатирован balance
    string balance = 4;                    // итог posted (debit − credit), знаковая строка
    bool disabled = 5;
    uint32 disabled_operations_flags = 6;
}
```

Отличия от `WalletCurrency`:
- ключ раскрыт в **плоские** `wallet_type_id` + `currency_id` (вложенный `Id` оставлен как
  переиспользуемый тип для запроса `GetV2`);
- `Ledger ledger` → `string balance` (`posted debit − credit`) + отдельное `currency_precision`;
- **убраны** `initialized`, `created_at`, `updated_at`;
- в `WalletCurrencyV2.List` **нет** поля `win_time` — WinTime идёт обычной позицией списка
  (валюта `WIN_TIME`).

### `biconom/client/wallet_currency/wallet_currency.proto`

Добавлены V2-версии методов (V1 `Get`/`List`/`Transfer` и их ключи `WalletCurrency.Id`
не тронуты):

```protobuf
rpc GetV2(biconom.types.WalletCurrencyV2.Id) returns (biconom.types.WalletCurrencyV2);
rpc ListV2(google.protobuf.Empty) returns (biconom.types.WalletCurrencyV2.List);
```

- **`GetV2`** — V2-версия `Get`: баланс одной валюты строкой (без `Ledger`).
- **`ListV2`** — V2-версия `List`: все балансы одним ответом, без фильтров/пагинации
  (`google.protobuf.Empty` на входе). WinTime — обычной позицией (валюта `WIN_TIME`).

---

## 3. 🪙 Удаление `WinTimeService`

Клиентский сервис `WinTimeService` (методы истории/баланса WinTime) **удалён целиком**
вместе с `biconom/client/win_time/win_time.proto`.

> **Причина**: дискретные начисления WinTime (реферал / квест / админ) переехали в общий
> леджер как валюта `WIN_TIME`, а пассивная часть считается формулой в подсистеме. Отдельная
> история транзакций WinTime через gRPC больше не отдаётся.
>
> **Баланс WinTime** теперь доступен как обычная валюта:
> - в `WalletCurrencyService/List` — отдельным полем `win_time` (для совместимости);
> - в `WalletCurrencyService/ListV2` и `GetV2` — обычной позицией списка (валюта `WIN_TIME`),
>   `balance` = актуальный агрегат (пассив + реферал + ledger).

**Сохранён** тип `biconom/types/win_time.proto` (`WinTime.Balance` / `TypeStat` и др.) — он
всё ещё используется полем `win_time` в `WalletCurrency.List`.

> ⚠️ **Breaking**: клиенты, вызывавшие `WinTimeService/GetBalance`,
> `WinTimeService/ListTransactions`, `WinTimeService/ListTransactionGroups`, должны перейти
> на баланс через `WalletCurrencyService` (поле `win_time` в `List` или позиция `WIN_TIME`
> в `ListV2`/`GetV2`). История транзакций WinTime через gRPC более недоступна.

---

## 💻 Как использовать на Frontend

### Карта геометок

```javascript
// Поставить свою метку
await GeoServiceClient.SetPin({ latitude: 50.45, longitude: 30.52 });

// Лёгкий список меток в видимой зоне карты (все границы обязательны)
const view = await GeoServiceClient.ListPins({
    minLat: 45.0, maxLat: 55.0, minLon: 20.0, maxLon: 40.0,
    minWinTime: 10000,
    scope: { id: 'COMPANY_WIDE' },
});
renderPins(view.pins);       // чужие точки (id + координаты + баланс)
renderMyPin(view.myPin);     // своя метка (если есть)

// Догрузить профили только для видимых точек
const visibleIds = collectVisibleIds(view.pins);
const details = await GeoServiceClient.GetDistributorsInfo({ distributorIds: visibleIds });
// details: pins + distributors + accounts (с user и аватаром), сопоставить по id
```

### Балансы V2 + WinTime как валюта

```javascript
// Плоский список всех балансов (включая WinTime как обычную валюту)
const list = await WalletCurrencyServiceClient.ListV2({});
for (const wc of list.items) {
    renderBalance(wc.walletTypeId, wc.currencyId, wc.balance, wc.currencyPrecision);
    // WinTime придёт как позиция с currencyId валюты WIN_TIME
}

// Один баланс V2
const one = await WalletCurrencyServiceClient.GetV2({ walletTypeId: 1, currencyId: 42 });
renderBalance(one.walletTypeId, one.currencyId, one.balance, one.currencyPrecision);

// Совместимость: WinTime всё ещё доступен и в старом List (поле win_time)
const v1 = await WalletCurrencyServiceClient.List({ /* ... */ });
if (v1.winTime) renderWinTime(v1.winTime.amount);
```

---

## 📋 Сводка изменений

| Что изменилось | Где |
|---|---|
| Новая подсистема Geo: `ScopeOptions` / `Coordinates` / `Pin` | `types/geo.proto` 🆕 |
| Новый `GeoService` (SetPin / RemovePin / ListPins / GetDistributorsInfo) | `client/geo/geo.proto` 🆕 |
| Право доступа `DISTRIBUTOR_MAP_VIEW` | сервер (access control) |
| Новая модель `WalletCurrencyV2` (баланс строкой, без `Ledger`) | `types/wallet_currency.proto` |
| Новые RPC `GetV2` / `ListV2` | `client/wallet_currency/wallet_currency.proto` |
| **Удалён** `WinTimeService` (история/баланс WinTime) | `client/win_time/win_time.proto` ❌ |
| Баланс WinTime — как валюта `WIN_TIME` в `ListV2`/`GetV2` (+ поле `win_time` в `List`) | `WalletCurrencyService` |
| Тип `WinTime.Balance` сохранён (поле `win_time` в `WalletCurrency.List`) | `types/win_time.proto` |
