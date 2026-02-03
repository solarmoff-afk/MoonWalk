# Moonwalk
Render engine

# Большой рефакторинг / ломающие изменения
На данный момент идёт работа над переходом на отдельный крейт moonwalk_backend в качестве абстрации над wgpu, а также планируются ломающие изменения в сигнатурах функций, MoonWalkError и прочих аспектах движка. Пока-что для использования экспериментальной версии нужно включить фичу modern 

```rust
#[cfg(feature = "modern")]
// Вызов из moonwalk_backend

#[cfg(not(feature = "modern"))]
// Вызов из gpu/abstract
```

## Сборка с фичей modern
```bash
cd moonwalk
cargo build --features modern
```