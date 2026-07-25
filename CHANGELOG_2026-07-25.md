# CHANGELOG: 25 Июля 2026

Сводка изменений API после релиза от 13 июля. Три блока:
**(1)** новая подсистема **WinTime Shop** — магазин товаров за WinTime-токены
(`WintimeShopService` + `WintimeShopAdminService` + модели + карточка в истории транзакций);
**(2)** **история транзакций** — новая карточка `WinTimeReferralDetails` (реферальный бонус WinTime);
**(3)** **AuthorizeTelegram** — вход/регистрация через Telegram Login Widget сразу с токеном сессии.

---

## 1. 🛍️ Новая подсистема WinTime Shop (магазин за WinTime-токены)

Дистрибьютор покупает товары за WinTime. Два семейства товаров с разной механикой:
**TREE_LICENSE** — лицензия-ваучер на слот в дереве (выдача = слот + активированный
ваучер в MLM; правило «один раз на кабинет»); **TEXT_COUPON** — уникальный текстовый
код из пула (выдача = код для копирования во внешний продукт).

Баланс покупки = леджер `WIN_TIME` (может уходить в минус) **+ личный пассив +
доход от личников** (модуль WinTime) — списание уводит только леджер-часть, не глубже пассива.

### `biconom/types/wintime_shop.proto` 🆕

Переиспользуемые модели (пространство имён `WintimeShop`):

```protobuf
message WintimeShop {
    message Product {
        message Id { oneof identifier { uint32 id = 1; string code = 2; } }
        enum Kind { KIND_UNSPECIFIED = 0; KIND_TREE_LICENSE = 1; KIND_TEXT_COUPON = 2; }
        message TreeLicenseSpec { uint32 tree_id = 1; }
        message TextCouponSpec  { string details = 1; }
        message List { repeated Product items = 1; }

        uint32 id = 1;
        string code = 2;              // уникальный неизменяемый slug [a-z0-9_]+;
                                      // название/описание локализует фронт по `code`
        Kind kind = 3;
        uint64 price_wintime = 4;
        bool available = 5;
        uint64 stock_remaining = 6;   // квота лицензий / новые коды пула
        uint64 sold_total = 7;
        google.protobuf.Timestamp created_at = 8;
        google.protobuf.Timestamp updated_at = 9;
        oneof spec { TreeLicenseSpec tree_license = 10; TextCouponSpec text_coupon = 11; }
    }

    message Delivery {
        message LicenseGrant { uint32 tree_id = 1; uint32 slot_id = 2; uint32 voucher_id = 3; }
        message CouponCode   { string code = 1; }
        oneof delivery { LicenseGrant license = 1; CouponCode coupon = 2; }
    }

    message Stats {
        message Coupons { uint64 new = 1; uint64 dispensed = 2; uint64 cancelled = 3; }
        uint32 product_id = 1;
        Product.Kind kind = 2;
        uint64 stock_remaining = 3;
        uint64 sold_total = 4;
        bool available = 5;
        Coupons coupons = 6;          // разбивка пула по статусам (только TEXT_COUPON)
    }
}
```

- Адресация ВЕЗДЕ через `Product.Id` — числовой `id` или строковый `code`
  (сервер нормализует: trim + lowercase, маска `[a-z0-9_]+`, иначе `WINTIME_SHOP_CODE_INVALID`).

### `biconom/client/wintime_shop/wintime_shop.proto` 🆕

Право — `Permission::WINTIME_SHOP_PURCHASE` (входит в стандартный набор пользователя);
покупатель — из контекста авторизации (`executor.distributor_id`), не из запроса.

```protobuf
service WintimeShopService {
    rpc ListProducts(ListProductsRequest) returns (ClientProduct.List);
    rpc GetProduct(biconom.types.WintimeShop.Product.Id) returns (ClientProduct);
    rpc Purchase(biconom.types.WintimeShop.Product.Id) returns (PurchaseResponse);
}

message ClientProduct {
    message List { repeated ClientProduct items = 1; }
    biconom.types.WintimeShop.Product product = 1;
    uint64 purchased_by_me = 2;   // сколько купил лично этот дистрибьютор
    bool purchasable_by_me = 3;   // доступно ли к покупке ИМЕННО ему сейчас
}

message PurchaseResponse {
    biconom.types.WintimeShop.Product product = 1; // актуальное состояние товара
    biconom.types.WintimeShop.Delivery delivery = 2;
    uint64 spent_wintime = 3;
}
```

