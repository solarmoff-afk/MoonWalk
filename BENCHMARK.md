--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-01-25 23:56:07]

## System Configuration
* **Build:** DEBUG
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:** KABINI (radeonsi, , ACO, DRM 2.50, 6.12.60-1-lts) (Gl)
* **Driver:** 

## Benchmark Results

| Category | Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **Geometry** | Rects Solid x10000 | 60.0 | 45.1 | 16.66 ms | 116 MB |
| **Geometry** | Rects Solid x50000 | 60.1 | 50.4 | 16.65 ms | 131 MB |
| **Geometry** | Rects Solid x100000 | 60.3 | 41.0 | 16.58 ms | 148 MB |
| **Geometry** | Rects Solid x200000 | 55.3 | 30.2 | 18.09 ms | 180 MB |
| **Geometry** | Rects Solid x5000 | 60.0 | 54.5 | 16.66 ms | 176 MB |
| **Geometry** | Rects Rounded x5000 | 60.0 | 52.7 | 16.66 ms | 176 MB |
| **Geometry** | Rects Border x5000 | 60.1 | 54.0 | 16.64 ms | 176 MB |
| **Geometry** | Rects Textured x5000 | 22.5 | 15.1 | 44.38 ms | 176 MB |
| **Geometry** | Rects GradientLinear x5000 | 54.7 | 32.9 | 18.26 ms | 176 MB |
| **Geometry** | Rects GradientRadial x5000 | 52.4 | 41.9 | 19.10 ms | 176 MB |
| **Fill Rate** | Fullscreen Layers x10 | 48.1 | 37.6 | 20.78 ms | 176 MB |
| **Fill Rate** | Fullscreen Layers x50 | 15.4 | 13.0 | 64.96 ms | 175 MB |
| **Fill Rate** | Fullscreen Layers x100 | 8.4 | 7.8 | 119.67 ms | 175 MB |
| **Text** | Text Short x1000 | 59.2 | 29.5 | 16.89 ms | 189 MB |
| **Text** | Text Short x10000 | 22.9 | 19.3 | 43.64 ms | 224 MB |
| **Text** | Text Paragraph x500 | 59.2 | 17.9 | 16.90 ms | 255 MB |
| **Text** | Text Justified x500 | 60.9 | 53.4 | 16.41 ms | 255 MB |
| **Dynamics** | Moving Rects x10000 | 24.5 | 22.4 | 40.75 ms | 255 MB |
| **Dynamics** | Moving Rects x50000 | 11.4 | 10.7 | 87.93 ms | 255 MB |
| **Dynamics** | Moving Text x2000 | 3.2 | 3.1 | 315.56 ms | 255 MB |
| **Vector** | Static x5000 | 61.0 | 39.4 | 16.38 ms | 255 MB |
| **Vector** | Dynamic (10 pts) | 60.0 | 54.5 | 16.67 ms | 255 MB |
| **Vector** | Dynamic (100 pts) | 60.0 | 44.4 | 16.67 ms | 255 MB |
| **Vector** | Dynamic (300 pts) | 39.1 | 32.0 | 25.58 ms | 255 MB |
| **Effects** | Color Matrix | 53.4 | 32.7 | 18.74 ms | 255 MB |
| **Effects** | Blur (r=5) | 31.4 | 22.4 | 31.89 ms | 255 MB |
| **Effects** | Blur (r=15) | 18.0 | 15.1 | 55.52 ms | 255 MB |
| **Effects** | Blur (r=30) | 18.1 | 15.9 | 55.40 ms | 255 MB |
| **Simulation** | Real Scene (200 Rects + 50 Text) | 27.6 | 24.1 | 36.20 ms | 255 MB |


--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-01-06 22:10:59]

## System Configuration
* **Build:** DEBUG
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:** KABINI (radeonsi, , ACO, DRM 2.50, 6.12.60-1-lts) (Gl)
* **Driver:** 

## Benchmark Results

