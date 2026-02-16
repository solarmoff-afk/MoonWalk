--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-02-16 14:10:22]

## System Configuration
* **Build:** DEBUG
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:**  ()
* **Driver:** 

## Benchmark Results

| Category | Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **Geometry** | Rects Solid x10000 | 60.0 | 57.8 | 16.66 ms | 110 MB |
| **Geometry** | Rects Solid x50000 | 19.3 | 17.5 | 51.72 ms | 127 MB |
| **Geometry** | Rects Solid x100000 | 10.1 | 9.8 | 98.87 ms | 144 MB |
| **Geometry** | Rects Solid x200000 | 5.2 | 5.0 | 191.84 ms | 174 MB |
| **Geometry** | Rects Solid x5000 | 46.0 | 40.1 | 21.75 ms | 174 MB |
| **Geometry** | Rects Rounded x5000 | 46.1 | 39.9 | 21.70 ms | 174 MB |
| **Geometry** | Rects Border x5000 | 46.3 | 41.7 | 21.60 ms | 174 MB |
| **Geometry** | Rects Textured x5000 | 26.2 | 22.2 | 38.24 ms | 174 MB |
| **Geometry** | Rects GradientLinear x5000 | 40.1 | 35.4 | 24.95 ms | 174 MB |
| **Geometry** | Rects GradientRadial x5000 | 38.9 | 35.4 | 25.69 ms | 174 MB |
| **Fill Rate** | Fullscreen Layers x10 | 40.9 | 35.0 | 24.47 ms | 174 MB |
| **Fill Rate** | Fullscreen Layers x50 | 16.4 | 16.0 | 60.89 ms | 174 MB |
| **Fill Rate** | Fullscreen Layers x100 | 8.8 | 8.6 | 114.15 ms | 174 MB |
| **Text** | Text Short x1000 | 10.5 | 8.8 | 94.95 ms | 187 MB |
| **Text** | Text Short x10000 | 1.3 | 1.2 | 789.55 ms | 222 MB |
| **Text** | Text Paragraph x500 | 3.6 | 3.5 | 275.11 ms | 232 MB |
| **Text** | Text Justified x500 | 3.6 | 3.5 | 274.63 ms | 232 MB |
| **Dynamics** | Moving Rects x10000 | 26.5 | 23.9 | 37.72 ms | 232 MB |
| **Dynamics** | Moving Rects x50000 | 12.0 | 11.4 | 83.28 ms | 233 MB |
| **Dynamics** | Moving Text x2000 | 3.1 | 3.0 | 324.03 ms | 233 MB |
| **Vector** | Static x5000 | 35.0 | 32.4 | 28.61 ms | 233 MB |
| **Vector** | Dynamic (10 pts) | 47.9 | 41.8 | 20.87 ms | 233 MB |
| **Vector** | Dynamic (100 pts) | 42.4 | 39.8 | 23.57 ms | 233 MB |
| **Vector** | Dynamic (300 pts) | 28.3 | 17.3 | 35.39 ms | 233 MB |
| **Effects** | Color Matrix | 40.9 | 26.5 | 24.44 ms | 233 MB |
| **Effects** | Blur (r=5) | 28.9 | 25.8 | 34.64 ms | 233 MB |
| **Effects** | Blur (r=15) | 20.9 | 18.7 | 47.84 ms | 233 MB |
| **Effects** | Blur (r=30) | 1.5 | 0.4 | 655.56 ms | 233 MB |
| **Simulation** | Real Scene (200 Rects + 50 Text) | 1.0 | 1.0 | 999.99 ms | 233 MB |


--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-02-16 13:32:51]

## System Configuration
* **Build:** DEBUG
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:**  ()
* **Driver:** 

## Benchmark Results

