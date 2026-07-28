# Сервис: WintimeAdminService (Admin)

## 1. Описание

**`WintimeAdminService`** — **сырая** история статус-событий WinTime по
**произвольному** дистрибьютору. Инструмент саппорта: подтвердить пользователю,
когда и почему менялась скорость начисления / произошёл откат множителя.

В отличие от клиентского `biconom.client.wintime.WintimeService.GetMaskHistory`
(этапы с булевыми статусами и пересчётом дохода) отдаёт журнал как есть:
события `Registered` + `MaskChanged` с сырой битовой маской (`1<<0` регистрация,
`1<<1` Lite, `1<<2` Pro) — вид для диагностики. Принимает `distributor_id`.

## 2. Метод

`GetMaskHistory(GetMaskHistoryRequest { distributor_id })` →
`MaskHistoryResponse { items[] }`.

- `distributor_id` обязателен: `0` → `InvalidArgument WINTIME_DISTRIBUTOR_REQUIRED`.
- Ответ — полная история в хронологическом порядке, без пагинации.

## 3. Права доступа

Только `Permission::ROOT`.

## 4. Примечание

История читается из durable-журнала статус-событий WinTime (`wt2_events`) через
индекс по дистрибьютору (`wt2_events_did`); прошлые события дозаполнены в индекс
миграцией v01_09_93. Журнал пересобран v01_09_89 — даты событий бизнес-корректны.
