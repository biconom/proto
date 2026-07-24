# Сервис: WintimeShopAdminService (Admin)

> Общие модели (`biconom.types.WintimeShop.*`) описаны в
> [`biconom/types/wintime_shop.md`](../../types/wintime_shop.md). Здесь — только
> админский сервис.

## 1. Описание

**`WintimeShopAdminService`** — административное управление WinTime-магазином
(каталог, тираж, статистика). Три зоны ответственности:

1. **Каталог** — создать/обновить определение товара (slug `code`, название, цена
   в WinTime, семейство, привязка к дереву для лицензий), включить/выключить продажу.
2. **Тираж** — пополнение того, что можно продать:
   - `TREE_LICENSE` — целочисленная квота остатка: добавить N / отнять N / установить;
   - `TEXT_COUPON` — пул текстовых кодов: массовая загрузка новых кодов с проверкой
     уникальности **в пределах товара**.
3. **Статистика** — остаток и сколько всего продано.

**Адресация товара.** Изменяющие/читающие методы принимают
`biconom.types.WintimeShop.Product.Id` (`oneof { id | code }`) — товар можно указать
числовым id или строковым slug. Исключение — `UpsertProduct`, где числовой `id`
служит дискриминатором create/update (`0` — создать, иначе — обновить).

## 2. Описание методов (RPC)

### Каталог

#### `rpc UpsertProduct(UpsertProductRequest) returns (WintimeShop.Product)`
- **Назначение**: создать новый товар или обновить существующий.
- **Дискриминатор**: `id == 0` — создать (id назначается сервером и возвращается);
  `id != 0` — обновить товар с этим id.
- **Входные параметры** (`UpsertProductRequest`):
    - `id` (uint32): `0` — создать; иначе — id обновляемого товара.
    - `code` (string): строковый slug. **Обязателен при создании**; нормализуется
      сервером (trim по краям → нижний регистр) и обязан соответствовать `[a-z0-9_]+`.
      При обновлении **игнорируется** (`code` неизменяем).
    - `kind` (`Product.Kind`): семейство товара. При обновлении обязано совпадать с
      текущим.
    - `title` (string): человекочитаемое название.
    - `price_wintime` (uint64): цена в WinTime-токенах (WIN_TIME precision 0), `> 0`.
    - `available` (bool): доступность к покупке сразу после операции.
    - `spec` (oneof): специфика семейства — `TreeLicenseSpec { tree_id }` для
      `TREE_LICENSE`, `TextCouponSpec { details }` для `TEXT_COUPON`; обязана
      соответствовать `kind`.
- **Возвращаемое значение**: полная модель `WintimeShop.Product` (после сохранения).
- **Ошибки**:
    - `InvalidArgument` (`WINTIME_SHOP_CODE_INVALID`) — при создании `code` пуст
      после trim или не соответствует `[a-z0-9_]+`.
    - `InvalidArgument` (`WINTIME_SHOP_CODE_DUPLICATE`) — при создании товар с таким
      `code` уже существует.
    - `InvalidArgument` (`WINTIME_SHOP_KIND_MISMATCH`) — при обновлении `kind` не
      совпал с текущим, либо `spec` не соответствует `kind`.
    - `InvalidArgument` (`WINTIME_SHOP_PRICE_INVALID`) — `price_wintime == 0`.
    - `NotFound` (`WINTIME_SHOP_PRODUCT_NOT_FOUND`) — обновление (`id != 0`), но товар
      не найден.

#### `rpc ListProducts(google.protobuf.Empty) returns (WintimeShop.Product.List)`
- **Назначение**: список **всех** товаров (включая недоступные) — для админ-панели.
- **Возвращаемое значение**: `WintimeShop.Product.List`.

#### `rpc SetAvailability(SetAvailabilityRequest) returns (WintimeShop.Product)`
- **Назначение**: включить/выключить доступность товара к покупке.
- **Входные параметры** (`SetAvailabilityRequest`):
    - `product` (`Product.Id`): товар (по id или code).
    - `available` (bool): новое значение флага.
- **Возвращаемое значение**: обновлённая модель товара.
- **Ошибки**: `NotFound` (`WINTIME_SHOP_PRODUCT_NOT_FOUND`).

### Тираж: лицензии дерева (`TREE_LICENSE`)

#### `rpc AdjustLicenseStock(AdjustLicenseStockRequest) returns (WintimeShop.Product)`
- **Назначение**: изменить квоту остатка товара-лицензии (можно добавить в пул
  несколько лицензий, отнять или установить точное значение).
