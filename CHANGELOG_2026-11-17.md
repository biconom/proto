# 2026-11-17 — proto 0.3.39 (core 1.10.61)

## Лотерея «5 из 36» — новые сервисы `LotteryService` и `LotteryAdminService`

Суточная лотерея за WinTime: один билет на тираж, раскрытие в 00:00 UTC, призы
(USDT на кошелёк, стейкинг-депозит, WINZU) делятся поровну между угадавшими 5 из 5.
Числа вычисляются после закрытия продаж из `server_seed` (хэш опубликован заранее),
отпечатков всех билетов и хэша блока BSC — их нельзя знать заранее никому.

- Модели: `biconom/types/lottery.proto` (`Lottery.Draw`, `Lottery.Ticket`, `Lottery.Fund`, `Lottery.Prize`).
- Клиент: `biconom/client/lottery/lottery.proto`, описание и алгоритм проверки — `lottery.md`.
- Админ: `biconom/admin/lottery/lottery.proto`.
- Вход фронта лотереи с отдельного поддомена: основной фронт вызывает
  `LotteryService/CreateLaunchCode` и открывает `https://<поддомен>/#code=<code>`;
  фронт лотереи меняет код на Lottery-токен (`ExchangeLaunchCode`), который
  действует только в лотерее.

## История транзакций

`biconom.types.Transaction.Group.Entry.details` — две новые карточки:

| Поле | Когда |
|---|---|
| `LotteryTicketDetails lottery_ticket = 36` | списание WinTime за билет (`draw_id`, `ticket_id`, `numbers`) |
| `LotteryPrizeDetails lottery_prize = 37` | начисление части выигрыша (`draw_id`, `ticket_id`, `prize_kind`) |
