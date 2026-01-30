# MoonWalk Backend
> Work in progress / В разработке

Отдельный крейт без утечек абстрации (кроме случаев когда нужно сохранение обратной совместимости, но тогда создаётся вторая, современная функция без зависимостей от wgpu) на основе крейтов **wgpu** и **image**. Цель: рефакторинг MoonWalk, а также вынести большое количество сложной работы в эту абстракцию

# Готово
- BackendContext
- BackendEncoder
- BackendTexture
- BackendBuffer

# В разработке (wip)
- BackendPipeline
- BackendRenderPass