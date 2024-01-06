# mandelbrot
Interactive zoomable Mandelbrot set

# Running webGPU enabled chrome on linux

`NIXPKGS_ALLOW_UNFREE=1 nix run github:r-k-b/browser-previews#google-chrome-dev --impure -- --enable-unsafe-webgpu --enable-features=Vulkan,UseSkiaRenderer`