| Category | Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **Geometry** | Rects Solid x10000 | 60.0 | 56.9 | 16.66 ms | 110 MB |
| **Geometry** | Rects Solid x50000 | 60.0 | 56.0 | 16.67 ms | 120 MB |
| **Geometry** | Rects Solid x100000 | 60.0 | 56.3 | 16.66 ms | 135 MB |
| **Geometry** | Rects Solid x200000 | 60.0 | 57.0 | 16.66 ms | 161 MB |
| **Geometry** | Rects Solid x5000 | 60.0 | 56.5 | 16.66 ms | 162 MB |
| **Geometry** | Rects Rounded x5000 | 60.0 | 55.5 | 16.66 ms | 162 MB |
| **Geometry** | Rects Border x5000 | 60.0 | 55.0 | 16.66 ms | 162 MB |
| **Geometry** | Rects Textured x5000 | 60.0 | 57.0 | 16.66 ms | 162 MB |
| **Geometry** | Rects GradientLinear x5000 | 60.0 | 57.4 | 16.66 ms | 162 MB |
| **Geometry** | Rects GradientRadial x5000 | 60.0 | 55.8 | 16.66 ms | 162 MB |
| **Fill Rate** | Fullscreen Layers x10 | 60.0 | 55.5 | 16.67 ms | 162 MB |
| **Fill Rate** | Fullscreen Layers x50 | 60.0 | 54.7 | 16.66 ms | 162 MB |
| **Fill Rate** | Fullscreen Layers x100 | 60.0 | 57.7 | 16.67 ms | 162 MB |
| **Text** | Text Short x1000 | 60.0 | 57.7 | 16.66 ms | 162 MB |
| **Text** | Text Short x10000 | 60.0 | 57.5 | 16.66 ms | 162 MB |
| **Text** | Text Paragraph x500 | 60.0 | 54.9 | 16.67 ms | 162 MB |
| **Text** | Text Justified x500 | 60.0 | 57.1 | 16.66 ms | 162 MB |
| **Dynamics** | Moving Rects x10000 | 60.0 | 56.7 | 16.66 ms | 163 MB |
| **Dynamics** | Moving Rects x50000 | 60.7 | 54.0 | 16.48 ms | 163 MB |
| **Dynamics** | Moving Text x2000 | 60.0 | 49.0 | 16.66 ms | 163 MB |
| **Vector** | Static x5000 | 60.0 | 57.1 | 16.67 ms | 163 MB |
| **Vector** | Dynamic (10 pts) | 60.0 | 57.2 | 16.66 ms | 163 MB |
| **Vector** | Dynamic (100 pts) | 60.0 | 57.3 | 16.66 ms | 163 MB |
| **Vector** | Dynamic (300 pts) | 47.5 | 40.4 | 21.05 ms | 163 MB |
| **Effects** | Color Matrix | 60.0 | 56.5 | 16.66 ms | 163 MB |
| **Effects** | Blur (r=5) | 48.2 | 40.2 | 20.73 ms | 163 MB |
| **Effects** | Blur (r=15) | 23.1 | 17.7 | 43.29 ms | 163 MB |
| **Effects** | Blur (r=30) | 23.4 | 16.6 | 42.76 ms | 163 MB |
| **Simulation** | Real Scene (200 Rects + 50 Text) | 60.0 | 57.2 | 16.66 ms | 163 MB |


--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-02-14 17:17:32]

## System Configuration
* **Build:** DEBUG
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:**  ()
* **Driver:** 

## Benchmark Results

