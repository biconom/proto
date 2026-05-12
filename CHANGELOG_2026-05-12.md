# CHANGELOG: 11-12 Мая 2026

За последние 24 часа был реализован колоссальный пласт функционала, охватывающий систему уведомлений (Email + Web Push), управление устройствами, настройки нотификаций, безопасность (SNI TLS, CORS, определение IP) и исправление бизнес-логики.

Ниже приведено подробное описание всех изменений.

---

## 1. 🚀 Глобальные нововведения: Web Push и Система Уведомлений

Мы полностью внедрили систему доставки уведомлений, включая поддержку браузерных Web Push уведомлений и транзакционных Email-сообщений через единую шину событий.

### Новые контракты (gRPC / biconom-proto)
*Где смотреть:* `biconom/proto/biconom/client/`, `biconom/proto/biconom/admin/`
* **Push-уведомления:** Добавлены сервисы `biconom.client.push` и `biconom.admin.push`. Позволяют управлять подписками устройств, привязывать их к сессиям и отвязывать.
* **Настройки уведомлений:** Добавлены `biconom.client.notification_preferences` и `biconom.admin.notification_preferences`. Теперь пользователи могут гибко настраивать, какие уведомления они хотят получать и по каким каналам (Email, Push, Telegram).
* **Token Grant:** Добавлен сервис `biconom.admin.token_grant` для выдачи токенов доступа.

### Инфраструктура в Core
*Где смотреть:* `biconom/core/src/infra/`
* **`push_subscription` & `push_sender`:** Полная реализация протокола Web Push с поддержкой VAPID (генерация ключей, HKDF-шифрование `p256dh` и `auth` payload-ов). Введено персистентное хранилище подписок.
* **`push_audit`:** Аудит всех попыток отправки push-уведомлений.
* **`notification` & `business_worker`:** Разработана единая шина бизнес-событий (`BusinessEvent`). Выделено 15 типов событий, каждое из которых теперь может автоматически диспатчиться по доступным каналам пользователя.
* **`notification_preferences`:** Логика маршрутизации уведомлений (Dispatch Engine) на основе пользовательских политик.
* **`device`:** Введен реестр устройств. Подписки Web Push строго привязываются к `device_id` и сессиям.

---

## 2. 🛡 Инфраструктура, Сеть и Безопасность

* **SNI TLS и HTTP/2:** Переписана логика загрузки сертификатов. Теперь поддерживается SNI (Server Name Indication) и ALPN-протоколы для правильной работы gRPC через HTTP/2 (`src/infra/tls.rs`).
* **Точное определение IP:** Убрана уязвимая логика чтения `X-Forwarded-For`. Теперь реальный IP клиента извлекается напрямую из TCP-соединения (`axum::extract::ConnectInfo`), так как балансировщики не используются (`src/service/mod.rs`).
* **CORS Политики:** Установлены жесткие ограничения. В production разрешены только `https://win2.pro` и `https://biconom.com` (`src/main.rs`).
* **Редизайн email-шаблонов:** 
  * Добавлена поддержка inline-логотипов через `CID` для HTML писем.
  * Исправлена кодировка `utf-8` для корректного отображения HTML в ElasticEmail.
  * Ссылка на поддержку обновлена на `@win2team`.

---

## 3. 🐛 Багфиксы и Улучшения Бизнес-логики

* **Персонализация писем:** 
  Исправлен баг, из-за которого в системных письмах вместо логина пользователя подставлялось пустое поле (`Здравствуйте, `). Теперь `username` корректно извлекается из *первого* дистрибьютора внутри системной сети (`NETWORK_ID_MAIN = 1`).
* **Email Авторизация:** 
  При SignIn по Email-коду система теперь проверяет, существует ли данный Email в базе активных. Если да, логин (юзернейм) будет успешно извлечен и подставлен в письмо с кодом.
* **Stream RPC:** В `biconom.client.confirmation` добавлен метод `SubscribeToFieldStatus` для получения статусов подтверждений по стриму.

---

## 💻 Пример для Frontend: Как подключить Web Push

Команде фронтенда для интеграции Web Push уведомлений необходимо реализовать следующий флоу.

### 1. Получение публичного VAPID ключа
Публичный ключ передается сервером (в конфигурации или через API).

### 2. Регистрация Service Worker и подписка
```javascript
// 1. Регистрируем Service Worker
const registration = await navigator.serviceWorker.register('/sw.js');

// 2. Спрашиваем разрешение у пользователя
const permission = await Notification.requestPermission();
if (permission !== 'granted') throw new Error('Permission denied');

// 3. Подписываемся на пуши
const subscription = await registration.pushManager.subscribe({
    userVisibleOnly: true,
    // applicationServerKey - это VAPID Public Key, конвертированный в Uint8Array
    applicationServerKey: urlBase64ToUint8Array('B...ВАШ_VAPID_PUBLIC_KEY...=')
});

// 4. Извлекаем ключи для отправки на бэкенд
const subJson = subscription.toJSON();
const endpoint = subJson.endpoint;
const p256dh = subJson.keys.p256dh;
const auth = subJson.keys.auth;
```

### 3. Отправка подписки на Backend (через gRPC)
Собранные данные нужно отправить через метод `Subscribe` в сервисе `biconom.client.push.PushService`.

**Пример Payload (на основе proto):**
```json
{
  "device_id": "уникальный-идентификатор-устройства-или-сессии",
  "endpoint": "https://fcm.googleapis.com/fcm/send/...",
  "auth": "auth-ключ-из-подписки",
  "p256dh": "p256dh-ключ-из-подписки"
}
```

### 4. Управление настройками
Для того чтобы дать пользователю возможность включать/выключать пуши, используйте сервис `biconom.client.notification_preferences`.
Метод `UpdatePreferences` позволяет точечно управлять флагами `email_enabled`, `push_enabled` и `telegram_enabled` для различных топиков (например, "Новый реферал", "Зачисление средств").
