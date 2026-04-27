# Arena — Арена соревнований

## Описание

Арена — абстракция над лидербордом, представляющая турнирный формат с циклами (раундами).
Дистрибьюторы зарабатывают баллы и соревнуются за ранг.

## Вложенные типы

| Тип | Описание |
|-----|---------|
| `Arena.Id` | Идентификация по числовому ID или строковому имени |
| `Arena.List` | Список арен |
| `Arena.Cycle` | Раунд арены, привязан к одному лидерборду |
| `Arena.Cycle.Id` | Составной ключ: arena_id + seq |
| `Arena.Cycle.Status.Id` | UNSPECIFIED, ACTIVE, FINISHED |
| `Arena.Entry` | Запись участника: distributor_id, value, rank |

## Жизненный цикл

```
Arena → start_cycle → Cycle(Active) → [duration expires] → Cycle(Finished)
                                                            ↓ auto_renew
                                                        new Cycle(Active)
```

## Арены в системе

| ID | Имя | Логика |
|----|-----|--------|
| 1 | wincoins_exchange | Баллы = кол-во купленных WinCoins |
| 2 | tree_1_license | 1 балл за первую активацию лицензии партнёром |
| 3 | tree_2_license | Аналогично для дерева 2 |
| 4 | tree_3_license | Аналогично для дерева 3 |
