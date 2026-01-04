--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-01-04 11:38:43]

## System Configuration
* **Build:** RELEASE
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:** KABINI (radeonsi, , ACO, DRM 2.50, 6.12.60-1-lts) (Gl)
* **Driver:** 

## Benchmark Results

| Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | ---: | ---: | ---: | ---: |
| **Rects Small (Untextured) x10000** | 60.0 | 58.2 | 16.66 ms | 147 MB |
| **Rects Small (Untextured) x50000** | 60.0 | 58.7 | 16.66 ms | 163 MB |
| **Rects FullScreen (Overdraw) x100** | 8.7 | 8.5 | 115.24 ms | 158 MB |
| **Rects Textured x10000** | 60.0 | 58.8 | 16.66 ms | 158 MB |
| **Real Scene (200 Rects + 50 Text Moving)** | 60.0 | 59.1 | 16.66 ms | 160 MB |
| **Text Static x5000** | 13.3 | 12.5 | 75.35 ms | 182 MB |
| **Rects Moving x10000** | 60.0 | 59.5 | 16.66 ms | 182 MB |
| **Color Matrix Filter** | 60.0 | 58.6 | 16.66 ms | 182 MB |
| **Blur Heavy (r=20.0)** | 21.1 | 19.2 | 47.34 ms | 183 MB |
| **Vector Dynamic 50 pts** | 60.0 | 58.9 | 16.66 ms | 184 MB |


--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-01-04 11:37:30]

## System Configuration
* **Build:** DEBUG
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:** KABINI (radeonsi, , ACO, DRM 2.50, 6.12.60-1-lts) (Gl)
* **Driver:** 

## Benchmark Results

| Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | ---: | ---: | ---: | ---: |
| **Rects Small (Untextured) x10000** | 60.0 | 59.1 | 16.66 ms | 141 MB |
| **Rects Small (Untextured) x50000** | 60.0 | 58.3 | 16.67 ms | 156 MB |
| **Rects FullScreen (Overdraw) x100** | 8.6 | 8.1 | 115.84 ms | 159 MB |
| **Rects Textured x10000** | 60.0 | 59.0 | 16.66 ms | 158 MB |
| **Real Scene (200 Rects + 50 Text Moving)** | 60.0 | 58.4 | 16.66 ms | 160 MB |
| **Text Static x5000** | 13.2 | 12.4 | 75.61 ms | 186 MB |
| **Rects Moving x10000** | 45.8 | 44.1 | 21.84 ms | 186 MB |
| **Color Matrix Filter** | 60.0 | 57.4 | 16.66 ms | 187 MB |
| **Blur Heavy (r=20.0)** | 21.1 | 19.2 | 47.47 ms | 182 MB |
| **Vector Dynamic 50 pts** | 60.0 | 58.6 | 16.66 ms | 179 MB |