| Category | Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **Geometry** | Rects Solid x10000 | 60.0 | 50.7 | 16.66 ms | 110 MB |
| **Geometry** | Rects Solid x50000 | 60.0 | 51.0 | 16.67 ms | 126 MB |
| **Geometry** | Rects Solid x100000 | 60.0 | 35.6 | 16.67 ms | 146 MB |
| **Geometry** | Rects Solid x200000 | 60.0 | 43.9 | 16.66 ms | 174 MB |
| **Geometry** | Rects Solid x5000 | 60.1 | 47.1 | 16.65 ms | 174 MB |
| **Geometry** | Rects Rounded x5000 | 60.0 | 42.2 | 16.68 ms | 174 MB |
| **Geometry** | Rects Border x5000 | 60.0 | 50.9 | 16.67 ms | 174 MB |
| **Geometry** | Rects Textured x5000 | 24.0 | 17.2 | 41.71 ms | 174 MB |
| **Geometry** | Rects GradientLinear x5000 | 60.0 | 44.7 | 16.67 ms | 174 MB |
| **Geometry** | Rects GradientRadial x5000 | 60.9 | 44.2 | 16.42 ms | 174 MB |
| **Fill Rate** | Fullscreen Layers x10 | 54.4 | 37.9 | 18.40 ms | 174 MB |
| **Fill Rate** | Fullscreen Layers x50 | 16.1 | 13.4 | 62.24 ms | 174 MB |
| **Fill Rate** | Fullscreen Layers x100 | 8.6 | 7.9 | 116.72 ms | 174 MB |
| **Text** | Text Short x1000 | 60.0 | 26.7 | 16.66 ms | 186 MB |
| **Text** | Text Short x10000 | 24.4 | 20.6 | 41.01 ms | 221 MB |
| **Text** | Text Paragraph x500 | 59.4 | 16.4 | 16.84 ms | 253 MB |
| **Text** | Text Justified x500 | 60.0 | 48.4 | 16.66 ms | 253 MB |
| **Dynamics** | Moving Rects x10000 | 24.3 | 20.7 | 41.21 ms | 253 MB |
| **Dynamics** | Moving Rects x50000 | 11.7 | 10.1 | 85.60 ms | 253 MB |
| **Dynamics** | Moving Text x2000 | 2.9 | 2.8 | 343.24 ms | 253 MB |
| **Vector** | Static x5000 | 61.2 | 44.5 | 16.35 ms | 253 MB |
| **Vector** | Dynamic (10 pts) | 60.0 | 52.4 | 16.67 ms | 253 MB |
| **Vector** | Dynamic (100 pts) | 60.6 | 43.9 | 16.50 ms | 253 MB |
| **Vector** | Dynamic (300 pts) | 36.4 | 28.5 | 27.46 ms | 253 MB |
| **Effects** | Color Matrix | 59.9 | 39.4 | 16.70 ms | 254 MB |
| **Effects** | Blur (r=5) | 31.3 | 9.3 | 31.92 ms | 254 MB |
| **Effects** | Blur (r=15) | 19.0 | 16.7 | 52.51 ms | 254 MB |
| **Effects** | Blur (r=30) | 19.1 | 16.8 | 52.31 ms | 254 MB |
| **Simulation** | Real Scene (200 Rects + 50 Text) | 26.2 | 21.6 | 38.15 ms | 254 MB |


--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-02-14 14:57:18]

## System Configuration
* **Build:** RELEASE
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:**  ()
* **Driver:** 

## Benchmark Results

