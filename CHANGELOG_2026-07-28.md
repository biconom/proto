# CHANGELOG: 28–29 Июля 2026 (proto v0.3.7, core v1.9.93)

Сводка изменений API после релиза от 26 июля. Четыре блока:

1. **История покупок WinTime-магазина** — `ListMyPurchases` (client) +
   `ListPurchases` (admin) в формате `TransactionService.History`;
   ⚠️ `Purchase` теперь возвращает транзакцию вместо `spent_wintime`.
2. **История статусов и доходов WinTime** — новые сервисы
   `biconom.client.wintime.WintimeService` (этапы пассива + доходы за
   приглашения + общий доход) и `biconom.admin.wintime.WintimeAdminService`
   (сырой журнал).
3. **Статус товара для покупателя** — `ClientProduct.status` (enum, аддитивно);
   ⚠️ уточнена семантика `purchasable_by_me` — теперь ТОЛЬКО личное условие.
4. **Чистка `types/win_time.proto`** — удалено наследие WinTime v1.

> ⚠️ **Breaking для фронта — один пункт**: в `WintimeShopService.PurchaseResponse`
> поле №3 сменило тип (`uint64 spent_wintime` → `biconom.types.Transaction.Group
> transaction`). Старый фронт, читающий `spent_wintime`, после деплоя бэкенда
> распарсит поле некорректно — обновиться нужно синхронно с релизом. Сумма
> списания теперь внутри блока: `transaction.entries[].amount` (со знаком «−»).
> Всё остальное — аддитивно или wire-безопасно.

---

## 1. 🧾 История покупок WinTime-магазина — «отфильтрованные транзакции»

Каждая покупка сохраняется на сервере со ссылкой на транзакцию списания WinTime
в MLM-леджере. Списки покупок возвращают **не внутренние записи магазина, а те
же визуальные блоки транзакций (`biconom.types.Transaction.Group`), что и
`TransactionService.History`** — аккаунт видит отфильтрованные транзакции
магазина, упакованные его глазами. Одна покупка = одна группа; детали покупки —
в `entry.details` (`WintimeShopPurchaseDetails`: `product_id`, `coupon_code`,
`slot_id`, `voucher_id`, `tree_id`); сумма entry — фактически списанная цена.
Прошлые покупки восстановлены из леджера миграцией core v01_09_92 — история
полная с самого запуска магазина.

Списки — **полные, без пагинации, фильтров и total** (покупок на одного
дистрибьютора по природе немного): один запрос → вся история сразу.

Кроме того, `WintimeShopService.Purchase` теперь возвращает финансовую
транзакцию покупки вместо `spent_wintime`:

```protobuf
message PurchaseResponse {
    biconom.types.WintimeShop.Product product = 1;
    biconom.types.WintimeShop.Delivery delivery = 2;
    // Транзакция этой покупки — блок формата TransactionService.History,
    // один и без справочников. Сумма списания — entry.amount.
    biconom.types.Transaction.Group transaction = 3; // ← заменил spent_wintime
}
```

### `biconom/client/wintime_shop/wintime_shop.proto`

Новый метод в `WintimeShopService` (покупатель — из контекста авторизации):

```protobuf
rpc ListMyPurchases(google.protobuf.Empty) returns (ListMyPurchasesResponse);

message ListMyPurchasesResponse {
    repeated biconom.types.Transaction.Group items = 1; // ВСЕ покупки, новые сверху; формат TransactionService.History
    repeated biconom.types.Account accounts = 2;        // дедуплицированные справочники
    repeated biconom.types.Distributor distributors = 3;
    repeated biconom.types.Slot slots = 4;
}
```

### `biconom/admin/wintime_shop/wintime_shop.proto`

Новый метод в `WintimeShopAdminService` — то же по произвольному дистрибьютору
(упаковка глазами ЕГО аккаунта — саппорт видит то же, что пользователь):

```protobuf
rpc ListPurchases(ListPurchasesRequest) returns (ListPurchasesResponse);

message ListPurchasesRequest {
    uint32 distributor_id = 1; // обязателен; 0 → WINTIME_SHOP_DISTRIBUTOR_REQUIRED
}
message ListPurchasesResponse {
    repeated biconom.types.Transaction.Group items = 1; // ВСЕ покупки, новые сверху
    repeated biconom.types.Account accounts = 2;
    repeated biconom.types.Distributor distributors = 3;
    repeated biconom.types.Slot slots = 4;
}
```

---

## 2. 🎭 История статусов и доходов WinTime — новые сервисы

Клиентский сервис отдаёт историю **этапами**: каждая смена статуса — с булевыми
флагами состояния и **пересчётом пассивного дохода** (заработано за этап +
нарастающий итог). Арифметика — та же, что у фактического начисления: полные
минуты интервала × множитель, отсчёт от максимума из {момент смены, регистрация,
старт пассивного дохода}; последний этап считается до текущего момента.

### `biconom/client/wintime/wintime.proto` (новый файл)

