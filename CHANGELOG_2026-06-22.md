# CHANGELOG: 22 Июня 2026

Добавлен новый глобальный маркетинговый флаг `allow_capacity_upgrade` — управляет доступностью покупки расширения ширины слота через клиентское API.

---

## 🚀 Изменения в API

### Новый маркетинговый флаг: `allow_capacity_upgrade`

Администраторы теперь могут включать/выключать возможность покупки расширения ширины слота для всех пользователей через существующие RPC `GetMarketingFlags` / `SetMarketingFlags`.

### `biconom/admin/marketing/marketing.proto`

В сообщения `GetMarketingFlagsResponse` и `SetMarketingFlagsRequest` добавлено новое поле:

```protobuf
message GetMarketingFlagsResponse {
    bool v2 = 1;
    // Разрешает покупку расширения ширины слота (capacity upgrade).
    bool allow_capacity_upgrade = 2;
}

message SetMarketingFlagsRequest {
    optional bool v2 = 1;
    // Разрешает покупку расширения ширины слота (capacity upgrade).
    optional bool allow_capacity_upgrade = 2;
}
```

### Новый код ошибки: `SLOT_CAPACITY_UPGRADE_DISABLED`

Когда флаг `allow_capacity_upgrade` **выключен** (по умолчанию — выключен), следующие RPC возвращают `FAILED_PRECONDITION` с кодом `SLOT_CAPACITY_UPGRADE_DISABLED`:

- `MarketingService.CalculateCapacityUpgradePrice`
- `MarketingService.PurchaseCapacityUpgrade`

---

## 💻 Как использовать на Frontend

### 1. Проверить состояние флага

Флаг доступен через публичный словарь или через админское API:

```javascript
// Админский способ: прочитать текущие маркетинговые флаги
const flags = await AdminMarketingServiceClient.GetMarketingFlags({});
console.log(flags.allowCapacityUpgrade); // true / false
```

### 2. Скрыть UI расширения ширины, если запрещено

```javascript
if (flags.allowCapacityUpgrade) {
    showCapacityUpgradeButton();
} else {
    hideCapacityUpgradeButton();
}
```

### 3. Обработать ошибку при вызове

Даже если UI не скрыт, бэкенд вернёт ошибку:

```javascript
try {
    const price = await MarketingServiceClient.CalculateCapacityUpgradePrice({
        targetChildrenCapacity: 5
    });
    showPriceDialog(price);
} catch (err) {
    if (err.code === grpc.status.FAILED_PRECONDITION) {
        if (err.message === 'SLOT_CAPACITY_UPGRADE_DISABLED') {
            showToast('Расширение ширины слота временно недоступно');
            return;
        }
    }
    handleGenericError(err);
}
```

### 4. Включить/выключить флаг (админ-панель)

```javascript
// Включить
await AdminMarketingServiceClient.SetMarketingFlags({
    allowCapacityUpgrade: true
});

// Выключить (не затрагивая другие флаги)
await AdminMarketingServiceClient.SetMarketingFlags({
    allowCapacityUpgrade: false
});

// Изменить только v2, не трогая allow_capacity_upgrade — просто не передавать поле
await AdminMarketingServiceClient.SetMarketingFlags({
    v2: true
});
```

> ⚠️ Каждое поле в `SetMarketingFlags` — **независимый бит**. Если поле не передано (`null`/`undefined`) — бит не меняется. `true` — устанавливает, `false` — сбрасывает.

---

## 📋 Сводка изменений

| Что изменилось | Где |
|---|---|
| Новое поле `allow_capacity_upgrade` | `GetMarketingFlagsResponse` (field 2) |
| Новое поле `optional allow_capacity_upgrade` | `SetMarketingFlagsRequest` (field 2) |
| Новый код ошибки `SLOT_CAPACITY_UPGRADE_DISABLED` | `CalculateCapacityUpgradePrice`, `PurchaseCapacityUpgrade` |