| Category | Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **Geometry** | Rects Solid x10000 | 60.0 | 46.2 | 16.66 ms | 92 MB |
| **Geometry** | Rects Solid x50000 | 60.0 | 49.5 | 16.66 ms | 107 MB |
| **Geometry** | Rects Solid x100000 | 60.0 | 49.2 | 16.66 ms | 128 MB |
| **Geometry** | Rects Solid x200000 | 60.4 | 54.9 | 16.57 ms | 154 MB |
| **Geometry** | Rects Solid x5000 | 60.0 | 48.0 | 16.66 ms | 154 MB |
| **Geometry** | Rects Rounded x5000 | 60.0 | 50.1 | 16.67 ms | 154 MB |
| **Geometry** | Rects Border x5000 | 60.0 | 52.2 | 16.66 ms | 154 MB |
| **Geometry** | Rects Textured x5000 | 25.1 | 18.0 | 39.78 ms | 154 MB |
| **Geometry** | Rects GradientLinear x5000 | 60.0 | 54.3 | 16.65 ms | 154 MB |
| **Geometry** | Rects GradientRadial x5000 | 60.0 | 46.6 | 16.66 ms | 154 MB |
| **Fill Rate** | Fullscreen Layers x10 | 58.0 | 40.5 | 17.24 ms | 154 MB |
| **Fill Rate** | Fullscreen Layers x50 | 16.6 | 14.7 | 60.16 ms | 154 MB |
| **Fill Rate** | Fullscreen Layers x100 | 8.8 | 8.3 | 113.51 ms | 154 MB |
| **Text** | Text Short x1000 | 60.0 | 33.9 | 16.67 ms | 169 MB |
| **Text** | Text Short x10000 | 25.0 | 21.4 | 40.00 ms | 203 MB |
| **Text** | Text Paragraph x500 | 59.4 | 17.3 | 16.85 ms | 236 MB |
| **Text** | Text Justified x500 | 60.0 | 54.0 | 16.66 ms | 236 MB |
| **Dynamics** | Moving Rects x10000 | 60.0 | 53.5 | 16.68 ms | 236 MB |
| **Dynamics** | Moving Rects x50000 | 37.2 | 28.4 | 26.92 ms | 236 MB |
| **Dynamics** | Moving Text x2000 | 16.9 | 16.1 | 59.20 ms | 236 MB |
| **Vector** | Static x5000 | 60.0 | 46.0 | 16.66 ms | 236 MB |
| **Vector** | Dynamic (10 pts) | 60.0 | 56.4 | 16.67 ms | 236 MB |
| **Vector** | Dynamic (100 pts) | 59.9 | 54.4 | 16.69 ms | 236 MB |
| **Vector** | Dynamic (300 pts) | 58.7 | 34.8 | 17.04 ms | 236 MB |
| **Effects** | Color Matrix | 60.0 | 48.9 | 16.68 ms | 236 MB |
| **Effects** | Blur (r=5) | 38.3 | 28.9 | 26.08 ms | 236 MB |
| **Effects** | Blur (r=15) | 20.5 | 16.7 | 48.84 ms | 236 MB |
| **Effects** | Blur (r=30) | 20.1 | 16.9 | 49.70 ms | 236 MB |
| **Simulation** | Real Scene (200 Rects + 50 Text) | 52.2 | 40.1 | 19.16 ms | 236 MB |


--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-02-14 14:30:36]

## System Configuration
* **Build:** DEBUG
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:**  ()
* **Driver:** 

## Benchmark Results

| Category | Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **Geometry** | Rects Solid x10000 | 60.0 | 47.2 | 16.65 ms | 110 MB |
| **Geometry** | Rects Solid x50000 | 60.0 | 47.1 | 16.66 ms | 125 MB |
| **Geometry** | Rects Solid x100000 | 60.0 | 36.5 | 16.66 ms | 144 MB |
| **Geometry** | Rects Solid x200000 | 60.5 | 34.9 | 16.54 ms | 173 MB |
| **Geometry** | Rects Solid x5000 | 60.0 | 52.0 | 16.67 ms | 175 MB |
| **Geometry** | Rects Rounded x5000 | 60.0 | 49.6 | 16.66 ms | 175 MB |
| **Geometry** | Rects Border x5000 | 60.1 | 45.5 | 16.65 ms | 175 MB |
| **Geometry** | Rects Textured x5000 | 24.9 | 18.7 | 40.11 ms | 175 MB |
| **Geometry** | Rects GradientLinear x5000 | 60.0 | 48.7 | 16.66 ms | 175 MB |
| **Geometry** | Rects GradientRadial x5000 | 60.0 | 45.1 | 16.67 ms | 175 MB |
| **Fill Rate** | Fullscreen Layers x10 | 58.0 | 41.2 | 17.25 ms | 175 MB |
| **Fill Rate** | Fullscreen Layers x50 | 16.5 | 13.4 | 60.45 ms | 175 MB |
| **Fill Rate** | Fullscreen Layers x100 | 8.8 | 8.2 | 113.69 ms | 175 MB |
| **Text** | Text Short x1000 | 60.0 | 28.7 | 16.66 ms | 187 MB |
| **Text** | Text Short x10000 | 25.2 | 21.3 | 39.69 ms | 222 MB |
| **Text** | Text Paragraph x500 | 60.0 | 18.2 | 16.66 ms | 254 MB |
| **Text** | Text Justified x500 | 60.0 | 49.7 | 16.67 ms | 254 MB |
| **Dynamics** | Moving Rects x10000 | 24.4 | 21.3 | 40.95 ms | 254 MB |
| **Dynamics** | Moving Rects x50000 | 11.6 | 11.0 | 85.97 ms | 254 MB |
| **Dynamics** | Moving Text x2000 | 2.9 | 2.4 | 345.14 ms | 255 MB |
| **Vector** | Static x5000 | 60.3 | 52.0 | 16.58 ms | 255 MB |
| **Vector** | Dynamic (10 pts) | 60.2 | 49.7 | 16.62 ms | 255 MB |
| **Vector** | Dynamic (100 pts) | 60.2 | 45.0 | 16.61 ms | 255 MB |
| **Vector** | Dynamic (300 pts) | 38.4 | 30.8 | 26.07 ms | 259 MB |
| **Effects** | Color Matrix | 60.0 | 40.2 | 16.66 ms | 259 MB |
| **Effects** | Blur (r=5) | 36.8 | 29.4 | 27.21 ms | 259 MB |
| **Effects** | Blur (r=15) | 19.7 | 17.6 | 50.67 ms | 262 MB |
| **Effects** | Blur (r=30) | 19.7 | 17.8 | 50.70 ms | 262 MB |
| **Simulation** | Real Scene (200 Rects + 50 Text) | 26.0 | 20.8 | 38.49 ms | 262 MB |