| Category | Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **Geometry** | Rects Solid x10000 | 60.0 | 45.4 | 16.66 ms | 108 MB |
| **Geometry** | Rects Solid x50000 | 60.0 | 48.2 | 16.67 ms | 123 MB |
| **Geometry** | Rects Solid x100000 | 60.0 | 45.9 | 16.67 ms | 141 MB |
| **Geometry** | Rects Solid x200000 | 60.0 | 39.6 | 16.66 ms | 173 MB |
| **Geometry** | Rects Solid x5000 | 60.0 | 53.7 | 16.66 ms | 175 MB |
| **Geometry** | Rects Rounded x5000 | 60.0 | 51.5 | 16.67 ms | 175 MB |
| **Geometry** | Rects Border x5000 | 60.0 | 48.2 | 16.66 ms | 175 MB |
| **Geometry** | Rects Textured x5000 | 42.1 | 33.3 | 23.73 ms | 175 MB |
| **Geometry** | Rects GradientLinear x5000 | 60.0 | 48.0 | 16.66 ms | 175 MB |
| **Geometry** | Rects GradientRadial x5000 | 60.0 | 47.4 | 16.66 ms | 175 MB |
| **Fill Rate** | Fullscreen Layers x10 | 60.2 | 48.1 | 16.62 ms | 175 MB |
| **Fill Rate** | Fullscreen Layers x50 | 29.7 | 25.2 | 33.63 ms | 175 MB |
| **Fill Rate** | Fullscreen Layers x100 | 16.3 | 14.7 | 61.25 ms | 175 MB |
| **Text** | Text Short x1000 | 60.0 | 38.3 | 16.65 ms | 188 MB |
| **Text** | Text Short x10000 | 41.8 | 27.3 | 23.91 ms | 223 MB |
| **Text** | Text Paragraph x500 | 60.0 | 24.0 | 16.67 ms | 255 MB |
| **Text** | Text Justified x500 | 60.0 | 42.2 | 16.66 ms | 255 MB |
| **Dynamics** | Moving Rects x10000 | 25.5 | 23.3 | 39.20 ms | 255 MB |
| **Dynamics** | Moving Rects x50000 | 11.8 | 10.1 | 84.89 ms | 255 MB |
| **Dynamics** | Moving Text x2000 | 3.3 | 3.2 | 302.02 ms | 258 MB |
| **Vector** | Static x5000 | 60.0 | 47.3 | 16.68 ms | 258 MB |
| **Vector** | Dynamic (10 pts) | 60.0 | 51.1 | 16.67 ms | 258 MB |
| **Vector** | Dynamic (100 pts) | 60.0 | 52.3 | 16.67 ms | 258 MB |
| **Vector** | Dynamic (300 pts) | 40.7 | 36.2 | 24.57 ms | 259 MB |
| **Effects** | Color Matrix | 60.0 | 53.1 | 16.67 ms | 259 MB |
| **Effects** | Blur (r=5) | 45.3 | 38.5 | 22.07 ms | 259 MB |
| **Effects** | Blur (r=15) | 21.5 | 20.3 | 46.59 ms | 259 MB |
| **Effects** | Blur (r=30) | 21.5 | 20.3 | 46.48 ms | 259 MB |
| **Simulation** | Real Scene (200 Rects + 50 Text) | 32.0 | 27.1 | 31.22 ms | 259 MB |


--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-01-04 13:42:18]

## System Configuration
* **Build:** RELEASE
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:** KABINI (radeonsi, , ACO, DRM 2.50, 6.12.60-1-lts) (Gl)
* **Driver:** 

## Benchmark Results

| Category | Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **Geometry** | Rects Solid x10000 | 60.0 | 45.3 | 16.66 ms | 97 MB |
| **Geometry** | Rects Solid x50000 | 60.0 | 53.3 | 16.67 ms | 113 MB |
| **Geometry** | Rects Solid x100000 | 60.0 | 47.8 | 16.66 ms | 132 MB |
| **Geometry** | Rects Solid x200000 | 60.6 | 36.4 | 16.50 ms | 163 MB |
| **Geometry** | Rects Solid x5000 | 60.0 | 48.0 | 16.67 ms | 163 MB |
| **Geometry** | Rects Rounded x5000 | 60.0 | 55.9 | 16.66 ms | 163 MB |
| **Geometry** | Rects Border x5000 | 60.0 | 55.8 | 16.67 ms | 163 MB |
| **Geometry** | Rects Textured x5000 | 24.4 | 20.4 | 41.01 ms | 163 MB |
| **Geometry** | Rects GradientLinear x5000 | 58.7 | 37.4 | 17.03 ms | 163 MB |
| **Geometry** | Rects GradientRadial x5000 | 56.9 | 39.9 | 17.58 ms | 163 MB |
| **Fill Rate** | Fullscreen Layers x10 | 52.5 | 37.5 | 19.06 ms | 164 MB |
| **Fill Rate** | Fullscreen Layers x50 | 16.7 | 15.7 | 59.77 ms | 164 MB |
| **Fill Rate** | Fullscreen Layers x100 | 9.1 | 8.9 | 109.92 ms | 164 MB |
| **Text** | Text Short x1000 | 60.4 | 36.5 | 16.55 ms | 175 MB |
| **Text** | Text Short x10000 | 24.4 | 23.0 | 41.00 ms | 210 MB |
| **Text** | Text Paragraph x500 | 60.9 | 18.0 | 16.43 ms | 243 MB |
| **Text** | Text Justified x500 | 60.0 | 49.1 | 16.66 ms | 243 MB |
| **Dynamics** | Moving Rects x10000 | 60.0 | 56.5 | 16.66 ms | 243 MB |
| **Dynamics** | Moving Rects x50000 | 38.4 | 27.0 | 26.06 ms | 243 MB |
| **Dynamics** | Moving Text x2000 | 17.5 | 16.5 | 57.01 ms | 243 MB |
| **Vector** | Static x5000 | 60.2 | 43.1 | 16.61 ms | 243 MB |
| **Vector** | Dynamic (10 pts) | 60.0 | 55.3 | 16.67 ms | 243 MB |
| **Vector** | Dynamic (100 pts) | 60.0 | 55.2 | 16.67 ms | 243 MB |
| **Vector** | Dynamic (300 pts) | 60.0 | 54.2 | 16.67 ms | 243 MB |
| **Effects** | Color Matrix | 60.0 | 53.8 | 16.67 ms | 243 MB |
| **Effects** | Blur (r=5) | 35.2 | 27.6 | 28.43 ms | 243 MB |
| **Effects** | Blur (r=15) | 19.5 | 15.9 | 51.16 ms | 238 MB |
| **Effects** | Blur (r=30) | 19.4 | 15.3 | 51.52 ms | 238 MB |
| **Simulation** | Real Scene (200 Rects + 50 Text) | 46.5 | 35.2 | 21.52 ms | 237 MB |