- **`ListProducts`** — витрина (по умолчанию только `available`; флаг
  `include_unavailable` — все). Каждая карточка обогащена персональными полями.
- **`GetProduct`** — одна обогащённая карточка (включая скрытые — для «coming soon»).
- **`Purchase`** — покупка. Проверки по порядку: `available`
  (`WINTIME_SHOP_UNAVAILABLE`) → остаток (`WINTIME_SHOP_OUT_OF_STOCK`) → полный
  баланс WinTime (`LEDGER_INSUFFICIENT_FUNDS`) → для лицензии гейт «один раз на
  кабинет» (`WINTIME_SHOP_TREE_ALREADY_OWNED`).
- **Гейт лицензии**: если у дистрибьютора КОГДА-ЛИБО существовал слот в дереве
  товара (активный, истёкший, откреплённый, купленный напрямую — не важно) —
  повторная покупка запрещена навсегда. `purchasable_by_me` отражает это заранее.
- **Выдача лицензии** — атомарно одним коммитом: слот в дереве товара + ваучер,
  лицензия сразу активирована (без маркетинг-выплат — оплатой является WinTime).
- **Выдача купона** — уникальный код из пула товара (durable, повторная выдача
  того же кода исключена даже при сбое).

### `biconom/admin/wintime_shop/wintime_shop.proto` 🆕

Право — `Permission::ADMIN_WINTIME_SHOP`; СОЗДАНИЕ товара (`UpsertProduct` с
`id == 0`) — только `Permission::ROOT`.

```protobuf
service WintimeShopAdminService {
    rpc UpsertProduct(UpsertProductRequest) returns (biconom.types.WintimeShop.Product);
    rpc ListProducts(google.protobuf.Empty) returns (biconom.types.WintimeShop.Product.List);
    rpc SetAvailability(SetAvailabilityRequest) returns (biconom.types.WintimeShop.Product);
    rpc AdjustLicenseStock(AdjustLicenseStockRequest) returns (biconom.types.WintimeShop.Product);
    rpc LoadCoupons(LoadCouponsRequest) returns (LoadCouponsResponse);
    rpc CancelCoupons(CancelCouponsRequest) returns (CancelCouponsResponse);
    rpc ListCoupons(ListCouponsRequest) returns (ListCouponsResponse);
    rpc GetStats(biconom.types.WintimeShop.Product.Id) returns (biconom.types.WintimeShop.Stats);
}

enum CouponStatus {
    COUPON_STATUS_UNSPECIFIED = 0; // в фильтре ListCoupons = все статусы
    COUPON_STATUS_NEW = 1;         // свободен к выдаче
    COUPON_STATUS_DISPENSED = 2;   // выдан покупателю
    COUPON_STATUS_CANCELLED = 3;   // отменён администратором
}
```

- **`UpsertProduct`** — `id == 0` создать (обязателен уникальный `code`, иначе
  `WINTIME_SHOP_CODE_DUPLICATE`); `id != 0` обновить (`code` и семейство `kind`
  неизменяемы; payload `spec` и цена/доступность — обновляются). `price_wintime > 0`,
  `spec` обязан соответствовать `kind` (`WINTIME_SHOP_KIND_MISMATCH`).
- **`SetAvailability`** — тумблер витрины.
- **`AdjustLicenseStock`** — квота остатка ЛИЦЕНЗИЙ: `OP_ADD` / `OP_SUBTRACT`
  (ниже нуля не уходит — клампится к 0) / `OP_SET`. Для купона — `WINTIME_SHOP_KIND_MISMATCH`.
