# CHANGELOG: 8 Июня 2026

Добавлены **контакты пользователя (email + Telegram)** в модель `biconom.types.User` — для отрисовки блока «Contact Information» на экране профиля партнёра. Поля заполняются только при наличии прав видеть контакты.

---

## 🚀 Изменения в API

### `types/user.proto` — два новых поля в `User` (под `avatar`)

```protobuf
message User {
    uint32 id = 1;
    biconom.types.Presence.Status presence_status = 2;
    uint32 policy_id = 3;
    google.protobuf.Timestamp created_at = 4;
    google.protobuf.Timestamp updated_at = 5;
    optional biconom.types.Image avatar = 6;

    // Email пользователя. Заполняется ТОЛЬКО там, где у запрашивающего есть право видеть контакты.
    // Отсутствует, если прав нет или email не привязан.
    optional string email = 7;

    // Telegram username (без внутреннего telegram_user_id). Условия — как у email.
    optional string telegram_username = 8;
}
```

> 📍 Контакты живут на самом `User`, поэтому в `DistributorService.Get` они приходят по пути
> `Response.account.owner.user.email` / `…user.telegram_username` (owner — это oneof User/Community).

### Где сейчас реально заполняется

На данный момент бэкенд заполняет эти поля **только в ответе `DistributorService.Get`** (просмотр профиля дистрибьютора) и **только если** запрашивающий удовлетворяет хотя бы одному условию:

1. это **его собственный** профиль;
2. целевой дистрибьютор в **его команде по реферальному дереву**;
3. хотя бы **один слот** целевого находится **под слотами** запрашивающего в иерархии;
4. (админ/саппорт) у сессии есть право глобального просмотра дистрибьюторов.

> ⚠️ Важно: во всех **остальных** ответах, где встречается `User` (список партнёров, аккаунт и т. п.),
> поля `email`/`telegram_username` **намеренно пустые** — даже не запрашивайте их там. Это сделано,
> чтобы контакты не утекали без проверки прав. Если контакты понадобятся на других экранах — заводите
> отдельную задачу на бэкенд, само наличие поля в `User` доступ не открывает.

Внутри «разрешённого» случая поле всё равно может отсутствовать, если контакт **не привязан** (нет email / нет активного Telegram) — это нормально, не ошибка.

> ⚠️ Наружу отдаётся **только** `telegram_username`. Числовой `telegram_user_id` НЕ передаётся.

#### 💻 Пример для Frontend

```javascript
const resp = await DistributorServiceClient.Get({ id });
const user = resp.account?.owner?.user; // owner: oneof user | community

const email = user?.email ?? null;
const telegram = user?.telegramUsername ? `@${user.telegramUsername}` : null;

if (email || telegram) {
  renderContactSection({ email, telegram });
} else {
  hideContactSection(); // прав нет ИЛИ контакты не привязаны
}
```

| Состояние поля | Что показать в UI |
|---|---|
| `user.email = "user@mail.com"` | строка Email со значением |
| `user.email` пустое | строку Email скрыть (или «не привязан») |
| `user.telegram_username = "ivan"` | строка Telegram: `@ivan` |
| `user.telegram_username` пустое | строка Telegram: «Not connected» / скрыть |

> 💡 Это контакты **чужого** профиля. Контакты **своего** аккаунта (с датами привязки) по-прежнему
> отдаются отдельно в `client.AccountService.GetCurrent` → `user_profile.email` / `user_profile.telegram`.

---

## 🛡️ Версионирование backend

Бэк: **`core@1.9.66`**. Изменение схемы **additive** (поля 7/8 опциональны в `User`) — старые клиенты их просто игнорируют. Можно деплоить бэк до фронта.
