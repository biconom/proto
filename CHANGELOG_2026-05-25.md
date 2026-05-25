# CHANGELOG: 25 Мая 2026

Доработки клиентского API вокруг привязки **Telegram** и расширение каталога тем уведомлений (для нужд страницы «Уведомления» в кабинете).

---

## 🚀 Изменения в API

### 1. `client/user/user.proto` — поле `is_already_connected`

В ответ `GetTelegramBotLinkResponse` добавлено булево поле, чтобы фронт мог **одновременно** показать ссылку и понять, нужно ли отрисовать UI как «Подключить» или «Перепривязать».

```protobuf
message GetTelegramBotLinkResponse {
    // Ссылка для подключения к телеграм боту.
    string link = 1;

    // true — у пользователя уже есть привязанный Telegram. Ссылка всё равно
    // возвращается: пользователь может перейти по ней и перепривязать TG
    // к этой же учётной записи (бот обработает relink-сценарий).
    bool is_already_connected = 2;
}
```

**Поведенческие изменения:**

- Раньше при уже привязанном Telegram сервер возвращал ошибку `TELEGRAM_BOT_ALREADY_CONNECTED` → фронту приходилось отдельно скрывать кнопку. Теперь **сервер всегда возвращает ссылку**, плюс флаг.
- Возможна новая ошибка `TELEGRAM_BOT_DISABLED` (`failed_precondition`) — если на стенде вообще выключена TG-интеграция в конфиге. На прод-стенде не встречается, на dev-стенде без секретов — обработать как «функция недоступна».

#### 💻 Пример для Frontend

```javascript
try {
  const { link, isAlreadyConnected } = await UserServiceClient.GetTelegramBotLink({});

  if (isAlreadyConnected) {
    // UI: кнопка «Перепривязать Telegram» + подсказка «Текущий TG: см. в профиле»
    renderRelinkButton(link);
  } else {
    // UI: кнопка «Подключить Telegram»
    renderConnectButton(link);
  }
} catch (e) {
  if (e.code === 'TELEGRAM_BOT_DISABLED') {
    hideTelegramSection(); // на этом стенде TG-интеграция выключена
  } else throw e;
}
```

> 💡 «Текущий привязанный Telegram» (username + дата) уже отдаётся в `client.AccountService.GetCurrent` → `user_profile.telegram` (поле появилось не сегодня — см. ниже).

---

### 2. `client/notification_preferences/notification_preferences.proto` — два новых топика

В enum `Topic` добавлены два новых значения для страницы «Настройки уведомлений» в кабинете.

```protobuf
enum Topic {
    // ... существующие 17 топиков ...

    // Security: ваш Telegram отвязан через личный кабинет (API).
    // Уведомление прилетает в отвязываемый TG-чат с IP / UA / гео.
    // Канал доставки: только Telegram (Mandatory — нельзя отключить).
    TOPIC_TELEGRAM_UNBOUND_BY_API = 18;

    // Депозит обнаружен в сети (BSC tx видна, но ещё ждёт подтверждений).
    // Парный к TOPIC_DEPOSIT_RECEIVED: сначала «обнаружен», потом «зачислен».
    // Каналы: Email/Push/Telegram, всё Optional, default ON.
    TOPIC_DEPOSIT_DETECTED = 19;
}
```

#### Сводная таблица по каналам (для UI настроек)

| Topic | Email | Push | Telegram | Видим в UI? |
|---|---|---|---|---|
| `TOPIC_TELEGRAM_UNBOUND_BY_API` | Forbidden | Forbidden | **Mandatory** | да, как нередактируемая строка «Telegram отвязан через API» |
| `TOPIC_DEPOSIT_DETECTED` | Optional ON | Optional ON | Optional ON | да, как обычный переключатель |

`Mandatory` → toggle не отрисовывать (или disabled). `Forbidden` → строку вовсе не показывать для этого канала. `Optional` → обычный toggle с default из `ChannelTopicPolicy.default_on`.

#### 💻 Пример для Frontend

```javascript
const { items } = await NotificationPreferencesServiceClient.Get({});

for (const item of items) {
  switch (item.topic) {
    case Topic.TOPIC_TELEGRAM_UNBOUND_BY_API:
      // Security: показываем строку с пометкой «всегда включено»
      renderRow(item, { readOnly: true, label: 'Telegram отвязан через личный кабинет' });
      break;
    case Topic.TOPIC_DEPOSIT_DETECTED:
      renderRow(item, { label: 'Депозит обнаружен в сети (до зачисления)' });
      break;
    // ...
  }
}
```

> ⚠️ **Обратная совместимость:** старые клиенты без знания о 18/19 проигнорируют новые значения enum (proto3 default behavior — неизвестные числовые значения сохраняются). Можно деплоить бэк до фронта.

---

## 🔔 Что приходит пользователю в Telegram (контекст для UI-копирайтеров)

Шаблоны сообщений теперь используют **HTML с жирным+моноширинным выделением** (`<b><code>…</code></b>`) для логина и ID — Telegram автоматически делает их tap-to-copy. На UI копировать этот стиль не нужно — это серверная отрисовка.

| Сценарий | Топик | Куда приходит |
|---|---|---|
| Первая привязка TG к аккаунту | (бот-side, не BusinessEvent) | новый TG-чат |
| Перепривязка TG к другому аккаунту | (бот-side) | тот же TG-чат, 2 сообщения подряд |
| Отвязка через `/unsubscribe` команду в боте | (бот-side) | тот же TG-чат |
| **Отвязка через `DeleteTelegram` API** | `TOPIC_TELEGRAM_UNBOUND_BY_API` | отвязанный TG-чат, с IP/UA/гео |
| **Новая регистрация в первую линию** | `TOPIC_NEW_REFERRAL` (level=1) | приглашающий |
| **Депозит замечен в сети** | `TOPIC_DEPOSIT_DETECTED` | владелец кошелька |
| **Депозит зачислен на счёт** | `TOPIC_DEPOSIT_RECEIVED` | владелец кошелька |
| **Вывод выполнен** | `TOPIC_WITHDRAWAL_PROCESSED` | инициатор |

---

## 🛡️ Версионирование backend

Бэк bumped до **`core@1.9.29`** (`Cargo.toml`). Для фронта не критично — proto schema additive.