- **`LoadCoupons`** — массовая загрузка кодов (только купоны). Каждый код: trim,
  непустой, ≤ 64 символов (`WINTIME_SHOP_COUPON_CODE_TOO_LONG` — запрос отклоняется
  ЦЕЛИКОМ); дубли внутри запроса схлопываются. По текущему статусу в пуле:
  отсутствует → добавлен (`added`); `NEW` → игнор (`skipped_duplicates`);
  `CANCELLED` → **реактивация** в New (`reactivated`); `DISPENSED` → отклонён —
  код уже у пользователя (`skipped_dispensed`). Уникальность — в пределах товара.
- **`CancelCoupons`** — отмена НОВЫХ кодов (выданные отменить нельзя); счётчики
  `cancelled / skipped_not_found / skipped_dispensed / skipped_cancelled`.
  Вернуть отменённый в продажу — повторный `LoadCoupons`.
- **`ListCoupons`** — постраничный список кодов пула: фильтр по `CouponStatus`,
  `limit` (≤ 1000) + `offset`, лексикографический порядок. Каждый `Item`:
  `code`, `status`, `created_at` (когда загружен; реактивация не меняет),
  `updated_at` (последняя смена статуса), `distributor_id` — **кому выдан** (0 = не выдан).
- **`GetStats`** — сводка; для купонов дополнительно `Stats.Coupons`
  (`new / dispensed / cancelled`).

### `biconom/types/transaction.proto` — карточка покупки в истории

```protobuf
WintimeShopPurchaseDetails wintime_shop_purchase = 25; // в oneof details

message WintimeShopPurchaseDetails {
    uint32 product_id = 1;
    string coupon_code = 2; // выданный код (только TEXT_COUPON) — копируется из истории
    uint32 slot_id = 3;     // созданный слот (только TREE_LICENSE)
    uint32 voucher_id = 4;  // созданный ваучер-лицензия (только TREE_LICENSE)
    uint32 tree_id = 5;     // дерево слота (обогащается сервером при извлечении)
}
```

### Стартовый каталог (миграция core v1.9.88)

| code | Семейство | Цена | available | Остаток после миграции |
|---|---|---|---|---|
| `win_lite` | лицензия (дерево 1) | 20 000 | ✅ | квота 0 (задаёт админ) |
| `vpn` | купон | 10 000 | ✅ | 1000 кодов |
| `content_factory` | купон | 30 000 | ✅ | 1000 кодов |
| `win_pro` | лицензия (дерево 2) | 60 000 | ❌ скрыт | 0 |
| `bank_card` | купон | 40 000 | ❌ скрыт | пул пуст |

---

## 2. 🧾 История транзакций: `WinTimeReferralDetails`

`biconom/types/transaction.proto`, oneof `details`:

```protobuf
WinTimeReferralDetails win_time_referral = 24;

message WinTimeReferralDetails {
    uint32 child_distributor_id = 1; // личник, за приглашение которого начислен бонус
}
```

Карточка единоразового реферального бонуса WinTime спонсору (сумма — в `amount`
проводки, валюта `WIN_TIME`). WinTime-часть приза за квест слота отдельного варианта
не имеет — использует общую `SlotQuestRewardDetails`.

---

## 3. 🔐 `client.AuthService.AuthorizeTelegram` 🆕

`biconom/client/auth/auth.proto` — вход или автоматическая регистрация через
Telegram Login Widget **без формы подтверждения** (личность уже подтверждена
HMAC-подписью виджета):

```protobuf
rpc AuthorizeTelegram(TelegramWidgetAuthRequest) returns (TelegramAuthResponse);

message TelegramWidgetAuthRequest {
    string data = 1; // данные виджета: query string "id=…&hash=…" или JSON
    biconom.types.Locale.Id locale = 2; // применяется только при регистрации
    optional RegisterRequest.DistributorRequest distributor_request = 3; // логин + спонсор
}

message TelegramAuthResponse {
    string authorization_bearer = 1; // готовый токен доступа на сессию
}
```

`Confirmation` не создаётся — при успехе сразу возвращается токен сессии.