--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-02-14 14:02:20]

## System Configuration
* **Build:** DEBUG
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:**  ()
* **Driver:** 

## Benchmark Results

| Category | Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **Geometry** | Rects Solid x10000 | 60.0 | 49.4 | 16.67 ms | 111 MB |
| **Geometry** | Rects Solid x50000 | 60.0 | 46.8 | 16.66 ms | 127 MB |
| **Geometry** | Rects Solid x100000 | 60.0 | 49.2 | 16.66 ms | 145 MB |
| **Geometry** | Rects Solid x200000 | 60.0 | 30.7 | 16.67 ms | 179 MB |
| **Geometry** | Rects Solid x5000 | 60.0 | 53.2 | 16.66 ms | 181 MB |
| **Geometry** | Rects Rounded x5000 | 60.0 | 53.7 | 16.66 ms | 181 MB |
| **Geometry** | Rects Border x5000 | 60.0 | 52.8 | 16.67 ms | 181 MB |
| **Geometry** | Rects Textured x5000 | 24.7 | 19.0 | 40.49 ms | 181 MB |
| **Geometry** | Rects GradientLinear x5000 | 60.0 | 50.0 | 16.66 ms | 181 MB |
| **Geometry** | Rects GradientRadial x5000 | 60.0 | 46.4 | 16.65 ms | 181 MB |
| **Fill Rate** | Fullscreen Layers x10 | 56.2 | 42.1 | 17.80 ms | 181 MB |
| **Fill Rate** | Fullscreen Layers x50 | 16.5 | 15.1 | 60.49 ms | 181 MB |
| **Fill Rate** | Fullscreen Layers x100 | 8.8 | 8.3 | 113.88 ms | 181 MB |
| **Text** | Text Short x1000 | 60.3 | 25.1 | 16.58 ms | 194 MB |
| **Text** | Text Short x10000 | 24.7 | 21.3 | 40.47 ms | 229 MB |
| **Text** | Text Paragraph x500 | 60.7 | 18.0 | 16.47 ms | 262 MB |
| **Text** | Text Justified x500 | 60.9 | 45.0 | 16.41 ms | 262 MB |
| **Dynamics** | Moving Rects x10000 | 24.8 | 21.5 | 40.35 ms | 262 MB |
| **Dynamics** | Moving Rects x50000 | 11.6 | 10.7 | 85.94 ms | 262 MB |
| **Dynamics** | Moving Text x2000 | 3.0 | 2.9 | 336.41 ms | 262 MB |
| **Vector** | Static x5000 | 61.0 | 52.3 | 16.40 ms | 262 MB |
| **Vector** | Dynamic (10 pts) | 59.9 | 50.2 | 16.69 ms | 262 MB |
| **Vector** | Dynamic (100 pts) | 60.1 | 50.8 | 16.64 ms | 262 MB |
| **Vector** | Dynamic (300 pts) | 36.0 | 18.7 | 27.78 ms | 262 MB |
| **Effects** | Color Matrix | 60.2 | 44.6 | 16.62 ms | 262 MB |
| **Effects** | Blur (r=5) | 35.4 | 28.5 | 28.25 ms | 262 MB |
| **Effects** | Blur (r=15) | 19.2 | 17.4 | 51.96 ms | 262 MB |
| **Effects** | Blur (r=30) | 19.2 | 17.1 | 52.16 ms | 262 MB |
| **Simulation** | Real Scene (200 Rects + 50 Text) | 27.5 | 24.2 | 36.37 ms | 262 MB |


