# Скрипты MoonWalk
Здесь представлены скрипты на bash которые автоматизируют задачи.

## test.sh
```bash
./scripts/test.sh
```

### Флаги
- Нет

### Задача
- Запускать cargo test крейта MoonWalk

## build_android.sh
```bash
./scripts/test.sh

./scripts/build_android.sh --arm32
./scripts/build_android.sh --chromeos
./scripts/build_android.sh --all
```

### Флаги
- `--all` собрать под все архитектуры (arm64, armeabi, x86_64)
- `--arm32` включает компиляцию для armeabi-v7a. Без этого флага компиляция будет только под arm64-v8a, с ним и под arm64-v8a и под armeabi-v7a
- `--chromeos` собрать под x86_64 для эмуляторов android и Chrome Os

### Задача
- Собрать .so файлы для ОС android под архитектуры arm64-v8a и armeabi-v7a. Скрипт не компилирует для x86 и так как эта архитектура устарела. x86_64 для эмуляторов и chrome os