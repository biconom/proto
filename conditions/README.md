# conditions

```
Пример сервиса с реализованным функционалом блоков:
https://en.scratch-wiki.info/wiki/Blocks
```

### типы данных для *"conditions.proto"*:
```protobuf
// ElementField - инструкция отображения поля ввода данных для элемента
message ElementField {
  uint32 varType = 1; // идентификатор типа переменной
  string placeholder = 2; // подсказка для поля ввода данных
}

// Element - глобальное хранилище элементов
message Element {
  uint32 elementID = 1; // глобальный идентификатор элемента
  uint32 categoryID = 2; // идентификатор категории
  uint32 viewType = 3; // идентификатор визуального сопровождения элемента
  uint32 returnType = 4; // идентификатор возвращаемого типа данных
  string content = 5; // формат данных для отображения / значение элемента (когда является переменной)
  map<string, ElementField> fields = 6; // перечень полей ввода данных используемых в элементе
}

// Component - компонент визуального сопровождения 
message Component {
  uint32 itemID = 1; // порядковый идентификатор (уникальный в каждой группе)
  uint32 itemRefID = 2; // идентификатор предыдущего элемента (если равен нулю, тогда элемент является основным)
  uint32 itemRefLine = 3; // идентификатор ответвления вышестоящего элемента
  uint32 elementID = 4; // идентификатор основного компонента из "Storage" 
  string field = 5; // идентификатор поля в блоке
  map<string, string> fields = 6; // перечень поля и его значения
}

// Storage - хранилище компонентов связанных между собой
message Storage {
  uint32 positionTop = 1; // значение отступа сверху выраженное в пикселях 
  uint32 positionLeft = 2; // значение отступа слева выраженное в пикселях
  repeated Component items = 3; // массив взаимосвязанных компонентов в хранилище
}

// Scene - Сцена всех компонентов с правилами взаимодействия
message Scene {
  uint32 sceneID = 1; // идентификатор сцены
  map<string, uint32> viewTypeIDs = 2; // перечень идентификаторов для визуального сопровождения
  map<string, uint32> varTypeIDs = 3; // перечень идентификаторов для работы с известными типами переменных
  repeated Element elements = 4; // массив основополагающих компонентов
  repeated Storage storages = 5; // массив хранилищ компонентов
}
```

```text
Компонент сопровождает внешний вид:
- как переменная, если вид отображения равен "var";
- как функция, если вид отображения равен "func" и ("returnType" > 0);
- как блок в всех остальных случаях;

Если компонент является функций или переменой с возвращаемым типом данных "bool", применить стиль отобразить с острыми углами.
Если компонент является функций или переменой с возвращаемым типом данных отличающийся от "bool", применить стиль отображения с закругленными углами.

Дерево взаимосвязи строится по уникальной группе состоящей из (itemRefID и itemRefLine), а так же "field" если компонент является результатом для поля ввода данных вышестоящего компонента

Если в блок-компоненте "itemRefLine" == 0, тогда продолжение построения цепочки блоков происходит после "itemRefID";
Если в блок-компоненте "itemRefLine" > 0, тогда продолжение построения цепочки блоков происходит в соответствующем ответвлении у блока "itemRefID";
```

###Пример API ответа в json формате:
```json
{
  "sceneID": 1,
  "viewTypeIDs": {
    "var": 1,
    "func": 2,
    "start": 3,
    "end": 4,
    "break": 5,
    "repeat_number": 6,
    "repeat_until": 7,
    "forever": 8,
    "if": 9,
    "if_else": 10
  },
  "varTypeIDs": {
    "bool": 1,
    "decimal": 6,
    "double": 5,
    "duration": 8,
    "int": 4,
    "string": 2,
    "time": 7,
    "uint": 3
  },
  "elements": [
    {
      "elementID": 1,
      "categoryID": 1,
      "viewType": 2,
      "returnType": 0,
      "content": "wait ${{waitValue}}",
      "fields": {
        "waitValue": {
          "varType": 8,
          "placeholder": ""
        }
      }
    },
    {
      "elementID": 2,
      "categoryID": 1,
      "viewType": 2,
      "returnType": 0,
      "content": "wait until ${{untilValue}}",
      "fields": {
        "untilValue": {
          "varType": 1,
          "placeholder": "please enter bool value"
        }
      }
    },
    {
      "elementID": 3,
      "categoryID": 1,
      "viewType": 2,
      "returnType": 1,
      "content": "${{first}} and ${{second}}",
      "fields": {
        "first": {
          "varType": 1,
          "placeholder": ""
        },
        "second": {
          "varType": 1,
          "placeholder": ""
        }
      }
    },
    {
      "elementID": 4,
      "categoryID": 1,
      "viewType": 2,
      "returnType": 1,
      "content": "not ${{value}}",
      "fields": {
        "value": {
          "varType": 1,
          "placeholder": ""
        }
      }
    },
    {
      "elementID": 5,
      "categoryID": 1,
      "viewType": 1,
      "returnType": 1,
      "content": "true",
      "fields": {}
    },
    {
      "elementID": 6,
      "categoryID": 1,
      "viewType": 1,
      "returnType": 1,
      "content": "false",
      "fields": {}
    },
    {
      "elementID": 7,
      "categoryID": 1,
      "viewType": 9,
      "returnType": 0,
      "content": "if ${{condition}} then",
      "fields": {
        "condition": {
          "varType": 1,
          "placeholder": ""
        }
      }
    },
    {
      "elementID": 8,
      "categoryID": 1,
      "viewType": 10,
      "returnType": 0,
      "content": "if ${{condition}} then ... else ...",
      "fields": {
        "condition": {
          "varType": 1,
          "placeholder": ""
        }
      }
    }
  ],
  "storages": [
    {
      "positionTop": 0,
      "positionLeft": 0,
      "items": [
        {
          "itemID": 1,
          "itemRefID": 0,
          "itemRefLine": 0,
          "elementID": 1,
          "field": "",
          "fields": {
            "waitValue": "15s"
          }
        },
        {
          "itemID": 2,
          "itemRefID": 1,
          "itemRefLine": 0,
          "elementID": 2,
          "field": "",
          "fields": {
            "untilValue": ""
          }
        },
        {
          "itemID": 3,
          "itemRefID": 2,
          "itemRefLine": 1,
          "elementID": 3,
          "field": "untilValue",
          "fields": {
            "first": "false",
            "second": "true"
          }
        },
        {
          "itemID": 4,
          "itemRefID": 3,
          "itemRefLine": 1,
          "elementID": 5,
          "field": "first",
          "fields": {}
        },
        {
          "itemID": 5,
          "itemRefID": 2,
          "itemRefLine": 0,
          "elementID": 7,
          "field": "",
          "fields": {
            "condition": "true"
          }
        },
        {
          "itemID": 6,
          "itemRefID": 5,
          "itemRefLine": 1,
          "elementID": 8,
          "field": "",
          "fields": {
            "condition": "true"
          }
        }
      ]
    }
  ]
}
```