--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-02-14 13:43:21]

## System Configuration
* **Build:** DEBUG
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:**  ()
* **Driver:** 

## Benchmark Results

| Category | Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **Geometry** | Rects Solid x10000 | 60.0 | 48.5 | 16.67 ms | 110 MB |
| **Geometry** | Rects Solid x50000 | 60.0 | 51.6 | 16.66 ms | 125 MB |
| **Geometry** | Rects Solid x100000 | 60.0 | 47.6 | 16.65 ms | 143 MB |
| **Geometry** | Rects Solid x200000 | 60.0 | 42.7 | 16.66 ms | 176 MB |
| **Geometry** | Rects Solid x5000 | 60.0 | 52.7 | 16.67 ms | 176 MB |
| **Geometry** | Rects Rounded x5000 | 60.1 | 55.3 | 16.65 ms | 176 MB |
| **Geometry** | Rects Border x5000 | 60.0 | 54.5 | 16.66 ms | 176 MB |
| **Geometry** | Rects Textured x5000 | 31.7 | 24.6 | 31.51 ms | 176 MB |
| **Geometry** | Rects GradientLinear x5000 | 60.0 | 53.5 | 16.67 ms | 176 MB |
| **Geometry** | Rects GradientRadial x5000 | 60.0 | 50.6 | 16.66 ms | 176 MB |
| **Fill Rate** | Fullscreen Layers x10 | 60.7 | 50.6 | 16.48 ms | 176 MB |
| **Fill Rate** | Fullscreen Layers x50 | 21.5 | 19.1 | 46.56 ms | 176 MB |
| **Fill Rate** | Fullscreen Layers x100 | 11.3 | 10.8 | 88.28 ms | 176 MB |
| **Text** | Text Short x1000 | 60.1 | 38.6 | 16.64 ms | 191 MB |
| **Text** | Text Short x10000 | 32.1 | 28.5 | 31.11 ms | 225 MB |
| **Text** | Text Paragraph x500 | 60.0 | 21.4 | 16.66 ms | 256 MB |
| **Text** | Text Justified x500 | 60.0 | 50.1 | 16.67 ms | 256 MB |
| **Dynamics** | Moving Rects x10000 | 24.4 | 20.2 | 40.96 ms | 257 MB |
| **Dynamics** | Moving Rects x50000 | 11.6 | 10.9 | 86.07 ms | 257 MB |
| **Dynamics** | Moving Text x2000 | 2.9 | 2.8 | 340.60 ms | 260 MB |
| **Vector** | Static x5000 | 60.2 | 49.9 | 16.62 ms | 260 MB |
| **Vector** | Dynamic (10 pts) | 60.0 | 51.7 | 16.66 ms | 260 MB |
| **Vector** | Dynamic (100 pts) | 60.0 | 53.0 | 16.67 ms | 260 MB |
| **Vector** | Dynamic (300 pts) | 35.3 | 15.4 | 28.32 ms | 260 MB |
| **Effects** | Color Matrix | 59.9 | 45.1 | 16.69 ms | 261 MB |
| **Effects** | Blur (r=5) | 40.2 | 33.4 | 24.86 ms | 261 MB |
| **Effects** | Blur (r=15) | 20.4 | 18.6 | 49.00 ms | 261 MB |
| **Effects** | Blur (r=30) | 20.4 | 19.3 | 48.96 ms | 261 MB |
| **Simulation** | Real Scene (200 Rects + 50 Text) | 27.2 | 22.9 | 36.74 ms | 261 MB |