- **Входные параметры** (`AdjustLicenseStockRequest`):
    - `product` (`Product.Id`): товар (по id или code).
    - `op` (`Op`): `OP_ADD` (+`amount`) / `OP_SUBTRACT` (−`amount`) / `OP_SET`
      (=`amount`).
    - `amount` (uint64): аргумент операции.
- **Возвращаемое значение**: обновлённая модель товара (с новым `stock_remaining`).
- **Ошибки**:
    - `InvalidArgument` (`WINTIME_SHOP_KIND_MISMATCH`) — товар семейства `TEXT_COUPON`
      (у купонов тираж управляется кодами, а не счётчиком).
    - `InvalidArgument` (`WINTIME_SHOP_STOCK_NEGATIVE`) — `OP_SUBTRACT` опустил бы
      остаток ниже 0.
    - `InvalidArgument` — `op == OP_UNSPECIFIED`.
    - `NotFound` (`WINTIME_SHOP_PRODUCT_NOT_FOUND`).

### Тираж: купоны (`TEXT_COUPON`)

#### `rpc LoadCoupons(LoadCouponsRequest) returns (LoadCouponsResponse)`
- **Назначение**: массово загрузить новые текстовые коды в пул товара-купона.
- **Уникальность в пределах товара**: коды, уже присутствующие в пуле этого товара
  (в т.ч. уже выданные), отклоняются; повторы **внутри запроса** также
  схлопываются. Один и тот же код у **разных** товаров — допустим.
- **Входные параметры** (`LoadCouponsRequest`):
    - `product` (`Product.Id`): товар (по id или code).
    - `codes` (repeated string): новые коды. Пустой список — `InvalidArgument`;
      каждый код после trim должен быть непустым.
- **Возвращаемое значение** (`LoadCouponsResponse`):
    - `added` (uint64): сколько кодов реально добавлено;
    - `skipped_duplicates` (uint64): сколько отклонено как дубли (в пуле или внутри запроса);
    - `stock_remaining` (uint64): остаток пула после загрузки.
- **Ошибки**:
    - `InvalidArgument` (`WINTIME_SHOP_KIND_MISMATCH`) — товар семейства `TREE_LICENSE`.
    - `InvalidArgument` (`WINTIME_SHOP_CODES_EMPTY`) — `codes` пуст или все коды пусты после trim.
    - `NotFound` (`WINTIME_SHOP_PRODUCT_NOT_FOUND`).

### Статистика

#### `rpc GetStats(WintimeShop.Product.Id) returns (WintimeShop.Stats)`
- **Назначение**: сводная статистика по товару.
- **Входные параметры**: `Product.Id` (по id или code).
- **Возвращаемое значение** (`WintimeShop.Stats`): `stock_remaining` (остаток —
  квота / коды в пуле), `sold_total` (всего продано за всё время), `available`.
- **Ошибки**: `NotFound` (`WINTIME_SHOP_PRODUCT_NOT_FOUND`).

> Отчёт «кем куплено» отдельным методом не предоставляется — покупателей можно
> получить через существующую историю MLM (ваучеры / транзакции WinTime).

## 3. Права доступа и безопасность

- **Требуемые права**: `Permission::ROOT` (см. `core/src/infra/access_control.rs`).
  По умолчанию выдаётся только `user_id = 1`.
- Токены `Guest` и `Confirmation` отклоняются; принимается только `Session`-токен с
  правом `ROOT`.

## 4. Сценарии использования

- **Завести товар-лицензию** («Win Lite»): `UpsertProduct` с `kind = TREE_LICENSE`,
  `code = "win_lite"`, `price_wintime = 20000`, `tree_license { tree_id = 1 }`; затем
  `AdjustLicenseStock` (`OP_SET`) — задать доступный тираж.
- **Завести товар-купон** (VPN): `UpsertProduct` с `kind = TEXT_COUPON`,
  `code = "vpn"`, `text_coupon { details = "…инструкция…" }`; затем `LoadCoupons` —
  загрузить пул промо-кодов.
- **Пополнить/скорректировать тираж**: `AdjustLicenseStock` (лицензии) или
  `LoadCoupons` (купоны).
- **Скрыть товар с витрины**: `SetAvailability(available = false)`.
- **Проверить остаток и продажи**: `GetStats`.

> **Примечание об именовании ошибок**: коды `WINTIME_SHOP_*` — предполагаемые
> строковые константы уровня `service::mod.rs`, добавляются на этапе реализации
> механики (см. `types/wintime_shop.md`, раздел «Реализация»).
