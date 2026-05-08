import re

with open('/Users/mikel/Documents/GitHub/biconom/proto/biconom/types/transaction.proto', 'r') as f:
    content = f.read()

# Remove TransactionGroup, Transaction, TransactionEntry, TransactionScopeId
# They span from "// TransactionGroup" up to "message HistoryGroup {"
start_idx = content.find('// TransactionGroup - это')
end_idx = content.find('// Агрегированный визуальный блок истории')

if start_idx != -1 and end_idx != -1:
    content = content[:start_idx] + content[end_idx:]

# Insert Direction inside HistoryEntry
direction_code = """
    // Направление движения средств.
    message Direction {
        enum Id {
            UNSPECIFIED = 0;
            // Списание со счета.
            DEBIT = 1;
            // Зачисление на счет.
            CREDIT = 2;
        }
    }
    Direction.Id direction = 2;
"""
content = re.sub(
    r'biconom\.types\.TransactionEntry\.Direction\.Id direction = 2;',
    direction_code.strip(),
    content
)

# Remove TransactionEntryReason
reason_start_idx = content.find('// Причина бухгалтерской проводки.')
if reason_start_idx != -1:
    content = content[:reason_start_idx]

with open('/Users/mikel/Documents/GitHub/biconom/proto/biconom/types/transaction.proto', 'w') as f:
    f.write(content.strip() + '\n')