--------------------------------------------------------------------------------
## MoonWalk Performance Audit [2026-02-14 11:22:24]

## System Configuration
* **Build:** DEBUG
* **OS:** Arch Linux 
* **CPU:** AMD A8-6410 APU with AMD Radeon R5 Graphics
* **RAM:** 3 GB
* **GPU:**  ()
* **Driver:** 

## Benchmark Results

| Category | Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **Geometry** | Rects Solid x10000 | 60.0 | 49.3 | 16.66 ms | 114 MB |
| **Geometry** | Rects Solid x50000 | 60.0 | 52.0 | 16.67 ms | 129 MB |
| **Geometry** | Rects Solid x100000 | 60.0 | 46.8 | 16.66 ms | 146 MB |
| **Geometry** | Rects Solid x200000 | 60.0 | 57.2 | 16.66 ms | 179 MB |
| **Geometry** | Rects Solid x5000 | 60.0 | 57.2 | 16.67 ms | 181 MB |
| **Geometry** | Rects Rounded x5000 | 60.0 | 57.3 | 16.66 ms | 181 MB |
| **Geometry** | Rects Border x5000 | 60.0 | 57.1 | 16.66 ms | 181 MB |
| **Geometry** | Rects Textured x5000 | 28.2 | 22.1 | 35.51 ms | 181 MB |
| **Geometry** | Rects GradientLinear x5000 | 60.0 | 57.6 | 16.66 ms | 181 MB |
| **Geometry** | Rects GradientRadial x5000 | 60.0 | 56.3 | 16.66 ms | 181 MB |
| **Fill Rate** | Fullscreen Layers x10 | 59.8 | 46.7 | 16.72 ms | 181 MB |
| **Fill Rate** | Fullscreen Layers x50 | 16.8 | 15.8 | 59.44 ms | 181 MB |
| **Fill Rate** | Fullscreen Layers x100 | 8.9 | 7.7 | 112.85 ms | 181 MB |
| **Text** | Text Short x1000 | 60.0 | 26.0 | 16.66 ms | 193 MB |
| **Text** | Text Short x10000 | 25.4 | 22.8 | 39.38 ms | 228 MB |
| **Text** | Text Paragraph x500 | 59.4 | 17.5 | 16.84 ms | 261 MB |
| **Text** | Text Justified x500 | 60.0 | 53.1 | 16.66 ms | 261 MB |
| **Dynamics** | Moving Rects x10000 | 27.0 | 25.3 | 37.03 ms | 261 MB |
| **Dynamics** | Moving Rects x50000 | 12.2 | 11.7 | 81.81 ms | 261 MB |
| **Dynamics** | Moving Text x2000 | 3.3 | 3.2 | 303.26 ms | 261 MB |
| **Vector** | Static x5000 | 60.0 | 51.7 | 16.67 ms | 261 MB |
| **Vector** | Dynamic (10 pts) | 60.0 | 47.4 | 16.66 ms | 261 MB |
| **Vector** | Dynamic (100 pts) | 60.0 | 54.1 | 16.66 ms | 261 MB |
| **Vector** | Dynamic (300 pts) | 45.9 | 35.5 | 21.79 ms | 261 MB |
| **Effects** | Color Matrix | 60.0 | 58.0 | 16.66 ms | 261 MB |
| **Effects** | Blur (r=5) | 37.3 | 27.8 | 26.78 ms | 261 MB |
| **Effects** | Blur (r=15) | 20.3 | 16.1 | 49.31 ms | 261 MB |
| **Effects** | Blur (r=30) | 20.4 | 15.6 | 48.94 ms | 261 MB |
| **Simulation** | Real Scene (200 Rects + 50 Text) | 30.8 | 26.9 | 32.42 ms | 261 MB |


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


