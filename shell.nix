let
  nixpkgs = import <nixpkgs> { };
in
  with nixpkgs;
  stdenv.mkDerivation {
    name = "moz_overlay_shell";
    buildInputs = [
      wasm-bindgen-cli
      simple-http-server
      openssl
      pkg-config
      libsodium
      zlib
      binutils
      lld_9
      xorg.libX11
      xorg.libXcursor
      xorg.libXrandr
      xorg.libXi
      libxkbcommon
      wayland
      libGL

      xdg-desktop-portal-hyprland
      libglvnd
      wasm-pack
    ];

  shellHook = ''
    export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${lib.getLib xorg.libX11}/lib:${lib.getLib xorg.libXcursor}/lib:${lib.getLib xorg.libXrandr}/lib:${lib.getLib xorg.libXi}/lib:${lib.getLib libGL}/lib:${lib.getLib wayland}/lib:${lib.getLib libxkbcommon}/lib
  '';
  }

