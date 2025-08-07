## API Documentation

## Документация API

- **Авторизация:** После успешной авторизации или регистрации токен необходимо передавать в заголовке `authorization`. Вы можете включить префикс "Bearer " перед токеном или просто передать токен напрямую. Оба формата принимаются.

- **Переключение аккаунтов:** Для переключения между аккаунтами используйте заголовок `account_id` и укажите числовой идентификатор требуемого аккаунта.

- **Код для стейджинга:** Для целей тестирования на стейджинговой среде код, генерируемый для подтверждения по электронной почте, равен "123456".

- **Возможные ошибки:** Ниже приведен список возможных кодов ошибок, с которыми вы можете столкнуться:
    - `GUEST_ACCESS_DENIED`: Доступ запрещен для гостевых пользователей.
    - `UNIMPLEMENTED`: Запрошенная функциональность не реализована.
    - `UNAUTHENTICATED`: Запрос требует аутентификации.
    - `AUTHORIZATION_EMPTY`: Заголовок авторизации пуст.
    - `AUTHORIZATION_WRONG`: Заголовок авторизации имеет некорректный формат или содержит недействительный токен.
    - `INTERNAL_ERROR`: Произошла непредвиденная внутренняя ошибка.
    - `LIST_LIMIT_MAX_EXCEEDED`: Запрошенный лимит списка превышает максимально допустимый.
    - `SORT_INVALID_DIRECTION`: Указано неверное направление сортировки.
    - `SORT_DIRECTION_UNSPECIFIED`: Направление сортировки не указано, когда это требуется.
    - `SESSION_IS_ACTIVE`: Сессия уже активна.
    - `SESSION_IS_PENDING`: Сессия находится в состоянии ожидания.
    - `SESSION_NOT_FOUND`: Сессия не найдена.
    - `SESSION_NOT_PRESENT`: Сессия отсутствует.
    - `SESSION_INVALID_FORMAT`: Сессия имеет неверный формат.
    - `SESSION_INACTIVE_CANCELLED`: Сессия неактивна из-за отмены.
    - `SESSION_INACTIVE_REVOKED`: Сессия неактивна из-за отзыва.
    - `SESSION_INACTIVE_EXPIRED`: Сессия неактивна из-за истечения срока действия.
    - `SESSION_INACTIVE_TERMINATED`: Сессия неактивна из-за завершения.
    - `SESSION_INVALID_FILTER_STATUS`: Статус фильтра сессии недействителен.
    - `SESSION_CURSOR_INVALID`: Курсор сессии недействителен.
    - `USER_BANNED`: Пользователь заблокирован.
    - `USER_PASSWORD_NOT_SET`: Пароль пользователя не установлен.
    - `USER_EMAIL_BUSY`: Указанный адрес электронной почты уже используется.
    - `USER_EMAIL_ALREADY_EXISTS`: Аккаунт с такой почтой уже существует.
    - `USER_NOT_AVAILABLE`: Пользователь недоступен.
    - `ACCESS_SCOPE_DENIED`: Доступ к запрошенной области запрещен.
    - `ACCOUNT_NOT_FOUND`: Аккаунт не найден.
    - `ACCOUNT_NOT_AVAILABLE`: Аккаунт недоступен.
    - `ACCOUNT_INVALID_FORMAT`: Аккаунт имеет неверный формат.
    - `COMMUNITY_UNSUPPORTED`: Указанное сообщество не поддерживается.
    - `GOOGLE_AUTHENTICATOR_NOT_ENABLED`: Google Authenticator не включен для этого аккаунта.
    - `GOOGLE_AUTHENTICATOR_ALREADY_ENABLED`: Google Authenticator уже включен для этого аккаунта.
    - `MNEMONIC_NOT_ENABLED`: Мнемоника не включена для этого аккаунта.
    - `MNEMONIC_ALREADY_ENABLED`: Мнемоника уже включена для этого аккаунта.
    - `CONFIRMATION_NOT_FOUND`: Подтверждение не найдено.
    - `CONFIRMATION_STATUS_NOT_ACTIVE`: Статус подтверждения неактивен.
    - `CONFIRMATION_INVALID_FILTER_STATUS`: Статус фильтра подтверждения недействителен.
    - `CONFIRMATION_CURSOR_INVALID`: Курсор подтверждения недействителен.
    - `CONFIRMATION_FIELD_NOT_FOUND`: Поле подтверждения не найдено.
    - `CONFIRMATION_FIELD_NOT_CHANEL`: Поле подтверждения не является допустимым каналом.
    - `CONFIRMATION_FIELD_ATTEMPT_LIMITED`: Количество попыток для поля подтверждения ограничено.
    - `CONFIRMATION_FIELD_ATTEMPT_DURATION`: Прошло время для попыток подтверждения поля.
    - `CONFIRMATION_FIELD_DUPLICATE`: Поле подтверждения является дубликатом.