--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-01-04 13:22:30]

## System Configuration
* **Build:** DEBUG
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:** KABINI (radeonsi, , ACO, DRM 2.50, 6.12.60-1-lts) (Gl)
* **Driver:** 

## Benchmark Results

| Category | Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **Geometry** | Rects Solid x10000 | 60.0 | 47.3 | 16.66 ms | 116 MB |
| **Geometry** | Rects Solid x50000 | 60.0 | 54.1 | 16.66 ms | 132 MB |
| **Geometry** | Rects Solid x100000 | 57.6 | 12.8 | 17.37 ms | 151 MB |
| **Geometry** | Rects Solid x200000 | 60.9 | 33.7 | 16.41 ms | 181 MB |
| **Geometry** | Rects Solid x5000 | 60.0 | 49.1 | 16.66 ms | 181 MB |
| **Geometry** | Rects Rounded x5000 | 60.0 | 56.5 | 16.66 ms | 181 MB |
| **Geometry** | Rects Border x5000 | 60.0 | 53.2 | 16.66 ms | 181 MB |
| **Geometry** | Rects Textured x5000 | 21.0 | 19.3 | 47.67 ms | 181 MB |
| **Geometry** | Rects GradientLinear x5000 | 56.6 | 44.8 | 17.68 ms | 181 MB |
| **Geometry** | Rects GradientRadial x5000 | 55.4 | 42.7 | 18.05 ms | 181 MB |
| **Fill Rate** | Fullscreen Layers x10 | 50.1 | 40.8 | 19.98 ms | 181 MB |
| **Fill Rate** | Fullscreen Layers x50 | 14.1 | 13.3 | 70.96 ms | 181 MB |
| **Fill Rate** | Fullscreen Layers x100 | 7.4 | 7.2 | 134.87 ms | 181 MB |
| **Text** | Text Short x1000 | 61.2 | 30.9 | 16.35 ms | 194 MB |
| **Text** | Text Short x10000 | 21.5 | 19.6 | 46.41 ms | 228 MB |
| **Text** | Text Paragraph x500 | 60.0 | 22.9 | 16.66 ms | 261 MB |
| **Text** | Text Justified x500 | 60.0 | 50.5 | 16.65 ms | 261 MB |
| **Dynamics** | Moving Rects x10000 | 24.2 | 23.0 | 41.33 ms | 262 MB |
| **Dynamics** | Moving Rects x50000 | 11.2 | 11.0 | 89.12 ms | 262 MB |
| **Dynamics** | Moving Text x2000 | 3.2 | 3.1 | 314.58 ms | 262 MB |
| **Vector** | Static x5000 | 60.0 | 50.4 | 16.67 ms | 262 MB |
| **Vector** | Dynamic (10 pts) | 60.0 | 55.7 | 16.66 ms | 262 MB |
| **Vector** | Dynamic (100 pts) | 60.0 | 56.8 | 16.67 ms | 262 MB |
| **Vector** | Dynamic (300 pts) | 40.0 | 35.8 | 24.99 ms | 262 MB |
| **Effects** | Color Matrix | 60.5 | 54.5 | 16.52 ms | 262 MB |
| **Effects** | Blur (r=5) | 36.7 | 31.3 | 27.25 ms | 262 MB |
| **Effects** | Blur (r=15) | 19.4 | 17.8 | 51.47 ms | 262 MB |
| **Effects** | Blur (r=30) | 19.6 | 17.8 | 50.96 ms | 262 MB |
| **Simulation** | Real Scene (200 Rects + 50 Text) | 28.4 | 23.2 | 35.26 ms | 262 MB |


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