```protobuf
service WintimeService {
    // ПОЛНАЯ история этапов текущего дистрибьютора, хронологический порядок,
    // БЕЗ пагинации (смены редки — десятки записей). Последний элемент —
    // текущее состояние.
    rpc GetMaskHistory(google.protobuf.Empty) returns (MaskHistoryResponse);
}

message MaskHistoryResponse {
    message PassiveItem {
        google.protobuf.Timestamp ts = 1; // момент смены статуса
        bool flag_account_active = 2;     // зарегистрирован и не забанен
        bool flag_tree_lite_active = 3;   // активный слот в ПЕРВОМ дереве (Lite)
        bool flag_tree_pro_active = 4;    // активный слот во ВТОРОМ дереве (Pro)
        uint32 multiplier = 5;            // множитель скорости дохода этого состояния
        uint64 earned = 6;                // начислено ЗА этап (до следующей смены / до now)
        uint64 accumulated = 7;           // накоплено к КОНЦУ этапа (нарастающий итог)
    }
    // Разовый доход за личное приглашение (из транзакций леджера, reason WinTimeReferral).
    message ReferralItem {
        google.protobuf.Timestamp ts = 1;  // момент начисления
        uint32 child_distributor_id = 2;   // личник, за которого начислен бонус
        uint64 amount = 3;                 // начислено WinTime
        uint64 accumulated = 4;            // накоплено за приглашения (нарастающий итог)
    }
    uint64 total_income = 1;                  // общий доход всех источников ниже (первым — под будущие бонусы)
    repeated PassiveItem passive_items = 2;   // этапы ЛИЧНОГО ПАССИВНОГО дохода
    repeated ReferralItem referral_items = 3; // доходы за ЛИЧНЫЕ ПРИГЛАШЕНИЯ (хронология)
}
```

Два списка доходов: этапы личного пассива (`passive_items`) и бонусы за личные
приглашения (`referral_items`, каждый с нарастающим итогом `accumulated`);
`total_income` — общий доход двух источников. Прочие дискретные
начисления (квесты, корректировки) в ответ не входят. Права: Session-токен с
`Permission::WINTIME_VIEW` (стандартный набор пользователя).

### `biconom/admin/wintime/wintime.proto` (новый файл)

СЫРОЙ журнал по произвольному дистрибьютору (саппорт/диагностика): события
`Registered` + `MaskChanged` с битовой маской как есть. Только `Permission::ROOT`:

```protobuf
service WintimeAdminService {
    rpc GetMaskHistory(GetMaskHistoryRequest) returns (MaskHistoryResponse);
}
message GetMaskHistoryRequest { uint32 distributor_id = 1; } // 0 → WINTIME_DISTRIBUTOR_REQUIRED
// MaskHistoryResponse.Item: kind (REGISTERED|MASK_CHANGED), ts, mask, multiplier.
```

---

## 3. 🏷️ Статус товара для покупателя — `ClientProduct.status`

Аддитивное поле в `biconom.client.wintime_shop.ClientProduct` (возвращается
`ListProducts` / `GetProduct`) — агрегат доступности карточки глазами покупателя:

```protobuf
message ClientProduct {
    enum Status {
        STATUS_UNSPECIFIED = 0;
        STATUS_AVAILABLE = 1;    // доступен к покупке прямо сейчас
        STATUS_RESTRICTED = 2;   // нельзя по ЛИЧНОМУ условию (TREE_LICENSE: уже владел деревом)
        STATUS_COMING_SOON = 3;  // товар скрыт админом («скоро будет»)
        STATUS_OUT_OF_STOCK = 4; // доступен, но нет в наличии (квота/пул исчерпаны)
    }
    // ... product = 1, purchased_by_me = 2, purchasable_by_me = 3 (семантика уточнена — см. ниже)
    Status status = 4; // новое поле
}
```

Модель карточки — **три НЕЗАВИСИМЫХ атрибута**, из которых по приоритету
выводится статус:

| Атрибут | Поле | От чего зависит |
|---|---|---|
| Доступность (витрина) | `product.available` | только админ-переключатель |
| Наличие на складе | `product.stock_remaining` | только квота лицензии / свободные коды пула |
| Доступность пользователю | `purchasable_by_me` | только личное условие (TREE_LICENSE — не владел деревом; TEXT_COUPON — всегда `true`) |

Приоритет при пересечении условий: `COMING_SOON` → `RESTRICTED` →
`OUT_OF_STOCK` → `AVAILABLE`. Личное ограничение сильнее «нет в наличии»:
пополнение остатка такому покупателю не поможет.

⚠️ Уточнена семантика `purchasable_by_me` (поле то же, тип тот же): теперь это
ТОЛЬКО личное условие покупателя — для купона с пустым пулом оно `true`
(`status = OUT_OF_STOCK`). Раньше поле значило «можно купить прямо сейчас»
(учитывало витрину и наличие) — этот агрегат теперь несёт
`status == STATUS_AVAILABLE`; кнопку покупки стройте по нему.

---

## 4. 🧹 Чистка `biconom/types/win_time.proto` (наследие WinTime v1)

Из файла удалены мёртвые модели истории транзакций v1 — `TxType`, `TypeStat`,
`TransactionGroup`, `Transaction` (+ Details): API v1 удалён вместе со старым
движком, ни один сервис их не референсит. Осталась живая `WinTime.Balance`
(поле `win_time` в `WalletCurrency.List`), из неё убраны всегда-нулевые
legacy-поля `seq` / `group_seq` / `stats` (номера 2–4 зарезервированы —
`reserved 2, 3, 4;`). Wire-совместимо: поля и раньше приходили пустыми.

---

Совместимость: всё аддитивно (новые поля/методы/файлы) либо wire-безопасно
(чистка win_time v1), КРОМЕ одного breaking-пункта — `PurchaseResponse.transaction`
вместо `spent_wintime` (см. предупреждение в шапке; фронт обновляется синхронно
с релизом). Серверная механика — core v1.9.93 (миграции v01_09_92 — backfill
истории покупок из леджера, v01_09_93 — индекс журнала WinTime; выполняются
автоматически при старте, до начала обслуживания запросов).
